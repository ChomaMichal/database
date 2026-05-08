use std::fmt::{self, Display, Formatter};
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

//takes reffreferences to option box arrays and swaps first len element
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

impl<T> BTree<T>
where
    T: PartialOrd + PartialEq + Display + std::fmt::Debug,
{
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
                return;
            }
            None => return,
        }
    }
    fn _len(&self) -> usize {
        let len = self.elements.iter().position(|e| e == &None);
        match len {
            Some(l) => l,
            None => ELEMENTS_LEN,
        }
    }

    pub fn find(&self, element: T) -> Option<&T> {
        let len = self._len();
        if len == 0 {
            return None;
        }
        let mut top_pos = len - 1;
        let mut bot_pos = 0;
        let mut prev = ORDER + 1;
        loop {
            let pos = bot_pos + (top_pos - bot_pos) / 2;
            if **self.elements[pos].as_ref().unwrap() == element {
                return Some(self.elements[pos].as_ref().unwrap());
            }
            if bot_pos == top_pos {
                if **self.elements[len - 1].as_ref().unwrap() < element {
                    top_pos += 1;
                }
                match self.pointers[top_pos].as_ref() {
                    Some(pointer) => return pointer.find(element),
                    None => return None,
                }
            }

            if **self.elements[pos].as_ref().unwrap() < element {
                if bot_pos == 0 && top_pos == 1 {
                    bot_pos = 1;
                } else {
                    bot_pos = pos;
                }
            } else if **self.elements[pos].as_ref().unwrap() > element {
                top_pos = pos;
            }
            if pos == prev {
                if pos == bot_pos {
                    bot_pos += 1;
                }
                if pos == top_pos {
                    top_pos -= 1;
                }
            }
            prev = pos;
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
                arr_swap(&mut self.pointers, &mut left.pointers, MIDLE + 1);
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
        OverFlow::new(tmp, Some(left), Some(right))
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
                        match self.pointers[i].as_mut() {
                            None => {
                                let tmp = self._is_leaf_full();
                                match tmp {
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
                            Some(ptr) => {
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
        match self.pointers[POINTERS_LEN - 1].as_mut() {
            Some(el) => match el._insert(element) {
                Some(extra) => match self._is_leaf_full() {
                    true => {
                        let mut n_extra = self._split_node();
                        let index = extra.right.as_ref().unwrap()._len() - 1;
                        n_extra
                            .right
                            .as_mut()
                            .unwrap()
                            .as_mut()
                            ._insert_overflow_at_index(index, extra);
                        return Some(n_extra);
                    }
                    false => {
                        let index = extra.right.as_ref().unwrap()._len() - 1;
                        self._insert_overflow_at_index(index, extra);
                        return None;
                    }
                },
                None => return None,
            },
            None => {
                let mut tmp = self._split_node();
                tmp.right.as_mut().unwrap()._insert(element);
                return Some(tmp);
            }
        }
    }

    //check if the node is full before calling
    fn _insert_overflow_at_index(&mut self, i: usize, mut extra: OverFlow<T>) {
        self.elements[i..].rotate_right(1);
        swap(&mut self.elements[i], &mut extra.element);
        swap(&mut self.pointers[i], &mut extra.right);
        self.pointers[i..].rotate_right(1);
        swap(&mut self.pointers[i], &mut extra.left);
    }

    fn fmt_pretty_with_indent(&self, depth: usize, out: &mut String) {
        let indent = "  ".repeat(depth);

        out.push_str(&format!("{indent}BTreeNode {{\n"));

        out.push_str(&format!("{indent}  elements: ["));
        let mut first = true;
        for e in &self.elements {
            if !first {
                out.push_str(", ");
            }
            first = false;
            match e {
                Some(v) => out.push_str(&format!("{v}")),
                None => out.push_str("_"),
            }
        }
        out.push_str("]\n");

        for (i, ptr) in self.pointers.iter().enumerate() {
            if let Some(child) = ptr.as_ref() {
                out.push_str(&format!("{indent}  child[{i}]:\n"));
                child.fmt_pretty_with_indent(depth + 1, out);
            }
        }

        out.push_str(&format!("{indent}}}\n"));
    }
}

impl<T> Display for BTree<T>
where
    T: PartialOrd + PartialEq + Display + std::fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        self.fmt_pretty_with_indent(0, &mut out);
        write!(f, "{out}")
    }
}

