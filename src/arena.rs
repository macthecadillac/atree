#![allow(clippy::match_bool)]
use std::collections::HashMap;
use std::ops::{Index, IndexMut};

use crate::alloc::Allocator;
use crate::iter::{Branch, ChildrenTokens};
use crate::node::Node;
use crate::token::Token;

/// A struct that provides the arena allocator.
#[derive(Default, Clone)]
pub struct Arena<T> {
    pub (crate) allocator: Allocator<Node<T>>
}

impl<T> Arena<T> {
    /// Returns true if the arena is empty.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Arena;
    ///
    /// let mut arena = Arena::default();
    /// assert!(arena.is_empty());
    ///
    /// let root_data = 1usize;
    /// arena.new_node(root_data);
    /// assert!(!arena.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool { self.allocator.is_empty() }

    /// Counts the number of nodes currently in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Arena;
    ///
    /// let root_data = 1usize;
    /// let (mut arena, root_token) = Arena::with_data(root_data);
    /// assert_eq!(arena.node_count(), 1);
    ///
    /// let next_node_token = root_token.append(&mut arena, 2usize);
    /// assert_eq!(arena.node_count(), 2);
    ///
    /// next_node_token.append(&mut arena, 3usize);
    /// assert_eq!(arena.node_count(), 3);
    /// ```
    pub fn node_count(&self) -> usize { self.allocator.len() }

    /// Returns the number of nodes the tree can hold without reallocating.
    pub fn capacity(&self) -> usize { self.allocator.capacity() }


    /// Initializes arena and initializes a new tree with the given data at the
    /// root node.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Arena;
    ///
    /// let root_data = 1usize;
    /// let (mut arena, root_token) = Arena::with_data(root_data);
    /// assert_eq!(arena[root_token].data, 1);
    /// ```
    pub fn with_data(data: T) -> (Self, Token) {
        let root_node = Node {
            data,
            parent: None,
            previous_sibling: None,
            token: Token::default(),
            next_sibling: None,
            first_child: None
        };
        let mut allocator = Allocator::new();
        let root_token = allocator.insert(root_node);
        (Arena { allocator }, root_token)
    }

    /// Creates a new free node in the given arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Arena;
    ///
    /// let mut arena = Arena::default();
    /// assert!(arena.is_empty());
    ///
    /// let root_data = 1usize;
    /// arena.new_node(root_data);
    /// assert!(!arena.is_empty());
    /// ```
    pub fn new_node(&mut self, data: T) -> Token {
        let token = self.allocator.head();
        let node = Node {
            data,
            parent: None,
            previous_sibling: None,
            token,
            next_sibling: None,
            first_child: None
        };
        self.allocator.set(token, node);
        token
    }

    /// Gets a reference to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Arena;
    ///
    /// let root_data = 1usize;
    /// let (mut arena, root_token) = Arena::with_data(root_data);
    /// let next_node_token = root_token.append(&mut arena, 2usize);
    ///
    /// // get the node we just inserted
    /// let next_node = arena.get(next_node_token).unwrap();
    /// assert_eq!(next_node.data, 2);
    /// ```
    pub fn get(&self, indx: Token) -> Option<&Node<T>> {
        self.allocator.get(indx)
    }

    /// Gets a mutable reference to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Arena;
    ///
    /// let root_data = 1usize;
    /// let (mut arena, root_token) = Arena::with_data(root_data);
    /// let next_node_token = root_token.append(&mut arena, 2usize);
    ///
    /// // get the node we just inserted
    /// let next_node = arena.get_mut(next_node_token).unwrap();
    /// // mutate the data as you wish
    /// next_node.data = 10;
    /// ```
    pub fn get_mut(&mut self, indx: Token) -> Option<&mut Node<T>> {
        self.allocator.get_mut(indx)
    }

    /// Sets data to node.
    pub (crate) fn set(&mut self, indx: Token, node: Node<T>) {
        if let Some(mut n) = self.allocator.set(indx, node) {
            n.remove_descendants(self)
        }
    }

    /// Overwrites node with given data and removes all its descendants.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Arena;
    ///
    /// let root_data = 1usize;
    /// let (mut arena, root_token) = Arena::with_data(root_data);
    ///
    /// let next_node = root_token.append(&mut arena, 2usize);
    /// let nnext_node1 = next_node.append(&mut arena, 3usize);
    /// let nnext_node2 = next_node.append(&mut arena, 4usize);
    /// 
    /// // now overwrite "next_node"
    /// arena.overwrite(next_node, 10);
    /// assert_eq!(arena[next_node].data, 10);
    ///  // the children of "next_node" are removed
    /// assert_eq!(arena.node_count(), 2);
    /// ```
    pub fn overwrite(&mut self, indx: Token, data: T) {
        indx.remove_descendants(self);  // this would panic if token is invalid
        if let Some(node) = self.get_mut(indx) {
            node.data = data
        }
    }

