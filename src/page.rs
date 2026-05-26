use std::{ops::Deref, sync::RwLock};

use crate::table::{PAGE_SIZE, file::File};

const CACHE_SIZE: usize = 1000;

//index of buffer in cache and struct in cache index are conected cache index represents the chace
//buffer with the same index in array
static CACHE: [RwLock<[u8; PAGE_SIZE]>; CACHE_SIZE] =
    [const { RwLock::new([0; PAGE_SIZE]) }; CACHE_SIZE];

static CACHE_INDEX: RwLock<[Option<CacheInfo>; CACHE_SIZE]> =
    RwLock::new([const { None }; CACHE_SIZE]);

pub struct CacheInfo {
    info: Info,
    hotness: usize,
}

#[derive(Clone)]
pub struct Info {
    file: File,
    page_index: usize,
}

pub struct Page {
    file: File,
    info: Info,
}

impl Info {
    fn write_to_page(&mut self, buf: &mut [u8; PAGE_SIZE]) -> Result<usize, std::io::Error> {
        let tmp = self.page_index;
        self.file.write_to_page(tmp, buf)
    }
    fn get_page(&mut self, buf: &mut [u8; PAGE_SIZE]) -> Result<usize, std::io::Error> {
        let tmp = self.page_index;
        self.file.get_page(tmp, buf)
    }
}

impl Deref for Page {
    type Target = RwLock<[u8; PAGE_SIZE]>;
    fn deref(&self) -> &Self::Target {
        let mut cache_index = CACHE_INDEX.write().unwrap();
        {
            let pos = cache_index
                .iter_mut()
                .enumerate()
                .find_map(|(i, e)| match e {
                    Some(e) => {
                        if e.info == self.info {
                            Some((i, e))
                        } else {
                            None
                        }
                    }
                    None => None,
                });

            if let Some((index, info)) = pos {
                info.hotness += 1;
                return &CACHE[index];
            }
        }

        {
            //there is empty space in the cache
            let new = cache_index.iter_mut().enumerate().find(|c| c.1.is_none());
            if let Some((i, e)) = new {
                let mut tmp = CacheInfo {
                    info: self.info.clone(),
                    hotness: 1,
                };
                let _ = tmp.info.file.get_page(
                    self.info.page_index,
                    &mut CACHE[i]
                        .write()
                        .expect("Failed to aquire write lock for buffer alocation"),
                ); //error handle later
                CACHE_INDEX.write().expect("failed to get write lock")[i] = Some(tmp);
                return &CACHE[i];
            }
        }

        {
            let new = cache_index
                .iter_mut()
                .enumerate()
                .min_by_key(|(_, e)| e.as_ref().unwrap().hotness);

            if let Some((i, e)) = new {
                let mut buffer = CACHE[i]
                    .write()
                    .expect("Failed to aquire write lock for buffer alocation");
                let mut arr = CACHE_INDEX
                    .write()
                    .expect("Failed to aquire read lock for buffer alocation");

                match e {
                    Some(e) => {
                        let _ = e.info.write_to_page(&mut buffer);
                    }
                    None => {
                        panic!("something is fucked up");
                    }
                }
                let _ = arr[i].as_mut().unwrap().info.write_to_page(&mut buffer); //errorhandle
                //later

                let mut tmp = CacheInfo {
                    info: self.info.clone(),
                    hotness: 1,
                };
                let _ = tmp.info.file.get_page(self.info.page_index, &mut buffer); //error handle later
                return &CACHE[i];
            }
        }

        panic!("Error in deref none of the condition were met");
    }
}

impl PartialEq for Info {
    fn eq(&self, other: &Self) -> bool {
        self.file == other.file && self.page_index == other.page_index
    }
}

// fn test(page: Page) {
//
//     let mut a = page.write().unwrap();
//     a[9] = a[4];
//     println!("{:?}", a);
// }
