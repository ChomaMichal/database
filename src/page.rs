use std::{ops::Deref, sync::RwLock};

use crate::table::{PAGE_SIZE, file::File};

const CACHE_SIZE: usize = 1000;

static CACHE: [RwLock<[u8; PAGE_SIZE]>; CACHE_SIZE] =
    [const { RwLock::new([0; PAGE_SIZE]) }; CACHE_SIZE];

enum State {
    Uncached,
    Cached(usize),
}

pub struct Page {
    test: i32,
    file: File,
    file_index: u64,
    cache_index: State,
}

impl Deref for Page {
    type Target = RwLock<[u8; PAGE_SIZE]>;
    fn deref(&self) -> &Self::Target {
        &CACHE[0]
    }
}

// fn test(page: Page) {
//     let mut a = page.write().unwrap();
//     a[9] = a[4];
//     println!("{:?}", a);
// }
