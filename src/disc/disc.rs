use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::os::unix::fs::FileExt;
use std::path::Path;

use super::PAGE;

pub struct Disc {
    file: File,
}

impl Disc {
    //opens existing file
    pub fn open_disc(name: &String) -> Result<Self, std::io::Error> {
        match OpenOptions::new().write(true).read(true).open(name) {
            Ok(file) => Ok(Self { file: file }),
            Err(e) => Err(e),
        }
    }
    //creates a file with pages
    pub fn create_disc(name: &String) -> Option<Self> {
        if Path::new(name).exists() == true {
            return None;
        }
        let file = match OpenOptions::new().write(true).create_new(true).open(name) {
            Ok(val) => val,
            Err(_) => {
                return None;
            }
        };
        return Some(Disc::new(file));
    }

    fn new(file: File) -> Self {
        Self { file: file }
    }

    pub fn new_page(&mut self) -> Option<usize> {
        let filelen = match self.file.metadata() {
            Err(_) => return None,
            Ok(meta) => meta.len(),
        };
        match self.file.set_len(filelen + PAGE as u64) {
            Err(_) => return None,
            Ok(_) => return Some(filelen as usize / PAGE + 1),
        }
    }

    //retuns buffer with the whole page in it
    pub fn get_page(&mut self, page: usize, buf: &mut [u8; PAGE]) -> Result<usize, std::io::Error> {
        self.file.seek(io::SeekFrom::Start((page * PAGE) as u64))?;
        self.file.read(buf)
    }

    //reads page of index page from positoion offset untill end of buffer
    pub fn get_part_of_page(
        &mut self,
        page: usize,
        offset: usize,
        buf: &mut [u8],
    ) -> Result<usize, std::io::Error> {
        assert!(PAGE < offset + buf.len());
        self.file
            .seek(io::SeekFrom::Start(((page * PAGE) + offset) as u64))?;
        self.file.read(buf)
    }

    //overwrites the whole page with index page with the buf
    pub fn write_to_page(
        &mut self,
        page: usize,
        buf: &mut [u8; PAGE],
    ) -> Result<usize, std::io::Error> {
        self.file.seek(io::SeekFrom::Start((page * PAGE) as u64))?;
        self.file.write(buf)
    }

    //writes to the page from offset until end of buf
    pub fn write_to_part_of_page(
        &mut self,
        page: usize,
        offset: usize,
        buf: &mut [u8],
    ) -> Result<usize, std::io::Error> {
        assert!(PAGE < offset + buf.len());
        self.file
            .seek(io::SeekFrom::Start((page * PAGE) as u64 + offset as u64))?;
        self.file.write(buf)
    }

    pub fn delete_discs(name: &String) {
        fs::remove_file(name);
    }
}
