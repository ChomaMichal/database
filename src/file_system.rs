//! `FileSystem` + `File` traits, with two implementations:
//!
//!   * `StdFs` / `StdFile` — backed by `std::fs`, sandboxed under a root dir.
//!   * `MemFs` / `MemFile` — entirely in-memory; each instance is isolated.
//!
//! Why two traits?
//!
//! Splitting `open` / `delete` onto a `FileSystem` (rather than putting
//! them as associated functions on `File`) is the move that makes DST
//! possible: every `Sim` owns its own `MemFs`, so tests running in
//! parallel never share state, and fault injection (slow disk, short
//! writes, fsync-then-crash) can be implemented per-instance.
//!
//! Both traits intentionally take `&self`, not `&mut self`. Interior
//! mutability (via `Mutex`) lives inside the implementations so the
//! filesystem can be cheaply shared (or cloned, since both impls are
//! `Clone` and Arc-backed).

use std::fmt::Debug;
use std::io;
use std::path::{Path, PathBuf};

/// Re-export so callers don't need to pull in std::io just for the seek enum.
pub use std::io::SeekFrom;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OpenMode {
    /// O_RDONLY. Errors if the file does not exist.
    Read,
    /// O_WRONLY | O_CREAT | O_TRUNC. Wipes the file on open.
    Write,
    /// O_RDWR | O_CREAT. Creates if missing, preserves existing contents.
    ReadWrite,
    /// O_WRONLY | O_CREAT | O_APPEND.
    Append,
}

pub trait FileSystem: Send + Sync {
    type File: File;

    fn open(&self, path: &str, mode: OpenMode) -> io::Result<Self::File>;
    fn delete(&self, path: &str) -> io::Result<()>;
}

pub trait File: Send + Debug {
    /// Explicit close. `Drop` also closes; the explicit form lets
    /// implementations surface deferred errors (buffered flushes,
    /// fsync failures, late allocation on NFS, etc.).
    fn close(self) -> io::Result<()>
    where
        Self: Sized;

    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
    fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
    fn lseek(&mut self, pos: SeekFrom) -> io::Result<u64>;
}

// ===========================================================================
// 1. Real filesystem (std::fs), sandboxed under a root directory
// ===========================================================================

#[derive(Clone, Debug)]
pub struct StdFs {
    root: PathBuf,
}

impl StdFs {
    /// Create a filesystem rooted at `root`. The directory is created
    /// if it doesn't exist. All `open` / `delete` paths are joined to
    /// this root (and forced to be relative).
    pub fn new(root: impl AsRef<Path>) -> io::Result<Self> {
        let root = root.as_ref().to_path_buf();
        std::fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    /// Resolve a caller-supplied path under `root`, stripping leading
    /// separators so an absolute-looking path can't escape the sandbox
    /// via the std `Path::join` "absolute wins" rule.
    ///
    /// NOTE: this does *not* defend against `..` traversal. For
    /// production use, canonicalize the result and check it's still a
    /// prefix of `root`, or use `openat2`/`O_BENEATH` on Linux.
    fn resolve(&self, path: &str) -> PathBuf {
        let rel = path.trim_start_matches(|c: char| c == '/' || c == '\\');
        self.root.join(rel)
    }
}

impl FileSystem for StdFs {
    type File = StdFile;

    fn open(&self, path: &str, mode: OpenMode) -> io::Result<StdFile> {
        let full = self.resolve(path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut opts = std::fs::OpenOptions::new();
        match mode {
            OpenMode::Read => {
                opts.read(true);
            }
            OpenMode::Write => {
                opts.write(true).create(true).truncate(true);
            }
            OpenMode::ReadWrite => {
                opts.read(true).write(true).create(true);
            }
            OpenMode::Append => {
                opts.append(true).create(true);
            }
        }
        Ok(StdFile {
            inner: opts.open(full)?,
        })
    }

    fn delete(&self, path: &str) -> io::Result<()> {
        std::fs::remove_file(self.resolve(path))
    }
}

#[derive(Debug)]
pub struct StdFile {
    inner: std::fs::File,
}

impl File for StdFile {
    fn close(self) -> io::Result<()> {
        // If you care about flush/sync errors at close time, call
        // self.inner.sync_all()? before dropping.
        drop(self.inner);
        Ok(())
    }

    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        io::Read::read(&mut self.inner, buf)
    }

    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        io::Write::write(&mut self.inner, buf)
    }

    fn lseek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        io::Seek::seek(&mut self.inner, pos)
    }
}

// ===========================================================================
// 2. In-memory filesystem — each MemFs instance is isolated
// ===========================================================================

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

type Inode = Arc<Mutex<Vec<u8>>>;

#[derive(Clone, Default, Debug)]
pub struct MemFs {
    /// Cloning a `MemFs` clones the Arc — both handles see the same files.
    /// Construct fresh `MemFs::new()` instances when you want isolated FSes.
    inner: Arc<Mutex<BTreeMap<String, Inode>>>,
}

impl MemFs {
    pub fn new() -> Self {
        Self::default()
    }