    /// Removes the given node from the arena and returns the tokens of its
    /// children. Use [`uproot`] instead if you no longer need the descendants
    /// of the node such that the freed memory could be reused.
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    /// ```
    /// use atree::Arena;
    /// use atree::iter::TraversalOrder;
    ///
    /// // root node that we will attach subtrees to
    /// let root_data = "Indo-European";
    /// let (mut arena, root) = Arena::with_data(root_data);
    ///
    /// // the Germanic branch
    /// let germanic = root.append(&mut arena, "Germanic");
    /// let west = germanic.append(&mut arena, "West");
    /// let scotts = west.append(&mut arena, "Scotts");
    /// let english = west.append(&mut arena, "English");
    ///
    /// // detach the west branch from the main tree
    /// let west_children = arena.remove(west);
    ///
    /// // the west branch is gone from the original tree
    /// let mut iter = root.subtree(&arena, TraversalOrder::Pre)
    ///     .map(|x| x.data);
    /// assert_eq!(iter.next(), Some("Indo-European"));
    /// assert_eq!(iter.next(), Some("Germanic"));
    /// assert!(iter.next().is_none());
    ///
    /// // its children are still areound
    /// let mut iter = west_children.iter().map(|&t| arena[t].data);
    /// assert_eq!(iter.next(), Some("Scotts"));
    /// assert_eq!(iter.next(), Some("English"));
    /// assert!(iter.next().is_none());
    /// ```
    ///
    /// [`uproot`]: struct.Arena.html#method.uproot
    // cannot return an iterator since we need to drop the mutable borrow
    pub fn remove(&mut self, token: Token) -> Vec<Token> {
        token.detach(self);
        // The chidlren will remain siblings. Change in the future if this leads
        // to problems.
        for child in token.children_mut(self) {
            child.parent = None;
        }
        // should not fail because children_mut checks the validity of token
        let first_child = self[token].first_child;
        self.allocator.remove(token);
        let iter = ChildrenTokens { arena: self, node_token: first_child };
        iter.collect()
    }

    /// Removes the given node along with all its descendants. If you only
    /// wanted to remove the node while keeping its children, use [`remove`]
    /// instead.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Arena;
    /// use atree::iter::TraversalOrder;
    ///
    /// let root_data = 1usize;
    /// let (mut arena, root_token) = Arena::with_data(root_data);
    ///
    /// let next_node = root_token.append(&mut arena, 2usize);
    /// let nnext_node1 = next_node.append(&mut arena, 3usize);
    /// let nnext_node2 = next_node.append(&mut arena, 4usize);
    /// 
    /// arena.uproot(next_node);
    /// let mut iter = root_token.subtree_tokens(&arena, TraversalOrder::Pre);
    /// assert_eq!(iter.next(), Some(root_token));
    /// assert!(iter.next().is_none());
    /// assert_eq!(arena.node_count(), 1);  // only the root node is left
    /// ```
    ///
    /// [`remove`]: struct.Arena.html#method.remove
    pub fn uproot(&mut self, token: Token) {
        token.remove_descendants(self);
        match self.allocator.remove(token) {
            None => panic!("Invalid token"),
            Some(node) => match (node.parent, node.previous_sibling,
                                 node.next_sibling) {
                (Some(_), Some(otkn), Some(ytkn)) => {
                    match self.get_mut(otkn) {
                        Some(o) => o.next_sibling = Some(ytkn),
                        None => panic!("Corrupt tree")
                    }
                    match self.get_mut(ytkn) {
                        Some(y) => y.previous_sibling = Some(otkn),
                        None => panic!("Corrupt tree")
                    }
                },
                (Some(_), Some(otkn), None) => match self.get_mut(otkn) {
                    Some(o) => o.next_sibling = None,
                    None => panic!("Corrupt tree")
                },
                (Some(ptkn), None, Some(ytkn)) => match self.get_mut(ptkn) {
                    Some(p) => p.first_child = Some(ytkn),
                    None => panic!("Corrupt tree")
                },
                (Some(ptkn), None, None) => match self.get_mut(ptkn) {
                    Some(p) => p.first_child = None,
                    None => panic!("Corrupt tree")
                },
                (None, None, None) => (),  // empty tree
                (None, None, Some(_))
                    | (None, Some(_), None)
                    | (None, Some(_), Some(_)) => panic!("Corrupt tree")
            }
        }
    }
}

