use crate::btree::BTree;

#[test]
fn btree_insert_and_find_small_set() {
    let mut btree = BTree::<u32>::new();
    let values = [0, 1, 2, 3, 4];

    for value in values {
        btree.insert(value);
    }

    for value in values {
        assert_eq!(btree.find(value), Some(&value));
    }
}

#[test]
fn btree_insert_and_find_medium_set() {
    let mut btree = BTree::<u32>::new();

    for value in 0..11 {
        btree.insert(value);
    }

    for value in 0..11 {
        assert_eq!(btree.find(value), Some(&value));
    }
}
