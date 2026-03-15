//! A module that containers the core of the arena allocator
#![allow(clippy::new_without_default)]
#![allow(unused)]
use std::mem;
use std::num::NonZeroUsize;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

use crate::token::Token;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Allocator<T> {
    data: Vec<Cell<T>>,
    head: Option<NonZeroUsize>,
    len: usize
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
                mem::swap(&mut x, cell);
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn default_creates_valid_allocator() {
        let alloc: Allocator<usize> = Allocator::default();
        assert_eq!(alloc.len(), 0);
        assert!(alloc.is_empty());
        assert!(alloc.capacity() > 0);
    }

    #[test]
    fn capacity_returns_correct_value() {
        let alloc: Allocator<usize> = Allocator::new();
        assert_eq!(alloc.capacity(), alloc.data.len());
    }

    #[test]
    fn is_valid_token_true_and_false() {
        let mut alloc: Allocator<usize> = Allocator::new();
        let token = alloc.insert(42);
        assert!(alloc.is_valid_token(token));

        alloc.remove(token);
        assert!(!alloc.is_valid_token(token));
    }

    #[test]
    fn get_returns_none_for_free_slot() {
        let mut alloc: Allocator<usize> = Allocator::new();
        let token = alloc.insert(99);
        alloc.remove(token);
        assert!(alloc.get(token).is_none());
    }

    #[test]
    fn get_mut_returns_none_for_free_slot() {
        let mut alloc: Allocator<usize> = Allocator::new();
        let token = alloc.insert(99);
        alloc.remove(token);
        assert!(alloc.get_mut(token).is_none());
    }

    #[test]
    fn reserve_extends_capacity() {
        let mut alloc: Allocator<usize> = Allocator::new();
        let initial_capacity = alloc.capacity();
        alloc.reserve(8);
        assert!(alloc.capacity() >= initial_capacity + 8);
    }

    #[test]
    fn insert_after_exhaustion_triggers_reserve() {
        let mut alloc: Allocator<usize> = Allocator::new();
        // Fill until the allocator must grow
        let initial_capacity = alloc.capacity();
        for i in 0..initial_capacity {
            alloc.insert(i);
        }
        // This insert should trigger reserve internally
        let token = alloc.insert(999);
        assert!(alloc.is_valid_token(token));
        assert_eq!(*alloc.get(token).unwrap(), 999);
    }

    #[test]
    fn head_when_full_triggers_reserve() {
        let mut alloc: Allocator<usize> = Allocator::new();
        let capacity = alloc.capacity();
        for i in 0..capacity {
            alloc.insert(i);
        }
        // head() when allocator is full should call reserve and return a valid token
        let token = alloc.head();
        assert!(alloc.capacity() > capacity);
        // head token should be a free slot pointing to a Nothing cell
        assert!(alloc.get(token).is_none());
    }
}
