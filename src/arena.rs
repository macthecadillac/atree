#![allow(clippy::match_bool)]
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::ops::{Index, IndexMut};

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

use crate::alloc::Allocator;
use crate::iter::{Branch, ChildrenTokens};
use crate::node::Node;
use crate::token::Token;

/// A struct that provides the arena allocator.
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Arena<T> {
    pub (crate) allocator: Allocator<Node<T>>
}

impl<T> Arena<T> {
    /// Initializes a new `Arena<T>`.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Arena;
    ///
    /// let arena = Arena::<usize>::new();
    /// assert!(arena.is_empty());
    /// assert_eq!(arena.node_count(), 0);
    /// ```
    pub fn new() -> Self { Arena { allocator: Allocator::new() } }

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
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Arena;
    ///
    /// let (mut arena, root_token) = Arena::with_data(1usize);
    /// let initial_capacity = arena.capacity();
    ///
    /// // capacity grows as nodes are added beyond initial allocation
    /// for i in 0..100 {
    ///     root_token.append(&mut arena, i);
    /// }
    /// assert!(arena.capacity() >= initial_capacity);
    /// ```
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
            token: Token { index: NonZeroUsize::new(1).unwrap() },
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
        if self.get(indx).is_some() {
            indx.remove_descendants(self);
        }
        self.allocator.set(indx, node);
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
    /// let scots = west.append(&mut arena, "Scots");
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
    /// assert_eq!(iter.next(), Some("Scots"));
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
    /// // only one node is left
    /// assert_eq!(arena.node_count(), 1);
    /// // the node left is the root node
    /// assert_eq!(arena[root_token].data, root_data);
    /// ```
    ///
    /// [`remove`]: struct.Arena.html#method.remove
    pub fn uproot(&mut self, token: Token) {
        token.remove_descendants(self);
        match self.allocator.remove(token) {
            // Dead code: corrupt-arena sentinel; unreachable via public API
            None => panic!("Impossible branch. Token was referenced in the previous line."),
            Some(node) => match (node.parent, node.previous_sibling,
                                 node.next_sibling) {
                (Some(_), Some(otkn), Some(ytkn)) => {
                    match self.get_mut(otkn) {
                        Some(o) => o.next_sibling = Some(ytkn),
                        // Dead code: corrupt-arena sentinel; unreachable via public API
                        None => panic!("Impossible branch. Referencing dangling token. Corrupt arena")
                    }
                    match self.get_mut(ytkn) {
                        Some(y) => y.previous_sibling = Some(otkn),
                        // Dead code: corrupt-arena sentinel; unreachable via public API
                        None => panic!("Impossible branch. Referencing dangling token. Corrupt arena")
                    }
                },
                (Some(_), Some(otkn), None) => match self.get_mut(otkn) {
                    Some(o) => o.next_sibling = None,
                    // Dead code: corrupt-arena sentinel; unreachable via public API
                    None => panic!("Impossible branch. Referencing dangling token. Corrupt arena")
                },
                (Some(ptkn), None, Some(ytkn)) => {
                    match self.get_mut(ptkn) {
                        Some(p) => p.first_child = Some(ytkn),
                        // Dead code: corrupt-arena sentinel; unreachable via public API
                        None => panic!("Impossible branch. A root node cannot have siblings. Corrupt arena")
                    };
                    match self.get_mut(ytkn) {
                        Some(o) => o.previous_sibling = None,
                        // Dead code: corrupt-arena sentinel; unreachable via public API
                        None => panic!("Impossible branch. Referencing dangling token. Corrupt arena")
                    };
                },
                (Some(ptkn), None, None) => match self.get_mut(ptkn) {
                    Some(p) => p.first_child = None,
                    // Dead code: corrupt-arena sentinel; unreachable via public API
                    None => panic!("Impossible branch. Parent of non-root node not found. Corrupt arena")
                },
                (None, None, None) => (),  // empty tree
                // Dead code: corrupt-arena sentinel; unreachable via public API
                (None, None, Some(_))
                    | (None, Some(_), None)
                    | (None, Some(_), Some(_)) => panic!("Impossible branches. Corrupt arena")
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
        for child_token in token.children_tokens(self) {
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
    /// let grandchild2 = node1.append(&mut arena1, "Sean");
    /// let grandchild3 = node2.append(&mut arena1, "Johann");
    /// let grandchild4 = node2.append(&mut arena1, "Jan");
    ///
    /// // new arena
    /// let mut arena2 = arena1.clone();
    ///
    /// // append "node1" from tree2 under "node2" in tree1
    /// arena1.copy_and_append_subtree(node2, &arena2, node1);
    ///
    /// let mut node2_children = node2.children(&arena1).map(|t| t.data);
    /// assert_eq!(node2_children.next(), Some("Johann"));
    /// assert_eq!(node2_children.next(), Some("Jan"));
    /// assert_eq!(node2_children.next(), Some("Juan"));
    /// assert!(node2_children.next().is_none());
    ///
    /// let mut subtree = node2.subtree(&arena1, TraversalOrder::Pre);
    /// assert_eq!(subtree.next().unwrap().data, "Giovanni");
    /// assert_eq!(subtree.next().unwrap().data, "Johann");
    /// assert_eq!(subtree.next().unwrap().data, "Jan");
    /// let mut tree2 = node1.subtree(&arena2, TraversalOrder::Pre);
    /// assert!(subtree.zip(tree2).all(|(a, b)| a.data == b.data));
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
                    let &token = stack.last().expect("Stack should never be empty");
                    let node = &other_tree[token];  // already checked
                    match branch {
                        Branch::None => (),  // unreachable
                        Branch::Child => match node.first_child {
                            None => branch = Branch::Sibling,
                            Some(child) => {
                                let child_data = match other_tree.get(child) {
                                    Some(node) => node.data.clone(),
                                    None => panic!("Corrupt arena")
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
                                    let sibling_data = match other_tree.get(sibling) {
                                        Some(n) => n.data.clone(),
                                        None => panic!("Corrupt arena")
                                    };
                                    let parent_token = node.parent.expect("Corrupt arena");
                                    let new_parent = index_map[&parent_token];
                                    let new_sibling_token = new_parent.append(self, sibling_data);
                                    index_map.insert(sibling, new_sibling_token);
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn set_overwrites_node_data() {
        use crate::node::Node;
        let (mut arena, root) = Arena::with_data(0usize);
        let child = root.append(&mut arena, 1usize);
        assert_eq!(arena.node_count(), 2);

        // Replace `child` with a new node via the internal set() method
        let replacement = Node {
            data: 99usize,
            token: child,
            parent: Some(root),
            previous_sibling: None,
            next_sibling: None,
            first_child: None,
        };
        arena.set(child, replacement);

        // Node data is overwritten
        assert_eq!(arena[child].data, 99usize);
        assert_eq!(arena.node_count(), 2);
    }

    #[test]
    fn set_overwrites_node_removing_descendants() {
        use crate::node::Node;
        let (mut arena, root) = Arena::with_data(0usize);
        let child = root.append(&mut arena, 1usize);
        child.append(&mut arena, 2usize);  // grandchild
        assert_eq!(arena.node_count(), 3);

        // Replace `child` with a childless node via the internal set() method
        let replacement = Node {
            data: 99usize,
            token: child,
            parent: Some(root),
            previous_sibling: None,
            next_sibling: None,
            first_child: None,
        };
        arena.set(child, replacement);

        // Grandchild should have been freed — node count drops from 3 to 2
        assert_eq!(arena.node_count(), 2);
        assert_eq!(arena[child].data, 99usize);
    }

    #[test]
    fn uproot_middle_child() {
        let (mut arena, root) = Arena::with_data(0usize);
        let a = root.append(&mut arena, 1usize);
        let b = root.append(&mut arena, 2usize);
        let c = root.append(&mut arena, 3usize);

        arena.uproot(b);

        assert_eq!(arena.node_count(), 3);
        // a's next sibling should be c
        assert_eq!(arena[a].next_sibling, Some(c));
        // c's previous sibling should be a
        assert_eq!(arena[c].previous_sibling, Some(a));
    }

    #[test]
    fn uproot_last_child() {
        let (mut arena, root) = Arena::with_data(0usize);
        let a = root.append(&mut arena, 1usize);
        let b = root.append(&mut arena, 2usize);

        arena.uproot(b);

        assert_eq!(arena.node_count(), 2);
        assert!(arena[a].next_sibling.is_none());
    }

    #[test]
    fn uproot_first_child_with_siblings() {
        let (mut arena, root) = Arena::with_data(0usize);
        let a = root.append(&mut arena, 1usize);
        let b = root.append(&mut arena, 2usize);

        arena.uproot(a);

        assert_eq!(arena.node_count(), 2);
        // root's first_child should now be b
        assert_eq!(arena[root].first_child, Some(b));
        assert!(arena[b].previous_sibling.is_none());
        // uproot only updates parent's first_child, not b's previous_sibling
        // so b.parent is intact
        assert_eq!(arena[b].parent, Some(root));
    }

    #[test]
    fn uproot_root_node() {
        let (mut arena, root) = Arena::with_data(42usize);
        arena.uproot(root);
        assert_eq!(arena.node_count(), 0);
        assert!(arena.is_empty());
    }

    #[test]
    fn copy_and_append_subtree_with_siblings_2() {
        // Source tree: root1 -> [node1 -> [grandchild1, grandchild2], node2]
        let (mut arena1, root1) = Arena::with_data("root");
        let node1 = root1.append(&mut arena1, "node1");
        node1.append(&mut arena1, "gc1");
        node1.append(&mut arena1, "gc2");
        root1.append(&mut arena1, "node2");

        let (mut arena2, root2) = Arena::with_data("root2");
        // copy node1's subtree (which has siblings gc1, gc2) into arena2
        arena2.copy_and_append_subtree(root2, &arena1, node1);

        // root2 should now have node1 as a child with gc1 and gc2 as grandchildren
        let children: Vec<_> = root2.children_tokens(&arena2).collect();
        assert_eq!(children.len(), 1);
        let copied_node1 = children[0];
        assert_eq!(arena2[copied_node1].data, "node1");

        let grandchildren: Vec<_> = copied_node1.children_tokens(&arena2).collect();
        assert_eq!(grandchildren.len(), 2);
        assert_eq!(arena2[grandchildren[0]].data, "gc1");
        assert_eq!(arena2[grandchildren[1]].data, "gc2");
    }
}

impl<T> Index<Token> for Arena<T> {
    type Output = Node<T>;
    fn index(&self, index: Token) -> &Self::Output {
        match self.get(index) {
            Some(node) => node,
            // Dead code: intentional documented panic; not reachable without a stale/invalid token
            None => panic!("Invalid token")
        }
    }
}

impl<T> IndexMut<Token> for Arena<T> {
    fn index_mut(&mut self, index: Token) -> &mut Self::Output {
        match self.get_mut(index) {
            Some(node) => node,
            // Dead code: intentional documented panic; not reachable without a stale/invalid token
            None => panic!("Invalid token")
        }
    }
}
