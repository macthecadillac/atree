//! A module that containers the core of the arena allocator
#![allow(clippy::new_without_default)]
#![allow(unused)]
use std::mem;

use crate::token::Token;

#[derive(Clone, Debug)]
pub struct Allocator<T> {
    data: Vec<Cell<T>>,
    head: Option<usize>,
    len: usize
}

#[derive(Clone, Debug)]
enum Cell<T> {
    Just(T),
    Nothing(Option<usize>)
}

impl<T> Default for Allocator<T> {
    fn default() -> Self {
        Allocator {
            data: vec![Cell::Nothing(None)],
            head: Some(0),
            len: 0
        }
    }
}

impl<T> Allocator<T> {
    pub fn head(&mut self) -> Token {
        match self.head {
            Some(head) => Token{ index: head },
            None => {
                self.reserve(self.len());
                self.head()
            }
        }
    }

    pub fn len(&self) -> usize { self.len }

    pub fn is_empty(&self) -> bool { self.len == 0 }

    pub fn capacity(&self) -> usize { self.data.len() }

    pub fn new() -> Self {
        Allocator { data: Vec::new(), head: None, len: 0 }
    }

    pub fn is_valid_token(&self, token: Token) -> bool {
        self.get(token).is_some()
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

    pub fn reserve(&mut self, additional: usize) {
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
                self.reserve(if self.len == 0 { 10 } else { self.len });
                self.insert(data)
            },
            Some(index) => {
                let next_head = match self.data.get(index) {
                    Some(Cell::Just(_)) | None => panic!("corrupt arena"),
                    Some(Cell::Nothing(next_head)) => next_head
                };
                self.head = *next_head;
                self.len += 1;
                self.data[index] = Cell::Just(data);
                Token { index }
            }
        }
    }

    pub fn set(&mut self, token: Token, data: T) -> Option<T> {
        let out = self.remove(token);
        self.insert(data);
        out
    }

    pub fn remove(&mut self, token: Token) -> Option<T> {
        match self.data.get_mut(token.index) {
            Some(Cell::Nothing(_)) | None => None,
            Some(mut cell) => {
                let mut x = Cell::Nothing(self.head);
                mem::swap(&mut x, &mut cell);
                self.head = Some(token.index);
                self.len -= 1;
                match x {
                    Cell::Just(data) => Some(data),
                    _ => panic!("something is wrong with the code")
                }
            }
        }
    }

    pub fn get(&self, token: Token) -> Option<&T> {
        match self.data.get(token.index) {
            Some(Cell::Nothing(_)) | None => None,
            Some(Cell::Just(data)) => Some(data)
        }
    }

    pub fn get_mut(&mut self, token: Token) -> Option<&mut T> {
        match self.data.get_mut(token.index) {
            Some(Cell::Nothing(_)) | None => None,
            Some(Cell::Just(data)) => Some(data)
        }
    }
}
