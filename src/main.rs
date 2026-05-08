//use rand::prelude::*;
//use std::env;
//use std::fs::File;
//use std::fs::OpenOptions;
//use std::os::unix::fs::FileExt;
pub mod btree;
#[cfg(test)]
mod btree_tests;
use btree::BTree;

fn main() {
    //    let mut rng = rand::rng();
    let mut btree = BTree::<u32>::new();
    let mut arr: Vec<u32> = Vec::<u32>::new();
    for i in 0..11 {
        let tmp: u32 = i; // rng.random();

        arr.push(tmp);
        btree.insert(tmp);
        println!("{:?}", btree);
        println!("===============");
    }
    // println!("{:?}", btree.elements);
    // println!("{:?}", btree.pointers);
    for item in arr.iter_mut() {
        let res = btree.find(*item);
        match res {
            Some(el) => {
                if el != item {
                    eprintln!("Found incorrect number {}  != {}", item, el);
                }
            }
            None => eprintln!("Number {} not found", item),
        }
    }
}
