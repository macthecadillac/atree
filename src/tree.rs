#![allow(clippy::match_bool)]
use std::collections::HashMap;
use std::ops::{Index, IndexMut};

use crate::arena::Arena;
use crate::iter::Branch;
use crate::node::Node;
use crate::token::Token;

/// A struct that provides the arena allocator.
#[derive(Default, Clone)]
pub struct Tree<T> {
    pub (crate) arena: Arena<Node<T>>
}

impl<T> Tree<T> {
    /// Returns true if the tree is empty.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let mut tree = Tree::default();
    /// assert!(tree.is_empty());
    ///
    /// let root_data = 1usize;
    /// tree.new_node(root_data);
    /// assert!(!tree.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool { self.arena.is_empty() }

    /// Counts the number of nodes currently in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    /// assert_eq!(tree.node_count(), 1);
    ///
    /// let next_node_token = root_token.append(&mut tree, 2usize);
    /// assert_eq!(tree.node_count(), 2);
    ///
    /// next_node_token.append(&mut tree, 3usize);
    /// assert_eq!(tree.node_count(), 3);
    /// ```
    pub fn node_count(&self) -> usize { self.arena.len() }

    /// Returns the number of nodes the tree can hold without reallocating.
    pub fn capacity(&self) -> usize { self.arena.capacity() }


