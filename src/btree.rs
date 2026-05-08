use std::mem::swap;

const ORDER: usize = 5;
const ELEMENTS_LEN: usize = ORDER;
const POINTERS_LEN: usize = ORDER + 1;
const MIDLE: usize = ORDER / 2;

// t has to be key value pair for this to make sence
#[derive(Debug)]
pub struct BTree<T: PartialOrd> {
    pub elements: [Option<Box<T>>; ORDER],
    pub pointers: [Option<Box<BTree<T>>>; ORDER + 1],
}

struct OverFlow<T: PartialOrd> {
    element: Option<Box<T>>,
    left: Option<Box<BTree<T>>>,
    right: Option<Box<BTree<T>>>,
}

impl<T: PartialOrd + PartialEq + std::fmt::Display + std::fmt::Debug> OverFlow<T> {
    fn new(
        element: Option<Box<T>>,
        left: Option<Box<BTree<T>>>,
        right: Option<Box<BTree<T>>>,
    ) -> Self {
        Self {
            element: element,
            left: left,
            right: right,
        }
    }
}

//takes reffferences to option box arrays and  swaps first len element
fn arr_swap<T>(src: &mut [T], dest: &mut [T], len: usize) {
    for i in 0..len {
        swap(&mut src[i], &mut dest[i]);
    }
}

pub fn test_arr_swap() {
    let mut array: Vec<usize> = (1..11).collect();
    let mut left: Vec<usize> = vec![0; 10];
    for el in left.iter_mut() {
        *el = 0;
    }
    let mut right: Vec<usize> = vec![0; 10];
    for el in right.iter_mut() {
        *el = 0;
    }

    arr_swap(&mut array, &mut left, 10 / 2);
    arr_swap(&mut array[((10 / 2) + 1)..], &mut right, (10 / 2) - 1);

    let mut exp_left: Vec<usize> = vec![0; 10];
    for i in 0..5 {
        exp_left[i] = i + 1;
    }
    let mut exp_right: Vec<usize> = vec![0; 10];
    for i in 0..4 {
        exp_right[i] = i + 7;
    }
    assert_eq!(left, exp_left);
    assert_eq!(right, exp_right);
    let mut array: Vec<usize> = (1..12).collect();
    let mut left: Vec<usize> = vec![0; 11];
    for el in left.iter_mut() {
        *el = 0;
    }
    let mut right: Vec<usize> = vec![0; 11];
    for el in right.iter_mut() {
        *el = 0;
    }

    arr_swap(&mut array, &mut left, 11 / 2);
    arr_swap(&mut array[((11 / 2) + 1)..], &mut right, 11 / 2);

    let mut exp_left: Vec<usize> = vec![0; 11];
    for i in 0..5 {
        exp_left[i] = i + 1;
    }
    let mut exp_right: Vec<usize> = vec![0; 11];
    for i in 0..5 {
        exp_right[i] = i + 7;
    }
    assert_eq!(left, exp_left);
    assert_eq!(right, exp_right);
}

impl<T: PartialOrd + PartialEq + std::fmt::Display + std::fmt::Debug> BTree<T> {
    pub fn new() -> Self {
        Self {
            elements: [const { None }; ORDER],
            pointers: [const { None }; ORDER + 1],
        }
    }

    pub fn insert(&mut self, element: T) {
        let tmp = self._insert(element);
        match tmp {
            Some(tmp) => {
                let mut btree: BTree<T> = BTree::<T>::new();
                btree.elements[0] = tmp.element;
                btree.pointers[0] = tmp.left;
                btree.pointers[1] = tmp.right;
                swap(self, &mut btree);
                // println!("{:?} == top", self);
                // println!("=====================================");
                return;
            }
            None => return,
        }
    }

    pub fn find(&self, element: T) -> Option<&T> {
        let len = self.elements.iter().position(|e| e == &None);
        let len = match len {
            Some(l) => l,
            None => ELEMENTS_LEN,
        };
        let mut top_pos = len;
        let mut bot_pos = 0;

        loop {
            let pos = bot_pos + (top_pos - bot_pos) / 2;
            if pos == bot_pos && pos == top_pos {
                match self.pointers[pos].as_ref() {
                    Some(child) => {
                        return child.find(element);
                    }
                    None => {
                        if **self.elements[pos].as_ref().unwrap() == element {
                            return Some(self.elements[pos].as_ref().unwrap());
                        }
                        return None;
                    }
                }
            } else if **self.elements[pos].as_ref().unwrap() < element {
                top_pos = top_pos - (top_pos - bot_pos) / 2;
            } else if **self.elements[pos].as_ref().unwrap() > element {
                bot_pos = bot_pos + (top_pos - bot_pos) / 2;
            } else if **self.elements[pos].as_ref().unwrap() == element {
                return Some(self.elements[pos].as_ref().unwrap());
            }
        }
    }

