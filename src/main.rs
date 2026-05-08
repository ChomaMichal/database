//use rand::prelude::*;
//use std::env;
//use std::fs::File;
//use std::fs::OpenOptions;
//use std::os::unix::fs::FileExt;
pub mod btree;
#[cfg(test)]
mod btree_tests;
mod file_system;
use btree::BTree;
use rand::RngExt;
pub mod table;

fn main() {
    let mut rng = rand::rng();
    let mut btree = BTree::<u32>::new();
    let mut arr: Vec<u32> = Vec::<u32>::new();
    for i in 0..100000 {
        let tmp: u32 = rng.random();

        arr.push(tmp);
        btree.insert(tmp);
        // println!("==========element{i}==============");
        // println!("{}", btree);
        // println!("{:?}", btree);
        // println!("===============");
    }
    println!("{}", btree);
    // let tmp = btree.find(6);
    // println!("return {:?}", tmp);
    // println!("{:?}", btree.elements);
    // println!("{:?}", btree.pointers);
    for item in arr.iter_mut() {
        let res = btree.find(*item);
        match res {
            Some(el) => {
                if el != item {
                    println!("Found incorrect number {}  != {} !!", item, el);
                    panic!();
                }
                println!("Found the numbner {el}");
            }
            None => {
                println!("Number {} not found !!", item);
                panic!()
            }
        }
    }
}