    /// Initializes arena and initializes a new tree with the given data at the
    /// root node.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    /// assert_eq!(tree[root_token].data, 1);
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
        let mut arena = Arena::new();
        let root_token = arena.insert(root_node);
        (Tree { arena }, root_token)
    }

    /// Initializes a new tree in existing arena and returns the root token.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let mut tree = Tree::default();
    /// assert!(tree.is_empty());
    ///
    /// let root_data = 1usize;
    /// tree.new_node(root_data);
    /// assert!(!tree.is_empty());
    /// ```
    pub fn new_node(&mut self, data: T) -> Token {
        let token = self.arena.head();
        let node = Node {
            data,
            parent: None,
            previous_sibling: None,
            token,
            next_sibling: None,
            first_child: None
        };
        self.arena.set(token, node);
        token
    }

    /// Gets a reference to a node on the tree.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    /// let next_node_token = root_token.append(&mut tree, 2usize);
    ///
    /// // get the node we just inserted
    /// let next_node = tree.get(next_node_token).unwrap();
    /// assert_eq!(next_node.data, 2);
    /// ```
    pub fn get(&self, indx: Token) -> Option<&Node<T>> {
        self.arena.get(indx)
    }

    /// Gets a mutable reference to a node on the tree.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    /// let next_node_token = root_token.append(&mut tree, 2usize);
    ///
    /// // get the node we just inserted
    /// let next_node = tree.get_mut(next_node_token).unwrap();
    /// // mutate the data as you wish
    /// next_node.data = 10;
    /// ```
    pub fn get_mut(&mut self, indx: Token) -> Option<&mut Node<T>> {
        self.arena.get_mut(indx)
    }

    /// Sets data to node.
    pub (crate) fn set(&mut self, indx: Token, node: Node<T>) {
        if let Some(mut n) = self.arena.set(indx, node) {
            n.remove_descendants(self)
        }
    }

    /// Overwrites node with given data and removes all its descendants.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node on the tree.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    ///
    /// let next_node = root_token.append(&mut tree, 2usize);
    /// let nnext_node1 = next_node.append(&mut tree, 3usize);
    /// let nnext_node2 = next_node.append(&mut tree, 4usize);
    /// 
    /// // now overwrite "next_node"
    /// tree.overwrite(next_node, 10);
    /// assert_eq!(tree[next_node].data, 10);
    ///  // the children of "next_node" are removed
    /// assert_eq!(tree.node_count(), 2);
    /// ```
    pub fn overwrite(&mut self, indx: Token, data: T) {
        indx.remove_descendants(self);  // this would panic if token is invalid
        if let Some(node) = self.get_mut(indx) {
            node.data = data
        }
    }

    /// Removes the given node along with all its descendants.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node on the tree.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    /// use atree::iter::TraversalOrder;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    ///
    /// let next_node = root_token.append(&mut tree, 2usize);
    /// let nnext_node1 = next_node.append(&mut tree, 3usize);
    /// let nnext_node2 = next_node.append(&mut tree, 4usize);
    /// 
    /// tree.remove(next_node);
    /// let mut iter = root_token.subtree_tokens(&tree, TraversalOrder::Pre);
    /// assert_eq!(iter.next(), Some(root_token));
    /// assert!(iter.next().is_none());
    /// assert_eq!(tree.node_count(), 1);  // only the root node is left
    /// ```
    pub fn remove(&mut self, token: Token) {
        token.remove_descendants(self);
        match self.arena.remove(token) {
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

impl<T> Tree<T> where T: Clone {
    /// **Example is not ideal**
    /// Detaches subtree starting from the given node into its tree in its own
    /// arena.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node on the tree.
    ///
    /// # Examples:
    /// ```
    /// use atree::Tree;
    /// use atree::iter::TraversalOrder;
    ///
    /// let root_data = "a0";
    /// let (mut atree, root_token) = Tree::with_data(root_data);
    ///
    /// let node1 = root_token.append(&mut atree, "a1");
    /// let node2 = root_token.append(&mut atree, "b1");
    /// let grandchild1 = node1.append(&mut atree, "a2");
    /// let grandchild2 = node2.append(&mut atree, "b2");
    ///
    /// // split tree
    /// let btree = atree.split_at(node2);
    ///
    /// let atree_elt: Vec<_> = root_token.subtree(&atree, TraversalOrder::Pre)
    ///     .map(|x| x.data).collect();
    /// let btree_elt: Vec<_> = root_token.subtree(&btree, TraversalOrder::Pre)
    ///     .map(|x| x.data).collect();
    ///
    /// assert_eq!(&["a0", "a1", "a2"], &atree_elt[..]);
    /// assert_eq!(&["b1", "b2"], &btree_elt[..]);
    /// ```
    // TODO: could probably be optimized
    pub fn split_at(&mut self, token: Token) -> Tree<T> where T: Clone {
        let root_data = match self.get(token) {
            Some(node) => node.data.clone(),
            None => panic!("Invalid token")
        };
        let (mut tree, root) = Tree::with_data(root_data);
        for child_token in token.children_tokens(&self) {
            tree.append_subtree(root, self, child_token);
        }
        self.remove(token);
        tree
    }

    /// Appends a sub-tree from one arena to a given node of another. It does so
    /// by walking the tree and copying node by node to the target tree.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node on the tree.
    ///
    /// # Examples:
    /// ```
    /// use atree::Tree;
    /// use atree::iter::TraversalOrder;
    ///
    /// let root_data = "John";
    /// let (mut tree1, root_token) = Tree::with_data(root_data);
    ///
    /// let node1 = root_token.append(&mut tree1, "Juan");
    /// let node2 = root_token.append(&mut tree1, "Giovanni");
    /// let grandchild1 = node1.append(&mut tree1, "Ivan");
    /// let grandchild2 = node2.append(&mut tree1, "Johann");
    ///
    /// let mut tree2 = tree1.clone();
    ///
    /// // append "node1" from tree2 under "node2" in tree1
    /// tree1.append_subtree(node2, &tree2, node1);
    /// let mut subtree = node2.subtree(&tree1, TraversalOrder::Pre);
    ///
    /// assert_eq!(subtree.next().unwrap().data, "Giovanni");
    /// assert_eq!(subtree.next().unwrap().data, "Johann");
    /// assert_eq!(subtree.next().unwrap().data, "Juan");
    /// assert_eq!(subtree.next().unwrap().data, "Ivan");
    /// assert!(subtree.next().is_none());
    /// ```
    pub fn append_subtree(&mut self, self_token: Token,
                          other_tree: &Tree<T>, other_token: Token) {
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

impl<T> Index<Token> for Tree<T> {
    type Output = Node<T>;
    fn index(&self, index: Token) -> &Self::Output {
        match self.get(index) {
            Some(node) => node,
            None => panic!("Invalid token")
        }
    }
}

impl<T> IndexMut<Token> for Tree<T> {
    fn index_mut(&mut self, index: Token) -> &mut Self::Output {
        match self.get_mut(index) {
            Some(node) => node,
            None => panic!("Invalid token")
        }
    }
}