    fn _is_leaf_full(&self) -> bool {
        match self.elements[ELEMENTS_LEN - 1] {
            None => return false,
            Some(_) => return true,
        }
    }

    fn _split_node(&mut self) -> OverFlow<T> {
        let mut left: Box<BTree<T>> = Box::new(BTree::<T>::new());
        let mut right: Box<BTree<T>> = Box::new(BTree::<T>::new());
        match ORDER % 2 {
            1 => {
                arr_swap(&mut self.elements, &mut left.elements, MIDLE);
                arr_swap(&mut self.elements[MIDLE + 1..], &mut right.elements, MIDLE);
                arr_swap(&mut self.pointers, &mut left.pointers, MIDLE);
                arr_swap(&mut self.pointers[MIDLE + 1..], &mut right.pointers, MIDLE);
            }
            0 => {
                arr_swap(&mut self.elements, &mut left.elements, MIDLE - 1);
                arr_swap(
                    &mut self.elements[MIDLE + 1..],
                    &mut right.elements,
                    ELEMENTS_LEN - MIDLE - 1,
                );
                arr_swap(&mut self.pointers, &mut left.pointers, MIDLE);
                arr_swap(
                    &mut self.pointers[(MIDLE) + 1..],
                    &mut right.pointers,
                    POINTERS_LEN - MIDLE - 1,
                );
            }
            _ => {}
        }
        let mut tmp = None;
        swap(&mut self.elements[MIDLE], &mut tmp);
        return OverFlow::new(tmp, Some(left), Some(right));
    }

    fn _insert(&mut self, element: T) -> Option<OverFlow<T>> {
        for i in 0..ELEMENTS_LEN {
            match self.elements[i].as_mut() {
                None => match self.pointers[i].as_mut() {
                    Some(el) => match el._insert(element) {
                        Some(extra) => match self._is_leaf_full() {
                            true => {
                                let mut n_extra = self._split_node();
                                n_extra
                                    .right
                                    .as_mut()
                                    .unwrap()
                                    ._insert_overflow_at_index(i, extra);
                                return Some(n_extra);
                            }
                            false => {
                                self._insert_overflow_at_index(i, extra);
                                return None;
                            }
                        },
                        None => return None,
                    },

                    None => {
                        self.elements[i] = Some(Box::new(element));
                        return None;
                    }
                },
                Some(el) => {
                    if **el == element {
                        println!("Duplicates");
                    } else if element < **el {
                        println!("enters 186");
                        match self.pointers[i].as_mut() {
                            //if it has no children
                            None => {
                                let tmp = self._is_leaf_full();
                                match tmp {
                                    //do the splitting here
                                    true => {
                                        return Some(self._split_node());
                                    }

                                    false => {
                                        self.elements[i..].rotate_right(1);
                                        self.elements[i] = Some(Box::new(element));
                                        self.pointers[i..].rotate_right(1);
                                        return None;
                                    }
                                }
                            }
                            //if it has children
                            Some(ptr) => {
                                print!("Inserting to children");
                                match ptr._insert(element) {
                                    Some(extra) => match self._is_leaf_full() {
                                        false => {
                                            self._insert_overflow_at_index(i, extra);
                                        }
                                        true => {}
                                    },
                                    None => return None,
                                };

                                return None;
                            }
                        }
                    }
                }
            };
        }
        let mut tmp = self._split_node();
        tmp.right.as_mut().unwrap()._insert(element);
        return Some(tmp);
    }

    //check if the node is full before calling
    fn _insert_overflow_at_index(&mut self, i: usize, mut extra: OverFlow<T>) {
        self.elements[i..].rotate_right(1);
        swap(&mut self.elements[i], &mut extra.element);
        swap(&mut self.pointers[i], &mut extra.right);
        self.pointers[i..].rotate_right(1);
        swap(&mut self.pointers[i], &mut extra.left);
    }
}

#[cfg(test)]
mod tests {
    use super::BTree;

    fn count_elements(tree: &BTree<u32>) -> usize {
        let current = tree.elements.iter().filter(|e| e.is_some()).count();
        current
            + tree
                .pointers
                .iter()
                .filter_map(|p| p.as_deref())
                .map(count_elements)
                .sum::<usize>()
    }

    #[test]
    fn insertion_stores_all_unique_values() {
        let mut tree = BTree::<u32>::new();
        let input = [8, 3, 10, 1, 6, 14, 4, 7, 13];

        for value in input {
            tree.insert(value);
        }

        assert_eq!(count_elements(&tree), input.len());
    }

    #[test]
    fn find_returns_inserted_values() {
        let mut tree = BTree::<u32>::new();
        let inserted = [7];

        for value in inserted {
            tree.insert(value);
        }

        for value in inserted {
            assert_eq!(tree.find(value), Some(&value));
        }
    }
}
