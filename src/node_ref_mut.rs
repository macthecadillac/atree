// FIXME: change all unwraps to get_unchecked
use crate::arena::Arena;
use crate::iter::*;
use crate::node::Node;
use crate::token::Token;
use crate::Error;

use std::mem::MaybeUninit;

pub struct NodeRefMut<'a, T> {
    token: Token,
    tree: &'a mut Arena<T>
}

fn node_operation<T>(
    self_token: Token,
    arena: &mut Arena<T>,
    other_token: Token,
    func: fn(Token, &mut Arena<T>, T) -> Token
) -> Result<(), Error> {
    // only a placeholder to get around some trait requirements so I can
    // reuse code. The uninitialized data will be removed so no risk here.
    let dummy_data: T = unsafe { MaybeUninit::zeroed().assume_init() };
    let token = func(self_token, arena, dummy_data);
    token.replace_node(arena, other_token)?;
    arena.remove(token);  // remove uninitialized data
    Ok(())
}

impl<'a, T> NodeRefMut<'a, T> {
    pub fn get_mut(self) -> &'a mut T {
        &mut self.tree.get_mut(self.token).unwrap().data
    }

    /// Checks whether a given node is actually a leaf.
    pub fn is_leaf(&self) -> bool {
        self.token.is_leaf(self.tree)
    }

    /// Creates a new node with the given data and append to the given node.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// panic!("test needed")
    /// ```
    pub fn append(self, data: T) -> Self {
        let new_node_token = self.tree.allocator.head();
        let previous_sibling = match self.token.children_mut(self.tree).last() {
            None => {
                // children_mut will have checked indexability so this will not
                // fail
                self.tree.get_mut(self.token).unwrap().first_child = Some(new_node_token);
                None
            },
            Some(last_child) => {
                last_child.next_sibling = Some(new_node_token);
                Some(last_child.token)
            }
        };

        let node = Node {
            data,
            token: new_node_token,
            parent: Some(self.token),
            previous_sibling,
            next_sibling: None,
            first_child: None
        };
        self.tree.set(new_node_token, node);
        self
    }

    /// Creates a new node with the given data and sets as the previous sibling
    /// of the current node.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// panic!("test needed")
    /// ```
    pub fn insert_before(self, data: T) -> Self {
        let new_node_token = self.tree.allocator.head();
        let (self_parent, self_previous_sibling) = match self.tree.get(self.token) {
            None => panic!("Invalid token"),
            Some(node) => (node.parent, node.previous_sibling)
        };
        self.tree.get_mut(self.token).unwrap().previous_sibling = Some(new_node_token);  // already checked
        let previous_sibling = match self_previous_sibling {
            Some(sibling) => match self.tree.get_mut(sibling) {
                None => panic!("Corrupt arena"),
                Some(ref mut node) => {
                    node.next_sibling = Some(new_node_token);
                    Some(sibling)
                }
            },
            None => match self_parent {
                None => panic!("Cannot insert as the previous sibling of the \
                                root node"),
                Some(p) => match self.tree.get_mut(p) {
                    None => panic!("Corrupt arena"),
                    Some(ref mut node) => {
                        node.first_child = Some(new_node_token);
                        None
                    }
                }
            }
        };

        let node = Node {
            data,
            token: new_node_token,
            parent: self_parent,
            previous_sibling,
            next_sibling: Some(self.token),
            first_child: None
        };
        self.tree.set(new_node_token, node);
        self
    }

    /// Returns the first child of the node.
    ///
    /// # Examples
    ///
    /// ```
    /// panic!("test needed")
    /// ```
    pub fn first_child(&self) -> Option<Token> {
        self.tree.get(self.token).unwrap().first_child
    }

    /// Returns the parent of the node.
    ///
    /// # Examples
    ///
    /// ```
    /// panic!("test needed")
    /// ```
    pub fn parent(&self) -> Option<Token> {
        self.tree.get(self.token).unwrap().parent
    }

    /// Returns the previous sibling of the node.
    ///
    /// # Examples
    ///
    /// ```
    /// panic!("test needed")
    /// ```
    pub fn previous_sibling(&self) -> Option<Token> {
        self.tree.get(self.token).unwrap().previous_sibling
    }

    /// Returns the next sibling of the node.
    ///
    /// # Examples
    ///
    /// ```
    /// panic!("test needed")
    /// ```
    pub fn next_sibling(&self) -> Option<Token> {
        self.tree.get(self.token).unwrap().next_sibling
    }
}
