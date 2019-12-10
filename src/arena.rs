#![allow(clippy::new_without_default)]
use std::mem;

use crate::token::Token;

#[derive(Default)]
pub struct Arena<T> {
    data: Vec<Cell<T>>,
    head: Option<usize>,
    len: usize
}

enum Cell<T> {
    Just(T),
    Nothing(Option<usize>)
}

impl<T> Arena<T> {
    pub fn len(&self) -> usize { self.len }

    pub fn is_empty(&self) -> bool { self.len == 0 }

    pub fn capacity(&self) -> usize { self.data.len() }

    pub fn new() -> Self {
        Arena { data: Vec::new(), head: None, len: 0 }
    }

    fn find_last_available(&self) -> Option<usize> {
        fn aux<T>(data: &[Cell<T>], indx: usize) -> Option<usize> {
            match data.get(indx) {
                Some(Cell::Just(_)) | None => panic!("corrpt arena"),
                Some(Cell::Nothing(next_head)) => match next_head {
                    Some(n) => aux(data, *n),
                    None => Some(indx)
                }
            }
        }
        match self.head {
            None => None,
            Some(head) => aux(&self.data[..], head) // walk the heap til the end
        }
    }

    fn allocate(&mut self, additional: usize) {
        self.data.reserve_exact(additional);
        let first_new_cell_indx = self.data.len();
        match self.find_last_available() {
            Some(n) => self.data[n] = Cell::Nothing(Some(first_new_cell_indx)),
            None => self.head = Some(first_new_cell_indx)
        };
        for i in (first_new_cell_indx + 1..).take(additional - 1) {
            self.data.push(Cell::Nothing(Some(i)));
        }
        self.data.push(Cell::Nothing(None));
    }

    pub fn insert(&mut self, data: T) -> Token {
        match self.head {
            None => {
                // TODO: thik of a better way to do this
                self.allocate(if self.len == 0 { 10 } else { self.len });
                self.insert(data)
            },
            Some(indx) => {
                let next_head = match self.data.get(indx) {
                    Some(Cell::Just(_)) | None => panic!("corrupt arena"),
                    Some(Cell::Nothing(next_head)) => next_head
                };
                self.head = *next_head;
                self.len += 1;
                self.data[indx] = Cell::Just(data);
                Token(indx)
            }
        }
    }

    pub fn set(&mut self, token: Token, data: T) -> Option<T> {
        let out = self.remove(token);
        self.insert(data);
        out
    }

    pub fn remove(&mut self, token: Token) -> Option<T> {
        match self.data.get_mut(token.0) {
            Some(Cell::Nothing(_)) | None => None,
            Some(mut cell) => {
                let mut x = Cell::Nothing(self.head);
                mem::swap(&mut x, &mut cell);
                self.head = Some(token.0);
                self.len -= 1;
                match x {
                    Cell::Just(data) => Some(data),
                    _ => panic!("something is wrong with the code")
                }
            }
        }
    }

    pub fn get(&self, token: Token) -> Option<&T> {
        match self.data.get(token.0) {
            Some(Cell::Nothing(_)) | None => None,
            Some(Cell::Just(data)) => Some(data)
        }
    }

    pub fn get_mut(&mut self, token: Token) -> Option<&mut T> {
        match self.data.get_mut(token.0) {
            Some(Cell::Nothing(_)) | None => None,
            Some(Cell::Just(data)) => Some(data)
        }
    }
}
