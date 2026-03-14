// FIXME: change all unwraps to get_unchecked
use crate::arena::Arena;
use crate::iter::*;
use crate::node::Node;
use crate::token::Token;

use std::ops::Deref;

pub struct NodeRef<'a, T> {
    token: Token,
    tree: &'a Arena<T>
}

impl<'a, T> NodeRef<'a, T> {
    pub fn get(&self) -> &'a T {
        &self.tree.get(self.token).unwrap().data
    }

    /// Checks whether a given node is actually a leaf.
    pub fn is_leaf(&self) -> bool {
        self.token.is_leaf(self.tree)
    }

    /// Returns the first child of the node.
    ///
    /// # Examples
    ///
    /// ```
    /// use atree::Arena;
    ///
    /// let root_data = "Indo-European";
    /// let (mut arena, root_token) = Arena::with_data("Germanic");
    /// let english = root_token.append(&mut arena, "English");
    ///
    /// let root = &arena[root_token];
    /// assert_eq!(root.first_child(), Some(english));
    /// ```
    pub fn first_child(&self) -> Option<Token> {
        self.tree.get(self.token).unwrap().first_child
    }

    /// Returns the parent of the node.
    ///
    /// # Examples
    ///
    /// ```
    /// use atree::Arena;
    ///
    /// let root_data = "Indo-European";
    /// let (mut arena, root_token) = Arena::with_data("Germanic");
    /// let english = root_token.append(&mut arena, "English");
    ///
    /// let child = &arena[english];
    /// assert_eq!(child.parent(), Some(root_token));
    /// ```
    pub fn parent(&self) -> Option<Token> {
        self.tree.get(self.token).unwrap().parent
    }

    /// Returns the previous sibling of the node.
    ///
    /// # Examples
    ///
    /// ```
    /// use atree::Arena;
    ///
    /// let root_data = "Indo-European";
    /// let (mut arena, root_token) = Arena::with_data("Germanic");
    /// let english = root_token.append(&mut arena, "English");
    /// let swedish = root_token.append(&mut arena, "Swedish");
    ///
    /// let second_child = &arena[swedish];
    /// assert_eq!(second_child.previous_sibling(), Some(english));
    /// ```
    pub fn previous_sibling(&self) -> Option<Token> {
        self.tree.get(self.token).unwrap().previous_sibling
    }

    /// Returns the next sibling of the node.
    ///
    /// # Examples
    ///
    /// ```
    /// use atree::Arena;
    ///
    /// let root_data = "Indo-European";
    /// let (mut arena, root_token) = Arena::with_data("Germanic");
    /// let english = root_token.append(&mut arena, "English");
    /// let swedish = root_token.append(&mut arena, "Swedish");
    ///
    /// let first_child = &arena[english];
    /// assert_eq!(first_child.next_sibling(), Some(swedish));
    /// ```
    pub fn next_sibling(&self) -> Option<Token> {
        self.tree.get(self.token).unwrap().next_sibling
    }
}

impl<'a, T> Deref for NodeRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}