    /// Test helper: list every path currently present.
    pub fn list(&self) -> Vec<String> {
        self.inner.lock().unwrap().keys().cloned().collect()
    }

    /// Test helper: snapshot a path's contents (useful for assertions).
    pub fn snapshot(&self, path: &str) -> Option<Vec<u8>> {
        let fs = self.inner.lock().unwrap();
        fs.get(path).map(|i| i.lock().unwrap().clone())
    }
}

impl FileSystem for MemFs {
    type File = MemFile;

    fn open(&self, path: &str, mode: OpenMode) -> io::Result<MemFile> {
        // Hold the FS lock only long enough to clone out the Arc handle
        // to the file's contents.
        let inode: Inode = {
            let mut fs = self.inner.lock().unwrap();
            match mode {
                OpenMode::Read => fs
                    .get(path)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, path.to_string()))?
                    .clone(),
                OpenMode::Write => {
                    let inode = fs
                        .entry(path.to_string())
                        .or_insert_with(|| Arc::new(Mutex::new(Vec::new())))
                        .clone();
                    inode.lock().unwrap().clear(); // truncate
                    inode
                }
                OpenMode::ReadWrite | OpenMode::Append => fs
                    .entry(path.to_string())
                    .or_insert_with(|| Arc::new(Mutex::new(Vec::new())))
                    .clone(),
            }
        };

        let (can_read, can_write, append) = match mode {
            OpenMode::Read => (true, false, false),
            OpenMode::Write => (false, true, false),
            OpenMode::ReadWrite => (true, true, false),
            OpenMode::Append => (false, true, true),
        };

        let cursor = if append {
            inode.lock().unwrap().len() as u64
        } else {
            0
        };

        Ok(MemFile {
            contents: inode,
            cursor,
            can_read,
            can_write,
            append,
        })
    }

    fn delete(&self, path: &str) -> io::Result<()> {
        let mut fs = self.inner.lock().unwrap();
        if fs.remove(path).is_none() {
            return Err(io::Error::new(io::ErrorKind::NotFound, path.to_string()));
        }
        // POSIX-style: existing open handles keep working, because each
        // one holds its own Arc<Mutex<Vec<u8>>>. The path is just gone
        // from the directory.
        Ok(())
    }
}

#[derive(Debug)]
pub struct MemFile {
    contents: Inode,
    cursor: u64,
    can_read: bool,
    can_write: bool,
    append: bool,
}

impl File for MemFile {
    fn close(self) -> io::Result<()> {
        Ok(())
    }

    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if !self.can_read {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "not opened for read",
            ));
        }
        let data = self.contents.lock().unwrap();
        let start = self.cursor as usize;
        if start >= data.len() {
            return Ok(0); // EOF
        }
        let n = std::cmp::min(buf.len(), data.len() - start);
        buf[..n].copy_from_slice(&data[start..start + n]);
        self.cursor += n as u64;
        Ok(n)
    }

    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if !self.can_write {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "not opened for write",
            ));
        }
        let mut data = self.contents.lock().unwrap();
        if self.append {
            // O_APPEND: every write goes to current EOF, atomically wrt
            // other writers on the same inode.
            self.cursor = data.len() as u64;
        }
        let start = self.cursor as usize;
        // Sparse semantics: writing past EOF zero-fills the hole.
        if start > data.len() {
            data.resize(start, 0);
        }
        let end = start + buf.len();
        if end > data.len() {
            data.resize(end, 0);
        }
        data[start..end].copy_from_slice(buf);
        self.cursor = end as u64;
        Ok(buf.len())
    }

    fn lseek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let len = self.contents.lock().unwrap().len() as u64;
        let new = match pos {
            SeekFrom::Start(n) => n,
            SeekFrom::End(d) => offset(len, d)?,
            SeekFrom::Current(d) => offset(self.cursor, d)?,
        };
        self.cursor = new;
        Ok(new)
    }
}

fn offset(base: u64, delta: i64) -> io::Result<u64> {
    let s = base as i128 + delta as i128;
    if s < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "seek before start of file",
        ));
    }
    if s > u64::MAX as i128 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "seek past u64 max",
        ));
    }
    Ok(s as u64)
}

mod test {
    use super::*;

