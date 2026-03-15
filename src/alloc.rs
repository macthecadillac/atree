//! Core arena allocator used to back the tree structure.
//!
//! The [`Allocator`] manages a flat `Vec` of [`Cell`]s. Free slots form an
//! intrusive singly-linked free-list: each `Cell::Nothing` holds the
//! one-based index of the next free slot (or `None` if it is the last).
//! `head` always points to the first free slot, so allocation and
//! deallocation are O(1). When no free slots remain the backing `Vec` is
//! doubled via [`Allocator::reserve`].
//!
//! Slots are addressed through [`Token`]s, which use one-based indexing so
//! that the null / invalid state can be represented by `Option<NonZeroUsize>`
//! without a separate tag word.
#![allow(clippy::new_without_default)]
#![allow(unused)]
use std::mem;
use std::num::NonZeroUsize;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

use crate::token::Token;

/// A slot-map / arena allocator whose entries are addressed by [`Token`].
///
/// Internally the storage is a `Vec<Cell<T>>`. Live entries are
/// `Cell::Just(value)`; free entries are `Cell::Nothing(next)` where `next`
/// is the one-based index of the following free slot in the free-list
/// (or `None` for the tail).
///
/// # Indexing convention
/// All indices stored inside [`Token`] and inside the free-list links are
/// **one-based** so they can be stored as `NonZeroUsize`. The implementation
/// subtracts 1 when indexing into `data`.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Allocator<T> {
    /// Backing storage. `data[0]` is the slot with one-based index 1.
    data: Vec<Cell<T>>,
    /// One-based index of the first free slot, or `None` when fully occupied.
    head: Option<NonZeroUsize>,
    /// Number of occupied (live) slots.
    len: usize
}

/// A single storage slot inside an [`Allocator`].
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
enum Cell<T> {
    /// The slot is occupied and holds a value.
    Just(T),
    /// The slot is free. The payload is the one-based index of the next free
    /// slot in the intrusive free-list, or `None` if this is the last free
    /// slot.
    Nothing(Option<NonZeroUsize>)
}

impl<T> Default for Allocator<T> {
    /// Creates an allocator with a single pre-allocated free slot.
    fn default() -> Self {
        Allocator {
            data: vec![Cell::Nothing(None)],
            head: Some(NonZeroUsize::new(1).unwrap()),
            len: 0
        }
    }
}

impl<T> Allocator<T> {
    /// Creates a new, empty allocator with a single pre-allocated free slot.
    pub fn new() -> Self {
        Allocator {
            data: vec![Cell::Nothing(None)],
            head: Some(NonZeroUsize::new(1).unwrap()),
            len: 0
        }
    }

    /// Returns a [`Token`] pointing to the next free slot, growing the
    /// allocator if necessary.
    ///
    /// The returned token is **not** yet occupied; it is only a hint about
    /// where the *next* [`insert`](Self::insert) will land.
    pub fn head(&mut self) -> Token {
        match self.head {
            Some(head) => Token{ index: head },
            None => {
                self.reserve(self.len());
                self.head()
            }
        }
    }

    /// Returns the number of occupied (live) slots.
    pub fn len(&self) -> usize { self.len }

    /// Returns `true` if there are no occupied slots.
    pub fn is_empty(&self) -> bool { self.len == 0 }

    /// Returns the total number of slots (occupied + free) in the backing
    /// storage. Equivalent to `self.data.len()`.
    pub fn capacity(&self) -> usize { self.data.len() }

    /// Returns `true` if `token` refers to a currently-occupied slot.
    pub fn is_valid_token(&self, token: Token) -> bool {
        self.get(token).is_some()
    }

    /// Walks the free-list from `head` and returns the one-based index of the
    /// **last** free slot (i.e., the tail of the list), or `None` if the
    /// allocator is full.
    fn find_last_available(&self) -> Option<NonZeroUsize> {
        fn aux<T>(data: &[Cell<T>], indx: NonZeroUsize) -> Option<NonZeroUsize> {
            match data.get(indx.get() - 1) {  // get back to zero-based indexing
                // Dead code: corrupt-arena sentinel; unreachable via public API
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

    /// Appends `additional` new free slots to the backing storage and links
    /// them into the free-list.
    ///
    /// If the allocator is currently full (`head` is `None`) the new head is
    /// set to the first newly-allocated slot; otherwise the tail of the
    /// existing free-list is linked to the first new slot.
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve_exact(additional);
        let head_indx = NonZeroUsize::new(self.data.len() + 1).unwrap();
        match self.find_last_available() {
            None => self.head = Some(head_indx),
            Some(n) => self.data[n.get() - 1] = Cell::Nothing(Some(head_indx)),
        };
        // Build a chain: each new slot points to the next, the last points to None.
        let new_cells = (head_indx.get()..)  // already bigger by 1
            .take(additional - 1)
            .map(|i| Cell::Nothing(Some(NonZeroUsize::new(i + 1).unwrap())))
            .chain(std::iter::once(Cell::Nothing(None)));
        self.data.extend(new_cells);
    }

    /// Inserts `data` into the next free slot and returns a [`Token`] that
    /// can later be used to retrieve or remove the value.
    ///
    /// If no free slots are available the allocator doubles its capacity via
    /// [`reserve`](Self::reserve) before inserting.
    pub fn insert(&mut self, data: T) -> Token {
        match self.head {
            None => {
                self.reserve(self.capacity());
                self.insert(data)
            },
            Some(index) => {
                let i = index.get() - 1;  // zero-based index
                let next_head = match self.data.get(i) {
                    // Dead code: corrupt-arena sentinel; unreachable via public API
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

    /// Replaces the value at `token` with `data`, returning the old value.
    ///
    /// Returns `None` if `token` does not refer to a live slot. Note that the
    /// replacement is always inserted at the *next* free slot, which may
    /// differ from the original position of `token` if other slots are free.
    pub fn set(&mut self, token: Token, data: T) -> Option<T> {
        let out = self.remove(token);
        self.insert(data);
        out
    }

    /// Removes the value at `token`, freeing the slot for future use.
    ///
    /// Returns the removed value, or `None` if `token` does not refer to a
    /// live slot (already removed, never inserted, or out of range).
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
                    // Dead code: corrupt-arena sentinel; unreachable via public API
                    _ => panic!("something is wrong with the code")
                }
            }
        }
    }

    /// Returns a shared reference to the value at `token`, or `None` if the
    /// slot is free or the token is out of range.
    pub fn get(&self, token: Token) -> Option<&T> {
        match self.data.get(token.index.get() - 1) {  // zero-based index
            Some(Cell::Nothing(_)) | None => None,
            Some(Cell::Just(data)) => Some(data)
        }
    }

    /// Returns an exclusive reference to the value at `token`, or `None` if
    /// the slot is free or the token is out of range.
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
    fn find_last_available_recursive_case() {
        let mut alloc: Allocator<usize> = Allocator::new();
        alloc.reserve(2);
        alloc.reserve(1);
        assert!(alloc.capacity() >= 4);
        assert_eq!(alloc.len(), 0);
        let t = alloc.insert(42);
        assert_eq!(*alloc.get(t).unwrap(), 42);
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
