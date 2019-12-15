use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

use crate::arena::Arena;
use crate::iter::*;
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
    /// tree.initialize(root_data);
    /// assert!(!tree.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool { self.arena.is_empty() }

    /// Counts the number of nodes.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
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


    /// Returns the token of the root node
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let mut tree = Tree::default();
    /// assert_eq!(tree.root_token(), None);
    ///
    /// let root_data = 1usize;
    /// let root_token = tree.initialize(root_data);
    /// assert_eq!(tree.root_token(), Some(root_token));
    /// ```
    pub fn root_token(&self) -> Option<Token> {
        if self.is_empty() { None }
        else { Some(Token { index: 0 }) }
    }

    /// Returns a reference to the root node.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, _) = Tree::with_root(root_data);
    /// assert_eq!(tree.root_node().unwrap().data, 1);
    /// ```
    pub fn root_node(&self) -> Option<&Node<T>> { self.get(Token { index: 0 }) }

    /// Returns a mutable reference to the root node.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, _) = Tree::with_root(root_data);
    ///
    /// let root_node = tree.root_node_mut().unwrap();
    /// root_node.data = 3;
    ///
    /// assert_eq!(tree.root_node().unwrap().data, 3);
    /// ```
    pub fn root_node_mut(&mut self) -> Option<&mut Node<T>> {
        self.get_mut(Token { index: 0 })
    }

    /// Creates tree with data at the root node.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
    /// assert_eq!(tree[root_token].data, 1);
    /// ```
    pub fn with_root(data: T) -> (Self, Token) {
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

    /// Creates root node on tree with data. Will erase all data if tree wasn't
    /// emtpy to begin with.
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
    /// tree.initialize(root_data);
    /// assert!(!tree.is_empty());
    /// ```
    pub fn initialize(&mut self, data: T) -> Token {
        let (tree, root_token) = Tree::with_root(data);
        *self = tree;
        root_token
    }

    /// Gets a reference to a node on the tree.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
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
    /// let (mut tree, root_token) = Tree::with_root(root_data);
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
    /// let (mut tree, root_token) = Tree::with_root(root_data);
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
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
    ///
    /// let next_node = root_token.append(&mut tree, 2usize);
    /// let nnext_node1 = next_node.append(&mut tree, 3usize);
    /// let nnext_node2 = next_node.append(&mut tree, 4usize);
    /// 
    /// tree.remove(next_node);
    /// let mut iter = tree.traverse_tokens_preord();
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

    /// Detaches subtree starting from the given node into its own tree.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node on the tree.
    ///
    /// # Examples:
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = "a0";
    /// let (mut atree, root_token) = Tree::with_root(root_data);
    ///
    /// let node1 = root_token.append(&mut atree, "a1");
    /// let node2 = root_token.append(&mut atree, "b1");
    /// let grandchild1 = node1.append(&mut atree, "a2");
    /// let grandchild2 = node2.append(&mut atree, "b2");
    ///
    /// // split tree
    /// let btree = atree.split_at(node2);
    /// let btree_root = btree.root_token().unwrap();
    ///
    /// let atree_elt: Vec<_> = atree.traverse_preord().map(|x| x.data).collect();
    /// let btree_elt: Vec<_> = btree.traverse_preord().map(|x| x.data).collect();
    ///
    /// assert_eq!(&["a0", "a1", "a2"], &atree_elt[..]);
    /// assert_eq!(&["b1", "b2"], &btree_elt[..]);
    /// ```
    pub fn split_at(&mut self, token: Token) -> Tree<T> where T: Clone {
        let root_data = match self.get(token) {
            Some(node) => node.data.clone(),
            None => panic!("Invalid token")
        };
        let (mut tree, root) = Tree::with_root(root_data);
        for child_token in token.children_tokens(&self) {
            root.append_subtree(&mut tree, child_token, self);
        }
        self.remove(token);
        tree
    }

    /// Returns an iterator of tokens in pre-order.
    ///
    /// # Examples:
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
    ///
    /// let first_child = root_token.append(&mut tree, 2usize);
    /// let second_child = root_token.append(&mut tree, 3usize);
    /// let third_child = root_token.append(&mut tree, 4usize);
    /// let first_grandchild = second_child.append(&mut tree, 10usize);
    /// let second_grandchild = second_child.append(&mut tree, 20usize);
    /// let fourth_child = root_token.append(&mut tree, 5usize);
    ///
    /// let mut iter = tree.traverse_tokens_preord();
    /// assert_eq!(iter.next(), Some(root_token));
    /// assert_eq!(iter.next(), Some(first_child));
    /// assert_eq!(iter.next(), Some(second_child));
    /// assert_eq!(iter.next(), Some(first_grandchild));
    /// assert_eq!(iter.next(), Some(second_grandchild));
    /// assert_eq!(iter.next(), Some(third_child));
    /// assert_eq!(iter.next(), Some(fourth_child));
    /// assert!(iter.next().is_none());
    /// ```
    pub fn traverse_tokens_preord(&self) -> PreordTokensTraversal<T> {
        let iter = match self.root_token() {
            None => IterEmptyOr::Empty,
            Some(root) => IterEmptyOr::Iter(root.descendants_tokens_preord(self))
        };
        PreordTokensTraversal { return_root: true, iter }
    }

    /// Returns an iterator of node references in pre-order.
    ///
    /// # Examples:
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
    ///
    /// root_token.append(&mut tree, 2usize);
    /// root_token.append(&mut tree, 3usize);
    /// let third_child = root_token.append(&mut tree, 4usize);
    /// root_token.append(&mut tree, 5usize);
    /// third_child.append(&mut tree, 10usize);
    /// third_child.append(&mut tree, 20usize);
    ///
    /// let mut iter = tree.traverse_preord();
    /// assert_eq!(iter.next().unwrap().data, 1);
    /// assert_eq!(iter.next().unwrap().data, 2);
    /// assert_eq!(iter.next().unwrap().data, 3);
    /// assert_eq!(iter.next().unwrap().data, 4);
    /// assert_eq!(iter.next().unwrap().data, 10);
    /// assert_eq!(iter.next().unwrap().data, 20);
    /// assert_eq!(iter.next().unwrap().data, 5);
    /// assert!(iter.next().is_none());
    /// ```
    pub fn traverse_preord(&self) -> PreordTraversal<T> {
        PreordTraversal { tree: self, iter: self.traverse_tokens_preord() }
    }

    /// Returns an iterator of mutable node references in pre-order.
    ///
    /// # Examples:
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
    ///
    /// root_token.append(&mut tree, 2usize);
    /// root_token.append(&mut tree, 3usize);
    /// let third_child = root_token.append(&mut tree, 4usize);
    /// root_token.append(&mut tree, 5usize);
    /// third_child.append(&mut tree, 10usize);
    /// third_child.append(&mut tree, 20usize);
    ///
    /// for x in tree.traverse_preord_mut() {
    ///     x.data += 100;
    /// }
    ///
    /// let mut iter = tree.traverse_preord();
    /// assert_eq!(iter.next().unwrap().data, 101);
    /// assert_eq!(iter.next().unwrap().data, 102);
    /// assert_eq!(iter.next().unwrap().data, 103);
    /// assert_eq!(iter.next().unwrap().data, 104);
    /// assert_eq!(iter.next().unwrap().data, 110);
    /// assert_eq!(iter.next().unwrap().data, 120);
    /// assert_eq!(iter.next().unwrap().data, 105);
    /// assert!(iter.next().is_none());
    /// ```
    pub fn traverse_preord_mut(&mut self) -> PreordTraversalMut<T> {
        PreordTraversalMut {
            tree: self as *mut Tree<T>,
            iter: self.traverse_tokens_preord(),
            marker: PhantomData::default()
        }
    }

    /// Returns an iterator of tokens in post-order.
    ///
    /// # Examples:
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
    ///
    /// let first_child = root_token.append(&mut tree, 2usize);
    /// let second_child = root_token.append(&mut tree, 3usize);
    /// let third_child = root_token.append(&mut tree, 4usize);
    /// let first_grandchild = second_child.append(&mut tree, 10usize);
    /// let second_grandchild = second_child.append(&mut tree, 20usize);
    /// let great_grandchild = second_grandchild.append(&mut tree, 20usize);
    /// let fourth_child = root_token.append(&mut tree, 5usize);
    ///
    /// let mut iter = tree.traverse_tokens_postord();
    /// assert_eq!(iter.next(), Some(first_child));
    /// assert_eq!(iter.next(), Some(first_grandchild));
    /// assert_eq!(iter.next(), Some(great_grandchild));
    /// assert_eq!(iter.next(), Some(second_grandchild));
    /// assert_eq!(iter.next(), Some(second_child));
    /// assert_eq!(iter.next(), Some(third_child));
    /// assert_eq!(iter.next(), Some(fourth_child));
    /// assert_eq!(iter.next(), Some(root_token));
    /// assert!(iter.next().is_none());
    /// ```
    pub fn traverse_tokens_postord(&self) -> PostordTokensTraversal<T> {
        let iter = match self.root_token() {
            None => IterEmptyOr::Empty,
            Some(root) => IterEmptyOr::Iter(root.descendants_tokens_postord(self))
        };
        PostordTokensTraversal { return_root: true, iter }
    }

    /// Returns an iterator of node references in post-order.
    ///
    /// # Examples:
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
    ///
    /// root_token.append(&mut tree, 2usize);
    /// root_token.append(&mut tree, 3usize);
    /// let third_child = root_token.append(&mut tree, 4usize);
    /// root_token.append(&mut tree, 5usize);
    /// third_child.append(&mut tree, 10usize);
    /// third_child.append(&mut tree, 20usize);
    ///
    /// let mut descendants = tree.traverse_postord();
    /// assert_eq!(descendants.next().unwrap().data, 2);
    /// assert_eq!(descendants.next().unwrap().data, 3);
    /// assert_eq!(descendants.next().unwrap().data, 10);
    /// assert_eq!(descendants.next().unwrap().data, 20);
    /// assert_eq!(descendants.next().unwrap().data, 4);
    /// assert_eq!(descendants.next().unwrap().data, 5);
    /// assert_eq!(descendants.next().unwrap().data, 1);
    /// assert!(descendants.next().is_none());
    /// ```
    pub fn traverse_postord(&self) -> PostordTraversal<T> {
        PostordTraversal { tree: self, iter: self.traverse_tokens_postord() }
    }

    /// Returns an iterator of mutable node references in post-order.
    ///
    /// # Examples:
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
    ///
    /// root_token.append(&mut tree, 2usize);
    /// root_token.append(&mut tree, 3usize);
    /// let third_child = root_token.append(&mut tree, 4usize);
    /// root_token.append(&mut tree, 5usize);
    /// third_child.append(&mut tree, 10usize);
    /// third_child.append(&mut tree, 20usize);
    ///
    /// for x in tree.traverse_postord_mut() {
    ///     x.data += 100;
    /// }
    ///
    /// let mut iter = tree.traverse_postord();
    /// assert_eq!(iter.next().unwrap().data, 102);
    /// assert_eq!(iter.next().unwrap().data, 103);
    /// assert_eq!(iter.next().unwrap().data, 110);
    /// assert_eq!(iter.next().unwrap().data, 120);
    /// assert_eq!(iter.next().unwrap().data, 104);
    /// assert_eq!(iter.next().unwrap().data, 105);
    /// assert_eq!(iter.next().unwrap().data, 101);
    /// assert!(iter.next().is_none());
    /// ```
    pub fn traverse_postord_mut(&mut self) -> PostordTraversalMut<T> {
        PostordTraversalMut {
            tree: self as *mut Tree<T>,
            iter: self.traverse_tokens_postord(),
            marker: PhantomData::default()
        }
    }

    /// Returns an iterator of tokens in level-order (breadth-first).
    ///
    /// # Examples:
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
    ///
    /// let first_child = root_token.append(&mut tree, 2usize);
    /// let second_child = root_token.append(&mut tree, 3usize);
    /// let third_child = root_token.append(&mut tree, 4usize);
    /// let first_grandchild = second_child.append(&mut tree, 10usize);
    /// let second_grandchild = second_child.append(&mut tree, 20usize);
    /// let fourth_child = root_token.append(&mut tree, 5usize);
    ///
    /// let mut iter = tree.traverse_tokens_levelord();
    /// assert_eq!(iter.next(), Some(root_token));
    /// assert_eq!(iter.next(), Some(first_child));
    /// assert_eq!(iter.next(), Some(second_child));
    /// assert_eq!(iter.next(), Some(third_child));
    /// assert_eq!(iter.next(), Some(fourth_child));
    /// assert_eq!(iter.next(), Some(first_grandchild));
    /// assert_eq!(iter.next(), Some(second_grandchild));
    /// assert!(iter.next().is_none());
    /// ```
    pub fn traverse_tokens_levelord(&self) -> LevelordTokensTraversal<T> {
        let iter = match self.root_token() {
            None => IterEmptyOr::Empty,
            Some(root) => IterEmptyOr::Iter(root.descendants_tokens_levelord(self))
        };
        LevelordTokensTraversal { return_root: true, iter }
    }

    /// Returns an iterator of node references in level-order (breadth-first).
    ///
    /// # Examples:
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
    ///
    /// root_token.append(&mut tree, 2usize);
    /// root_token.append(&mut tree, 3usize);
    /// let third_child = root_token.append(&mut tree, 4usize);
    /// root_token.append(&mut tree, 5usize);
    /// third_child.append(&mut tree, 10usize);
    /// third_child.append(&mut tree, 20usize);
    ///
    /// let mut iter = tree.traverse_levelord();
    /// assert_eq!(iter.next().unwrap().data, 1);
    /// assert_eq!(iter.next().unwrap().data, 2);
    /// assert_eq!(iter.next().unwrap().data, 3);
    /// assert_eq!(iter.next().unwrap().data, 4);
    /// assert_eq!(iter.next().unwrap().data, 5);
    /// assert_eq!(iter.next().unwrap().data, 10);
    /// assert_eq!(iter.next().unwrap().data, 20);
    /// assert!(iter.next().is_none());
    /// ```
    pub fn traverse_levelord(&self) -> LevelordTraversal<T> {
        LevelordTraversal { tree: self, iter: self.traverse_tokens_levelord() }
    }

    /// Returns an iterator of mutable node references in level-order.
    ///
    /// # Examples:
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
    ///
    /// let first_child = root_token.append(&mut tree, 2usize);
    /// let second_child = root_token.append(&mut tree, 3usize);
    /// let third_child = root_token.append(&mut tree, 4usize);
    /// let first_grandchild = second_child.append(&mut tree, 10usize);
    /// let second_grandchild = second_child.append(&mut tree, 20usize);
    /// let fourth_child = root_token.append(&mut tree, 5usize);
    ///
    /// for x in tree.traverse_levelord_mut() {
    ///     x.data += 100;
    /// }
    ///
    /// let mut iter = tree.traverse_levelord();
    /// assert_eq!(iter.next().unwrap().data, 101);
    /// assert_eq!(iter.next().unwrap().data, 102);
    /// assert_eq!(iter.next().unwrap().data, 103);
    /// assert_eq!(iter.next().unwrap().data, 104);
    /// assert_eq!(iter.next().unwrap().data, 105);
    /// assert_eq!(iter.next().unwrap().data, 110);
    /// assert_eq!(iter.next().unwrap().data, 120);
    /// assert!(iter.next().is_none());
    /// ```
    pub fn traverse_levelord_mut(&mut self) -> LevelordTraversalMut<T> {
        LevelordTraversalMut {
            tree: self as *mut Tree<T>,
            iter: self.traverse_tokens_levelord(),
            marker: PhantomData::default()
        }
    }

}

/// Deep copies the tree and reclaim the space freed by prior node removals
impl<T> Tree<T> where T: Clone {
    pub fn shrink_to_fit(&mut self) {
        if let Some(root) = self.root_node() {
            let old_root_token = root.token;
            let old_root_data = root.data.clone();
            let (mut new_tree, root_token) = Tree::with_root(old_root_data);
            for child_token in old_root_token.children_tokens(self) {
                root_token.append_subtree(&mut new_tree, child_token, self);
            }
            *self = new_tree
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