    /// Generic exercise: write, seek, read back, append, delete, then
    /// verify the path is gone.
    fn round_trip<FS: FileSystem>(fs: &FS, path: &str) {
        // Cleanup from any prior run.
        let _ = fs.delete(path);

        // Open for write, lay down 13 bytes.
        let mut f = fs.open(path, OpenMode::Write).expect("open Write");
        assert_eq!(f.write(b"hello, world!").unwrap(), 13);
        assert_eq!(f.lseek(SeekFrom::Current(0)).unwrap(), 13);
        f.close().expect("close");

        // Re-open for read; seek around.
        let mut f = fs.open(path, OpenMode::Read).expect("open Read");
        f.lseek(SeekFrom::Start(7)).unwrap();
        let mut buf = [0u8; 5];
        assert_eq!(f.read(&mut buf).unwrap(), 5);
        assert_eq!(&buf, b"world");

        // Read just the '!' then EOF.
        assert_eq!(f.read(&mut buf).unwrap(), 1);
        assert_eq!(buf[0], b'!');
        assert_eq!(f.read(&mut buf).unwrap(), 0);
        f.close().unwrap();

        // Append a couple more bytes.
        let mut f = fs.open(path, OpenMode::Append).unwrap();
        f.write(b"!!").unwrap();
        f.close().unwrap();

        // Verify final contents.
        let mut f = fs.open(path, OpenMode::Read).unwrap();
        let mut all = Vec::new();
        let mut tmp = [0u8; 8];
        loop {
            let n = f.read(&mut tmp).unwrap();
            if n == 0 {
                break;
            }
            all.extend_from_slice(&tmp[..n]);
        }
        assert_eq!(&all, b"hello, world!!!");
        f.close().unwrap();

        // Delete and confirm it's gone.
        fs.delete(path).unwrap();
        let err = fs.open(path, OpenMode::Read).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn mem_fs_round_trip() {
        let fs = MemFs::new();
        round_trip(&fs, "data/round_trip.bin");
    }

    #[test]
    fn std_fs_round_trip() {
        // Sandboxed under a unique tempdir per test process.
        let dir = std::env::temp_dir().join(format!(
            "distkv-stdfs-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(0),
        ));
        let fs = StdFs::new(&dir).expect("create StdFs");
        round_trip(&fs, "data/round_trip.bin");
        // Best-effort cleanup; ignore errors.
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Two `MemFs` instances are completely isolated from each other —
    /// this is the whole reason we split FileSystem off File.
    #[test]
    fn mem_fs_instances_are_isolated() {
        let a = MemFs::new();
        let b = MemFs::new();

        a.open("shared.bin", OpenMode::Write)
            .unwrap()
            .write(b"a-data")
            .unwrap();

        // `b` doesn't see the file `a` created.
        let err = b.open("shared.bin", OpenMode::Read).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
        assert!(b.list().is_empty());
        assert_eq!(a.list(), vec!["shared.bin".to_string()]);
    }

    /// Cloning a `MemFs`, however, shares the underlying namespace —
    /// useful when you want to hand a handle to another component.
    #[test]
    fn mem_fs_clone_shares_namespace() {
        let a = MemFs::new();
        let b = a.clone();

        a.open("k.bin", OpenMode::Write)
            .unwrap()
            .write(b"v")
            .unwrap();

        assert_eq!(b.snapshot("k.bin"), Some(b"v".to_vec()));
    }

    /// Two handles in the same FS on the same path see each other's writes
    /// (POSIX-style "shared inode").
    #[test]
    fn mem_fs_shared_inode() {
        let fs = MemFs::new();

        let mut a = fs.open("shared", OpenMode::ReadWrite).unwrap();
        a.write(b"abc").unwrap();
        a.close().unwrap();

        // ReadWrite (not Write — Write would truncate).
        let mut b = fs.open("shared", OpenMode::ReadWrite).unwrap();
        b.lseek(SeekFrom::End(0)).unwrap();
        b.write(b"def").unwrap();
        b.close().unwrap();

        assert_eq!(fs.snapshot("shared"), Some(b"abcdef".to_vec()));
    }

    /// Sparse-write semantics: seeking past EOF and writing zero-fills.
    #[test]
    fn mem_sparse_write() {
        let fs = MemFs::new();
        let mut f = fs.open("sparse", OpenMode::Write).unwrap();
        f.lseek(SeekFrom::Start(5)).unwrap();
        f.write(b"X").unwrap();
        f.close().unwrap();

        assert_eq!(fs.snapshot("sparse"), Some(vec![0, 0, 0, 0, 0, b'X']));
    }

    /// Mode-based permission enforcement.
    #[test]
    fn mem_permissions() {
        let fs = MemFs::new();
        // Create first so Read mode can open it.
        fs.open("perms", OpenMode::Write).unwrap().close().unwrap();

        let mut r = fs.open("perms", OpenMode::Read).unwrap();
        assert_eq!(
            r.write(b"nope").unwrap_err().kind(),
            std::io::ErrorKind::PermissionDenied,
        );

        let mut w = fs.open("perms", OpenMode::Write).unwrap();
        let mut buf = [0u8; 4];
        assert_eq!(
            w.read(&mut buf).unwrap_err().kind(),
            std::io::ErrorKind::PermissionDenied,
        );
    }

    /// Unlink-while-open: deleting a path doesn't invalidate existing handles.
    #[test]
    fn mem_unlink_while_open() {
        let fs = MemFs::new();
        let mut f = fs.open("doomed", OpenMode::ReadWrite).unwrap();
        f.write(b"still here").unwrap();

        fs.delete("doomed").unwrap();
        assert!(fs.list().is_empty());

        // But the open handle still works.
        f.lseek(SeekFrom::Start(0)).unwrap();
        let mut buf = [0u8; 16];
        let n = f.read(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"still here");
    }
}
