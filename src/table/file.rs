use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::os::unix::fs::FileExt;
use std::path::Path;

use super::PAGE;

//this implements caching
pub struct File {
    inner: std::fs::File,
}

impl File {
    //opens existing file
    pub fn open_file(name: &String) -> Result<Self, std::io::Error> {
        match OpenOptions::new().write(true).read(true).open(name) {
            Ok(file) => Ok(Self { inner: file }),
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
        return Some(File::new(file));
    }

    fn new(file: std::fs::File) -> Self {
        Self { inner: file }
    }

    pub fn new_page(&mut self) -> Option<usize> {
        let filelen = match self.inner.metadata() {
            Err(_) => return None,
            Ok(meta) => meta.len(),
        };
        match self.inner.set_len(filelen + PAGE as u64) {
            Err(_) => return None,
            Ok(_) => return Some(filelen as usize / PAGE + 1),
        }
    }

    //retuns buffer with the whole page in it
    pub fn get_page(&mut self, page: usize, buf: &mut [u8; PAGE]) -> Result<usize, std::io::Error> {
        self.inner.seek(io::SeekFrom::Start((page * PAGE) as u64))?;
        self.inner.read(buf)
    }

    //reads page of index page from positoion offset untill end of buffer
    pub fn get_part_of_page(
        &mut self,
        page: usize,
        offset: usize,
        buf: &mut [u8],
    ) -> Result<usize, std::io::Error> {
        assert!(PAGE < offset + buf.len());
        self.inner
            .seek(io::SeekFrom::Start(((page * PAGE) + offset) as u64))?;
        self.inner.read(buf)
    }

    //overwrites the whole page with index page with the buf
    pub fn write_to_page(
        &mut self,
        page: usize,
        buf: &mut [u8; PAGE],
    ) -> Result<usize, std::io::Error> {
        self.inner.seek(io::SeekFrom::Start((page * PAGE) as u64))?;
        self.inner.write(buf)
    }

    //writes to the page from offset until end of buf
    pub fn write_to_part_of_page(
        &mut self,
        page: usize,
        offset: usize,
        buf: &mut [u8],
    ) -> Result<usize, std::io::Error> {
        assert!(PAGE < offset + buf.len());
        self.inner
            .seek(io::SeekFrom::Start((page * PAGE) as u64 + offset as u64))?;
        self.inner.write(buf)
    }

    pub fn delete_files(name: &String) {
        fs::remove_file(name);
    }
}
