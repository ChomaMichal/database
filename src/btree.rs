use std::mem::swap;

pub const ORDER: usize = 5;
pub const LEN_ELEMETS: usize = ORDER;
pub const LEN_POINTERS: usize = ORDER + 1;

pub struct BTree<T: PartialOrd> {
    elements: [Option<Box<T>>; ORDER],
    pointers: [Option<Box<BTree<T>>>; ORDER + 1],
}

//takes reffferences to option box arrays and  swaps first len element
fn arr_swap<T>(src: &mut [Option<Box<T>>], dest: &mut [Option<Box<T>>], len: usize) {
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
    //i think that i only need to move one element up because self.elementss can be drained
    //(swapped with None) to make self.element the empty BTree node
    pub fn insert(&mut self, element: Box<T>) -> Option<Box<T>> {
        // do bin search

        let is_full = self.elements.iter().position(|e| e.is_none()).is_none();
        let is_empty = self.elements.iter().position(|e| e.is_some()).is_none();

        let pos = self.elements.iter().position(|e| match e {
            None => false,
            Some(e) => *e < element,
        });

        match pos {
            None => match &mut self.pointers[0] {
                Some(ptr) => {
                    let some = ptr.insert(element);
                }
                None => match is_full {
                    false => {
                        self.elements.rotate_right(1);
                        self.elements[0] = Some(element);
                    }
                    true => {
                        let mut left: BTree<T> = BTree::<T>::new();
                        let mut right: BTree<T> = BTree::<T>::new();
                        match ORDER % 2 {
                            1 => {
                                arr_swap(&mut self.elements, &mut left.elements, ORDER / 2 - 1);
                                arr_swap(
                                    &mut self.elements[(ORDER / 2) + 1..],
                                    &mut right.elements,
                                    ORDER / 2 - 1,
                                );
                                arr_swap(&mut self.pointers, &mut left.pointers, ORDER / 2);
                                arr_swap(
                                    &mut self.pointers[(ORDER / 2)..],
                                    &mut right.pointers,
                                    ORDER / 2,
                                );
                            }
                            0 => {
                                arr_swap(&mut self.elements, &mut left.elements, ORDER / 2 - 1);
                                arr_swap(
                                    &mut self.elements[(ORDER / 2) + 1..],
                                    &mut right.elements,
                                    ORDER / 2,
                                );
                            }
                            _ => {}
                        }
                    }
                },
            }, // smallest
            Some(id) => {}
        }
        None
    }
}
