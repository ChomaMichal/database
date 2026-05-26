use std::fs;
use std::fs::OpenOptions;
use std::os::unix::fs::FileExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::sync::{Arc, RwLock};

use super::PAGE_SIZE;

static FILE_ARRAY: RwLock<Vec<Arc<FileStatic>>> = RwLock::new(Vec::new());

struct FileStatic {
    name: String,
    inner: std::fs::File,
}

//this implements caching
#[derive(Clone)]
pub struct File {
    index: usize,
}

impl File {
    fn get_file(&self) -> Arc<FileStatic> {
        FILE_ARRAY
            .read()
            .unwrap()
            .get(self.index)
            .expect("incorrect index passed to file")
            .clone()
    }

    fn register_file(name: String, file: std::fs::File) -> Self {
        let mut files = FILE_ARRAY.write().unwrap();
        let index = files.len();
        files.push(Arc::new(FileStatic { name, inner: file }));
        Self { index }
    }

    //opens existing file
    pub fn open_file(name: &String) -> Result<Self, std::io::Error> {
        match OpenOptions::new().write(true).read(true).open(name) {
            Ok(file) => Ok(Self::register_file(name.clone(), file)),
            Err(e) => Err(e),
        }
    }
    //creates a file with pages
    pub fn create_file(name: &String) -> Option<Self> {
        if Path::new(name).exists() == true {
            return None;
        }
        let file = match OpenOptions::new().write(true).create_new(true).open(name) {
            Ok(val) => val,
            Err(_) => {
                return None;
            }
        };
        return Some(Self::register_file(name.clone(), file));
    }

    pub fn new_page(&mut self) -> Option<usize> {
        let file = self.get_file();
        let filelen = match file.inner.metadata() {
            Err(_) => return None,
            Ok(meta) => meta.len(),
        };
        match file.inner.set_len(filelen + PAGE_SIZE as u64) {
            Err(_) => return None,
            Ok(_) => return Some(filelen as usize / PAGE_SIZE + 1),
        }
    }

    //retuns buffer with the whole page in it
    pub fn get_page(
        &mut self,
        page: usize,
        buf: &mut [u8; PAGE_SIZE],
    ) -> Result<usize, std::io::Error> {
        let file = self.get_file();
        file.inner.read_at(buf, (page * PAGE_SIZE) as u64)
    }

    //reads page of index page from positoion offset untill end of buffer
    pub fn get_part_of_page(
        &mut self,
        page: usize,
        offset: usize,
        buf: &mut [u8],
    ) -> Result<usize, std::io::Error> {
        assert!(offset + buf.len() <= PAGE_SIZE);
        let file = self.get_file();
        file.inner
            .read_at(buf, ((page * PAGE_SIZE) + offset) as u64)
    }

    //overwrites the whole page with index page with the buf
    pub fn write_to_page(
        &mut self,
        page: usize,
        buf: &mut [u8; PAGE_SIZE],
    ) -> Result<usize, std::io::Error> {
        let file = self.get_file();
        file.inner.write_at(buf, (page * PAGE_SIZE) as u64)
    }

    //writes to the page from offset until end of buf
    pub fn write_to_part_of_page(
        &mut self,
        page: usize,
        offset: usize,
        buf: &mut [u8],
    ) -> Result<usize, std::io::Error> {
        assert!(offset + buf.len() <= PAGE_SIZE);
        let file = self.get_file();
        file.inner.write_at(buf, (page * PAGE_SIZE + offset) as u64)
    }

    pub fn delete_files(name: &String) -> Result<(), std::io::Error> {
        fs::remove_file(name)
    }
}

impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        let self_file = self.get_file();
        let other_file = other.get_file();
        self_file.inner.as_raw_fd() == other_file.inner.as_raw_fd()
    }
}
