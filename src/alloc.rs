//! A module that containers the core of the arena allocator
#![allow(clippy::new_without_default)]
#![allow(unused)]
use std::mem;
use std::num::NonZeroUsize;

use crate::token::Token;

#[derive(Clone, Debug)]
pub struct Allocator<T> {
    data: Vec<Cell<T>>,
    head: Option<NonZeroUsize>,
    len: usize
}

#[derive(Clone, Debug)]
enum Cell<T> {
    Just(T),
    Nothing(Option<NonZeroUsize>)
}

impl<T> Default for Allocator<T> {
    fn default() -> Self {
        Allocator {
            data: vec![Cell::Nothing(None)],
            head: Some(NonZeroUsize::new(1).unwrap()),
            len: 0
        }
    }
}

impl<T> Allocator<T> {
    pub fn new() -> Self {
        Allocator {
            data: vec![Cell::Nothing(None)],
            head: Some(NonZeroUsize::new(1).unwrap()),
            len: 0
        }
    }

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

    pub fn is_valid_token(&self, token: Token) -> bool {
        self.get(token).is_some()
    }

    fn find_last_available(&self) -> Option<NonZeroUsize> {
        fn aux<T>(data: &[Cell<T>], indx: NonZeroUsize) -> Option<NonZeroUsize> {
            match data.get(indx.get() - 1) {  // get back to zero-based indexing
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
        let head_indx = NonZeroUsize::new(self.data.len() + 1).unwrap();
        match self.find_last_available() {
            None => self.head = Some(head_indx),
            Some(n) => self.data[n.get() - 1] = Cell::Nothing(Some(head_indx)),
        };
        let new_cells = (head_indx.get()..)  // already bigger by 1
            .take(additional - 1)
            .map(|i| Cell::Nothing(Some(NonZeroUsize::new(i + 1).unwrap())))
            .chain(std::iter::once(Cell::Nothing(None)));
        self.data.extend(new_cells);
    }

    pub fn insert(&mut self, data: T) -> Token {
        match self.head {
            None => {
                self.reserve(self.capacity());
                self.insert(data)
            },
            Some(index) => {
                let i = index.get() - 1;  // zero-based index
                let next_head = match self.data.get(i) {
                    Some(Cell::Just(_)) | None => panic!("corrupt arena"),
                    Some(Cell::Nothing(next_head)) => next_head
                };
                self.head = *next_head;
                self.len += 1;
                self.data[i] = Cell::Just(data);
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
        match self.data.get_mut(token.index.get() - 1) {  // zero-based index
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
        match self.data.get(token.index.get() - 1) {  // zero-based index
            Some(Cell::Nothing(_)) | None => None,
            Some(Cell::Just(data)) => Some(data)
        }
    }

    pub fn get_mut(&mut self, token: Token) -> Option<&mut T> {
        match self.data.get_mut(token.index.get() - 1) {  // zero-based index
            Some(Cell::Nothing(_)) | None => None,
            Some(Cell::Just(data)) => Some(data)
        }
    }
}