impl<T> Arena<T> where T: Clone {
    /// Moves subtree with the root at the given node into its own arena. To
    /// detach a given subtree root node from a tree into its own while
    /// remaining in the same arena, use [`detach`] instead.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    /// ```
    /// use atree::Arena;
    /// use atree::iter::TraversalOrder;
    ///
    /// let root_data = "a0";
    /// let (mut arena1, root1) = Arena::with_data(root_data);
    ///
    /// let node1 = root1.append(&mut arena1, "a1");
    /// let node2 = root1.append(&mut arena1, "b1");
    /// let grandchild1 = node1.append(&mut arena1, "a2");
    /// let grandchild2 = node2.append(&mut arena1, "b2");
    ///
    /// // split tree
    /// let (arena2, root2) = arena1.split_at(node2);
    ///
    /// let arena1_elt: Vec<_> = root1.subtree(&arena1, TraversalOrder::Pre)
    ///     .map(|x| x.data).collect();
    /// let arena2_elt: Vec<_> = root2.subtree(&arena2, TraversalOrder::Pre)
    ///     .map(|x| x.data).collect();
    ///
    /// assert_eq!(&["a0", "a1", "a2"], &arena1_elt[..]);
    /// assert_eq!(&["b1", "b2"], &arena2_elt[..]);
    /// ```
    ///
    /// [`detach`]: struct.Token.html#method.detach
    // TODO: could probably be optimized
    pub fn split_at(&mut self, token: Token) -> (Self, Token) where T: Clone {
        let root_data = match self.get(token) {
            Some(node) => node.data.clone(),
            None => panic!("Invalid token")
        };
        let (mut arena, root) = Arena::with_data(root_data);
        for child_token in token.children_tokens(&self) {
            arena.copy_and_append_subtree(root, self, child_token);
        }
        self.uproot(token);
        (arena, root)
    }

    /// Copies a sub-tree from one arena and append to the given node of another.
    /// It does so by walking the tree and copying node by node to the target
    /// arena.  Potentially expensive operation.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    /// ```
    /// use atree::Arena;
    /// use atree::iter::TraversalOrder;
    ///
    /// let root_data = "John";
    /// let (mut arena1, root_token) = Arena::with_data(root_data);
    ///
    /// let node1 = root_token.append(&mut arena1, "Juan");
    /// let node2 = root_token.append(&mut arena1, "Giovanni");
    /// let grandchild1 = node1.append(&mut arena1, "Ivan");
    /// let grandchild2 = node2.append(&mut arena1, "Johann");
    ///
    /// // new arena
    /// let mut arena2 = arena1.clone();
    ///
    /// // append "node1" from tree2 under "node2" in tree1
    /// arena1.copy_and_append_subtree(node2, &arena2, node1);
    /// let mut subtree = node2.subtree(&arena1, TraversalOrder::Pre);
    ///
    /// assert_eq!(subtree.next().unwrap().data, "Giovanni");
    /// assert_eq!(subtree.next().unwrap().data, "Johann");
    /// assert_eq!(subtree.next().unwrap().data, "Juan");
    /// assert_eq!(subtree.next().unwrap().data, "Ivan");
    /// assert!(subtree.next().is_none());
    /// ```
    pub fn copy_and_append_subtree(&mut self, self_token: Token,
                                   other_tree: &Arena<T>, other_token: Token) {
        match other_tree.get(other_token) {
            None => panic!("Invalid token"),
            Some(node) => {
                let new_subtree_root = self_token.append(self, node.data.clone());
                let mut index_map: HashMap<Token, Token> = HashMap::new();
                index_map.insert(other_token, new_subtree_root);

                let mut stack = vec![other_token];
                let mut branch = Branch::Child;

                loop {
                    let &token = stack.last().unwrap(); // never fails
                    let node = &other_tree[token];  // already checked
                    match branch {
                        Branch::None => (),  // unreachable
                        Branch::Child => match node.first_child {
                            None => branch = Branch::Sibling,
                            Some(child) => {
                                let child_data = match other_tree.get(child) {
                                    Some(node) => node.data.clone(),
                                    None => panic!("Corrupt tree")
                                };
                                let new_parent = index_map[&token];
                                let new_child_token =
                                    new_parent.append(self, child_data);
                                index_map.insert(child, new_child_token);
                                stack.push(child);
                            }
                        },
                        Branch::Sibling => match Some(other_token) == stack.pop() {
                            true => break,
                            false => match node.next_sibling {
                                None => (),
                                Some(sibling) => {
                                    stack.push(sibling);
                                    branch = Branch::Child;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<T> Index<Token> for Arena<T> {
    type Output = Node<T>;
    fn index(&self, index: Token) -> &Self::Output {
        match self.get(index) {
            Some(node) => node,
            None => panic!("Invalid token")
        }
    }
}

impl<T> IndexMut<Token> for Arena<T> {
    fn index_mut(&mut self, index: Token) -> &mut Self::Output {
        match self.get_mut(index) {
            Some(node) => node,
            None => panic!("Invalid token")
        }
    }
}
