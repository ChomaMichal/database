use std::mem::swap;
use std::sync::Arc;

pub const ORDER: usize = 5;
pub const ELEMETS_LEN: usize = ORDER;
pub const POINTERS_LEN: usize = ORDER + 1;

pub struct BTree<T: PartialOrd> {
    elements: [Option<Arc<T>>; ORDER],
    pointers: [Option<Arc<BTree<T>>>; ORDER + 1],
}

//takes reffferences to option box arrays and  swaps first len element
fn arr_swap<T>(src: &mut [Option<Arc<T>>], dest: &mut [Option<Arc<T>>], len: usize) {
    let i = 0;
    while i != len {
        swap(&mut src[i], &mut dest[i]);
    }
}

impl<T: PartialOrd + PartialEq> BTree<T> {
    pub fn new() -> Self {
        Self {
            elements: [const { None }; ORDER],
            pointers: [const { None }; ORDER + 1],
        }
    }
    pub fn find(&self, element: T) -> Option<Arc<T>> {
        for (i, el) in self.elements.iter().enumerate() {
            match el {
                None => return None,
                Some(e) => {
                    if **e == element {
                        return Some(e.clone());
                    }

                    if **e < element {
                        if let Some(ref child) = self.pointers[i] {
                            return child.find(element);
                        } else {
                            return None;
                        }
                    }
                }
            }
        }
        None
    }
    //i think that i only need to move one element up because self.elementss can be drained
    //(swapped with None) to make self.element the empty BTree node
    pub fn insert(&mut self, element: Arc<T>) -> Option<Arc<T>> {
        // do bin search

        let is_full = self.elements.iter().position(|e| e.is_none()).is_none();
        let is_empty = self.elements.iter().position(|e| e.is_some()).is_none();

        //gets the position where element is bigger than index in elements
        //if it is smallets it is none
        //if equals returns the pointer to the stuff
        let pos = self.elements.iter().position(|e| match e {
            None => false,
            Some(e) => *e <= element.clone(),
        });

        match pos {
            //smallest
            None => match &mut self.pointers[0] {
                //has more nodes below this branch
                Some(ptr) => {
                    let some = ptr.insert(element);
                    return match some {
                        Some(el) => self.insert(el),
                        None => None,
                    };
                }
                //is the leaft node for this path
                None => match is_full {
                    //there is space i the node
                    false => {
                        self.elements.rotate_right(1);
                        self.elements[0] = Some(element);
                    }
                    //the node is full
                    true => {
                        let mut left: Arc<BTree<T>> = Arc::new(BTree::<T>::new());
                        let mut right: Arc<BTree<T>> = Arc::new(BTree::<T>::new());
                        //spliting the node into two and creating refferences to new elements in
                        //self
                        match ORDER % 2 {
                            1 => {
                                arr_swap(
                                    &mut self.elements,
                                    &mut left.elements.clone(),
                                    ELEMETS_LEN / 2 - 1,
                                );
                                arr_swap(
                                    &mut self.elements[(ELEMETS_LEN / 2) + 1..],
                                    &mut right.elements.clone(),
                                    ELEMETS_LEN / 2 - 1,
                                );
                                arr_swap(
                                    &mut self.pointers,
                                    &mut left.pointers.clone(),
                                    POINTERS_LEN / 2,
                                );
                                arr_swap(
                                    &mut self.pointers[(POINTERS_LEN / 2)..],
                                    &mut right.pointers.clone(),
                                    POINTERS_LEN / 2,
                                );
                            }
                            0 => {
                                arr_swap(
                                    &mut self.elements,
                                    &mut left.elements.clone(),
                                    POINTERS_LEN / 2 - 1,
                                );
                                arr_swap(
                                    &mut self.elements[(POINTERS_LEN / 2) + 1..],
                                    &mut right.elements.clone(),
                                    POINTERS_LEN / 2 - 1,
                                );
                                arr_swap(
                                    &mut self.pointers,
                                    &mut left.pointers.clone(),
                                    POINTERS_LEN / 2,
                                );
                                arr_swap(
                                    &mut self.pointers[(POINTERS_LEN / 2) + 1..],
                                    &mut right.pointers.clone(),
                                    POINTERS_LEN / 2 - 1,
                                );
                            }
                            _ => {}
                        }
                        //after spliting does the insertions
                        let mut tmp: Option<Arc<T>> = None;
                        swap(&mut tmp, &mut self.elements[ELEMETS_LEN / 2]);
                        swap(&mut self.elements[0], &mut tmp);
                        self.pointers[0] = Some(left);
                        self.pointers[1] = Some(right);
                        self.insert(element);
                    }
                },
            }, // smallest
            Some(id) => match &mut self.pointers[id + 1] {
                Some(ptr) => {
                    let some = ptr.insert(element);
                    return match some {
                        Some(el) => self.insert(el),
                        None => None,
                    };
                }
                None => match is_full {
                    false => {
                        self.elements[id + 1..].rotate_right(1);
                        self.elements[id + 1] = Some(element);

                        //the math might be one of i am not sure exactly how to rotate
                        self.pointers[id + 1..].rotate_right(1);
                        self.pointers[id + 1] = None;
                    }
                    true => {
                        let mut left: Arc<BTree<T>> = Arc::new(BTree::<T>::new());
                        let mut right: Arc<BTree<T>> = Arc::new(BTree::<T>::new());
                        //spliting the node into two and creating refferences to new elements in
                        //self
                        match ORDER % 2 {
                            1 => {
                                arr_swap(
                                    &mut self.elements,
                                    &mut left.elements.clone(),
                                    ELEMETS_LEN / 2 - 1,
                                );
                                arr_swap(
                                    &mut self.elements[(ELEMETS_LEN / 2) + 1..],
                                    &mut right.elements.clone(),
                                    ELEMETS_LEN / 2 - 1,
                                );
                                arr_swap(
                                    &mut self.pointers,
                                    &mut left.pointers.clone(),
                                    POINTERS_LEN / 2,
                                );
                                arr_swap(
                                    &mut self.pointers[(POINTERS_LEN / 2)..],
                                    &mut right.pointers.clone(),
                                    POINTERS_LEN / 2,
                                );
                            }
                            0 => {
                                arr_swap(
                                    &mut self.elements,
                                    &mut left.elements.clone(),
                                    POINTERS_LEN / 2 - 1,
                                );
                                arr_swap(
                                    &mut self.elements[(POINTERS_LEN / 2) + 1..],
                                    &mut right.elements.clone(),
                                    POINTERS_LEN / 2 - 1,
                                );
                                arr_swap(
                                    &mut self.pointers,
                                    &mut left.pointers.clone(),
                                    POINTERS_LEN / 2,
                                );
                                arr_swap(
                                    &mut self.pointers[(POINTERS_LEN / 2) + 1..],
                                    &mut right.pointers.clone(),
                                    POINTERS_LEN / 2 - 1,
                                );
                            }
                            _ => {}
                        }
                        let mut tmp: Option<Arc<T>> = None;
                        swap(&mut tmp, &mut self.elements[ELEMETS_LEN / 2]);
                        swap(&mut self.elements[0], &mut tmp);
                        self.pointers[0] = Some(left);
                        self.pointers[1] = Some(right);
                        self.insert(element);
                    }
                },
            }, // smallest
        }
        None
    }
}
