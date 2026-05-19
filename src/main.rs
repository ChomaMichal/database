//use rand::prelude::*;
//use std::env;
//use std::fs::File;
//use std::fs::OpenOptions;
//
//use std::os::unix::fs::FileExt;
//

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng, rng};
pub mod btree;
#[cfg(test)]
mod btree_tests;
use btree::BTree;
pub mod table;

fn main() {}
