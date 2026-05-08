use crate::btree::BTree;

fn assert_all_present(tree: &BTree<u32>, values: &[u32]) {
    for value in values {
        assert_eq!(tree.find(*value), Some(value));
    }
}

#[test]
fn btree_insert_and_find_small_set() {
    let mut btree = BTree::<u32>::new();
    let values = [0, 1, 2, 3, 4];

    for value in values {
        btree.insert(value);
    }

    assert_all_present(&btree, &values);
}

#[test]
fn btree_insert_and_find_medium_set() {
    let mut btree = BTree::<u32>::new();

    for value in 0..11 {
        btree.insert(value);
    }

    let expected: Vec<u32> = (0..11).collect();
    assert_all_present(&btree, &expected);
}

#[test]
fn btree_insert_descending_and_find_all() {
    let mut btree = BTree::<u32>::new();

    for value in (0..11).rev() {
        btree.insert(value);
    }

    let expected: Vec<u32> = (0..11).collect();
    assert_all_present(&btree, &expected);
}

#[test]
fn btree_insert_non_sequential_values_and_find_all() {
    let mut btree = BTree::<u32>::new();
    let values = [8, 3, 10, 1, 6, 14, 4, 7, 13, 2, 5];

    for value in values {
        btree.insert(value);
    }

    assert_all_present(&btree, &values);
}

#[test]
fn btree_insert_with_duplicates_and_find_unique_values() {
    let mut btree = BTree::<u32>::new();
    let inserted = [4, 2, 6, 2, 4, 6, 1, 3, 5, 7];
    let unique = [1, 2, 3, 4, 5, 6, 7];

    for value in inserted {
        btree.insert(value);
    }

    assert_all_present(&btree, &unique);
}
