#![allow(clippy::match_bool)]
use std::collections::VecDeque;
use std::marker::PhantomData;

use crate::iter::*;
use crate::node::Node;
use crate::tree::Tree;

/// A `Token` is a handle to a node in the arena.
#[derive(Clone, Copy, Eq, PartialEq, Default, Debug, Hash)]
pub struct Token{
    pub (crate) index: usize
}

impl Token {
    /// Creates a new node with the given data and append to the given node.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    /// use atree::iter::TraversalOrder;
    ///
    /// let root_data = "Indo-European";
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    ///
    /// let next_node_token = root_token.append(&mut tree, "Germanic");
    /// next_node_token.append(&mut tree, "Romance");
    /// let mut subtree = root_token.subtree(&tree, TraversalOrder::Pre);
    ///
    /// assert_eq!(subtree.next().unwrap().data, "Indo-European");
    /// assert_eq!(subtree.next().unwrap().data, "Germanic");
    /// assert_eq!(subtree.next().unwrap().data, "Romance");
    /// ```
    pub fn append<T>(self, tree: &mut Tree<T>, data: T) -> Token {
        let new_node_token = tree.arena.head();
        let previous_sibling = match self.children_mut(tree).last() {
            None => {
                match tree.get_mut(self) {
                    None => panic!("Invalid token"),
                    Some(curr_node) => curr_node.first_child = Some(new_node_token)
                }
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
            parent: Some(self),
            previous_sibling,
            next_sibling: None,
            first_child: None
        };
        tree.set(new_node_token, node);
        new_node_token
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
    /// use atree::Tree;
    /// use atree::iter::TraversalOrder;
    ///
    /// let root_data = "Indo-European";
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    ///
    /// let child2 = root_token.append(&mut tree, "Germanic");
    /// let child4 = root_token.append(&mut tree, "Slavic");
    /// child2.append(&mut tree, "English");
    /// // insert in between children 2 and 4
    /// let child3 = child4.insert_before(&mut tree, "Romance");
    /// // insert before child 2
    /// let child1 = child2.insert_before(&mut tree, "Celtic");
    ///
    /// let subtree: Vec<_> = root_token.subtree(&tree, TraversalOrder::Pre)
    ///     .map(|x| x.data)
    ///     .collect();
    /// assert_eq!(&["Indo-European", "Celtic", "Germanic", "English", "Romance", "Slavic"],
    ///            &subtree[..]);
    /// ```
    pub fn insert_before<T>(self, tree: &mut Tree<T>, data: T) -> Token {
        let new_node_token = tree.arena.head();
        let (self_parent, self_previous_sibling) = match tree.get(self) {
            None => panic!("Invalid token"),
            Some(node) => (node.parent, node.previous_sibling)
        };
        tree[self].previous_sibling = Some(new_node_token);  // already checked
        let previous_sibling = match self_previous_sibling {
            Some(sibling) => match tree.get_mut(sibling) {
                None => panic!("Corrupt tree"),
                Some(ref mut node) => {
                    node.next_sibling = Some(new_node_token);
                    Some(sibling)
                }
            },
            None => match self_parent {
                None => panic!("Cannot insert as the previous sibling of the \
                                root node"),
                Some(p) => match tree.get_mut(p) {
                    None => panic!("Corrupt tree"),
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
            next_sibling: Some(self),
            first_child: None
        };
        tree.set(new_node_token, node);
        new_node_token
    }

    /// Creates a new node with the given data and sets as the next sibling of
    /// the current node.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    /// use atree::iter::TraversalOrder;
    ///
    /// let root_data = "Indo-European";
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    ///
    /// let child1 = root_token.append(&mut tree, "Romance");
    /// let child3 = root_token.append(&mut tree, "Germanic");
    /// child1.append(&mut tree, "French");
    /// // insert betwern children 1 and 4
    /// let child2 = child1.insert_after(&mut tree, "Celtic");
    /// // insert after child 3
    /// child3.insert_after(&mut tree, "Slavic");
    ///
    /// let subtree: Vec<_> = root_token.subtree(&tree, TraversalOrder::Pre)
    ///     .map(|x| x.data)
    ///     .collect();
    /// assert_eq!(&["Indo-European", "Romance", "French", "Celtic", "Germanic", "Slavic"],
    ///            &subtree[..]);
    /// ```
    pub fn insert_after<T>(self, tree: &mut Tree<T>, data: T) -> Token {
        let new_node_token = tree.arena.head();
        let (self_parent, self_next_sibling) = match tree.get(self) {
            None => panic!("Invalid token"),
            Some(node) => (node.parent, node.next_sibling)
        };
        tree[self].next_sibling = Some(new_node_token);  // already checked
        let next_sibling = match self_next_sibling {
            None => None,
            Some(sibling) => match tree.get_mut(sibling) {
                None => panic!("Corrupt tree"),
                Some(ref mut node) => {
                    node.previous_sibling = Some(new_node_token);
                    Some(sibling)
                }
            },
        };

        let node = Node {
            data,
            token: new_node_token,
            parent: self_parent,
            previous_sibling: Some(self),
            next_sibling,
            first_child: None
        };
        tree.set(new_node_token, node);
        new_node_token
    }

    /// Returns an iterator of tokens of ancestor nodes.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = "Indo-European";
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    ///
    /// let child_token = root_token.append(&mut tree, "Germanic");
    /// let grandchild_token = child_token.append(&mut tree, "English");
    /// let mut ancestors_tokens = grandchild_token.ancestors_tokens(&tree);
    ///
    /// assert_eq!(ancestors_tokens.next(), Some(child_token));
    /// assert_eq!(ancestors_tokens.next(), Some(root_token));
    /// assert!(ancestors_tokens.next().is_none());
    /// ```
    pub fn ancestors_tokens<'a, T>(self, tree: &'a Tree<T>)
        -> AncestorTokens<'a, T> {
        let parent = match tree.get(self) {
            Some(n) => n.parent,
            None => panic!("Invalid token")
        };
        AncestorTokens { tree, node_token: parent }
    }

    /// Returns an iterator of tokens of siblings preceding the current node.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = "Indo-European";
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    ///
    /// let first_child_token = root_token.append(&mut tree, "Germanic");
    /// let second_child_token = root_token.append(&mut tree, "Romance");
    /// let third_child_token = root_token.append(&mut tree, "Slavic");
    /// root_token.append(&mut tree, "Hellenic");
    ///
    /// let mut sibling_tokens = third_child_token.preceding_siblings_tokens(&tree);
    /// assert_eq!(sibling_tokens.next(), Some(second_child_token));
    /// assert_eq!(sibling_tokens.next(), Some(first_child_token));
    /// assert!(sibling_tokens.next().is_none());
    /// ```
    pub fn preceding_siblings_tokens<'a, T>(self, tree: &'a Tree<T>)
        -> PrecedingSiblingTokens<'a, T> {
        let previous_sibling = match tree.get(self) {
            Some(n) => n.previous_sibling,
            None => panic!("Invalid token")
        };
        PrecedingSiblingTokens { tree, node_token: previous_sibling }
    }

    /// Returns an iterator of tokens of siblings following the current node.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = "Indo-European";
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    ///
    /// root_token.append(&mut tree, "Romance");
    /// let second_child_token = root_token.append(&mut tree, "Germanic");
    /// let third_child_token = root_token.append(&mut tree, "Slavic");
    /// let fourth_child_token = root_token.append(&mut tree, "Hellenic");
    ///
    /// let mut sibling_tokens = second_child_token.following_siblings_tokens(&tree);
    /// assert_eq!(sibling_tokens.next(), Some(third_child_token));
    /// assert_eq!(sibling_tokens.next(), Some(fourth_child_token));
    /// assert!(sibling_tokens.next().is_none());
    /// ```
    pub fn following_siblings_tokens<'a, T>(self, tree: &'a Tree<T>)
        -> FollowingSiblingTokens<'a, T> {
        let next_sibling = match tree.get(self) {
            Some(n) => n.next_sibling,
            None => panic!("Invalid token")
        };
        FollowingSiblingTokens { tree, node_token: next_sibling }
    }

    /// Returns an iterator of tokens of child nodes in the order of insertion.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = "Indo-European";
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    ///
    /// let first_child_token = root_token.append(&mut tree, "Romance");
    /// let second_child_token = root_token.append(&mut tree, "Germanic");
    /// let third_child_token = root_token.append(&mut tree, "Slavic");
    /// let fourth_child_token = root_token.append(&mut tree, "Hellenic");
    ///
    /// let mut children_tokens = root_token.children_tokens(&tree);
    /// assert_eq!(children_tokens.next(), Some(first_child_token));
    /// assert_eq!(children_tokens.next(), Some(second_child_token));
    /// assert_eq!(children_tokens.next(), Some(third_child_token));
    /// assert_eq!(children_tokens.next(), Some(fourth_child_token));
    /// assert!(children_tokens.next().is_none());
    /// ```
    pub fn children_tokens<'a, T>(self, tree: &'a Tree<T>)
        -> ChildrenTokens<'a, T> {
        let first_child = match tree.get(self) {
            Some(n) => n.first_child,
            None => panic!("Invalid token")
        };
        ChildrenTokens { tree, node_token: first_child }
    }

    /// Returns an iterator of references of ancestor nodes.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = "Indo-European";
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    ///
    /// let child_token = root_token.append(&mut tree, "Germanic");
    /// let grandchild_token = child_token.append(&mut tree, "Swedish");
    /// let mut ancestors = grandchild_token.ancestors(&tree);
    ///
    /// assert_eq!(ancestors.next().unwrap().data, "Germanic");
    /// assert_eq!(ancestors.next().unwrap().data, "Indo-European");
    /// assert!(ancestors.next().is_none());
    /// ```
    pub fn ancestors<'a, T>(self, tree: &'a Tree<T>) -> Ancestors<'a, T> {
        Ancestors { token_iter: self.ancestors_tokens(tree) }
    }

    /// Returns an iterator of references of sibling nodes preceding the current
    /// node.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = "Indo-European";
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    ///
    /// root_token.append(&mut tree, "Romance");
    /// root_token.append(&mut tree, "Germanic");
    /// let third_child_token = root_token.append(&mut tree, "Slavic");
    /// root_token.append(&mut tree, "Celtic");
    ///
    /// let mut siblings = third_child_token.preceding_siblings(&tree);
    /// assert_eq!(siblings.next().unwrap().data, "Germanic");
    /// assert_eq!(siblings.next().unwrap().data, "Romance");
    /// assert!(siblings.next().is_none());
    /// ```
    pub fn preceding_siblings<'a, T>(self, tree: &'a Tree<T>)
        -> PrecedingSiblings<'a, T> {
        PrecedingSiblings { token_iter: self.preceding_siblings_tokens(tree) }
    }

    /// Returns an iterator of references of sibling nodes following the current
    /// node.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = "Indo-European";
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    ///
    /// root_token.append(&mut tree, "Romance");
    /// let second_child_token = root_token.append(&mut tree, "Germanic");
    /// root_token.append(&mut tree, "Slavic");
    /// root_token.append(&mut tree, "Hellenic");
    ///
    /// let mut siblings = second_child_token.following_siblings(&tree);
    /// assert_eq!(siblings.next().unwrap().data, "Slavic");
    /// assert_eq!(siblings.next().unwrap().data, "Hellenic");
    /// assert!(siblings.next().is_none());
    /// ```
    pub fn following_siblings<'a, T>(self, tree: &'a Tree<T>)
        -> FollowingSiblings<'a, T> {
        FollowingSiblings { token_iter: self.following_siblings_tokens(tree) }
    }

    /// Returns an iterator of child node references in the order of insertion.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = "Indo-European";
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    ///
    /// let first_child_token = root_token.append(&mut tree, "Germanic");
    /// let second_child_token = root_token.append(&mut tree, "Romance");
    /// let third_child_token = root_token.append(&mut tree, "Slavic");
    /// let fourth_child_token = root_token.append(&mut tree, "Celtic");
    ///
    /// let mut children = root_token.children(&tree);
    /// assert_eq!(children.next().unwrap().data, "Germanic");
    /// assert_eq!(children.next().unwrap().data, "Romance");
    /// assert_eq!(children.next().unwrap().data, "Slavic");
    /// assert_eq!(children.next().unwrap().data, "Celtic");
    /// assert!(children.next().is_none());
    /// ```
    pub fn children<'a, T>(self, tree: &'a Tree<T>) -> Children<'a, T> {
        Children { token_iter: self.children_tokens(tree) }
    }

    /// Returns an iterator of mutable ancestor node references.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    ///
    /// let child_token = root_token.append(&mut tree, 2usize);
    /// let grandchild_token = child_token.append(&mut tree, 3usize);
    /// let great_grandchild_token = grandchild_token.append(&mut tree, 4usize);
    /// let ggreat_grandchild_token = great_grandchild_token.append(&mut tree, 5usize);
    ///
    /// for x in ggreat_grandchild_token.ancestors_mut(&mut tree) {
    ///     x.data += 2;
    /// }
    ///
    /// let mut ancestors = ggreat_grandchild_token.ancestors(&tree);
    /// assert_eq!(ancestors.next().unwrap().data, 6usize);
    /// assert_eq!(ancestors.next().unwrap().data, 5usize);
    /// assert_eq!(ancestors.next().unwrap().data, 4usize);
    /// assert_eq!(ancestors.next().unwrap().data, 3usize);
    /// assert!(ancestors.next().is_none());
    /// ```
    pub fn ancestors_mut<'a, T>(self, tree: &'a mut Tree<T>)
        -> AncestorsMut<'a, T> {
        AncestorsMut {
            tree: tree as *mut Tree<T>,
            node_token: Some(self),
            marker: PhantomData::default()
        }
    }

    /// Returns an iterator of mutable references of sibling nodes that follow
    /// the current node.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    ///
    /// root_token.append(&mut tree, 2usize);
    /// let second_child_token = root_token.append(&mut tree, 3usize);
    /// root_token.append(&mut tree, 4usize);
    /// root_token.append(&mut tree, 5usize);
    ///
    /// for x in second_child_token.following_siblings_mut(&mut tree) {
    ///     x.data += 2;
    /// }
    ///
    /// let mut children = root_token.children(&tree);
    /// assert_eq!(children.next().unwrap().data, 2usize);
    /// assert_eq!(children.next().unwrap().data, 3usize);
    /// assert_eq!(children.next().unwrap().data, 6usize);
    /// assert_eq!(children.next().unwrap().data, 7usize);
    /// assert!(children.next().is_none());
    /// ```
    pub fn following_siblings_mut<'a, T>(self, tree: &'a mut Tree<T>)
        -> FollowingSiblingsMut<'a, T> {
        let next_sibling = match tree.get(self) {
            Some(n) => n.next_sibling,
            None => panic!("Invalid token")
        };
        FollowingSiblingsMut {
            tree: tree as *mut Tree<T>,
            node_token: next_sibling,
            marker: PhantomData::default()
        }
    }

    /// Returns an iterator of mutable references of sibling nodes that precede
    /// the current node.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    ///
    /// root_token.append(&mut tree, 2usize);
    /// root_token.append(&mut tree, 3usize);
    /// root_token.append(&mut tree, 4usize);
    /// let fourth_child_token = root_token.append(&mut tree, 5usize);
    ///
    /// for x in fourth_child_token.preceding_siblings_mut(&mut tree) {
    ///     x.data += 5;
    /// }
    ///
    /// let mut children = root_token.children(&tree);
    /// assert_eq!(children.next().unwrap().data, 7usize);
    /// assert_eq!(children.next().unwrap().data, 8usize);
    /// assert_eq!(children.next().unwrap().data, 9usize);
    /// assert_eq!(children.next().unwrap().data, 5usize);
    /// assert!(children.next().is_none());
    /// ```
    pub fn preceding_siblings_mut<'a, T>(self, tree: &'a mut Tree<T>)
        -> PrecedingSiblingsMut<'a, T> {
        let previous_sibling = match tree.get(self) {
            Some(n) => n.previous_sibling,
            None => panic!("Invalid token")
        };
        PrecedingSiblingsMut {
            tree: tree as *mut Tree<T>,
            node_token: previous_sibling,
            marker: PhantomData::default()
        }
    }

    /// Returns an iterator of mutable references of child nodes in the order of
    /// insertion.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    ///
    /// root_token.append(&mut tree, 2usize);
    /// root_token.append(&mut tree, 3usize);
    /// let third_child_token = root_token.append(&mut tree, 4usize);
    /// root_token.append(&mut tree, 5usize);
    /// let grandchild = third_child_token.append(&mut tree, 10usize);
    ///
    /// for x in root_token.children_mut(&mut tree) {
    ///     x.data += 10;
    /// }
    ///
    /// let mut children = root_token.children(&tree);
    /// assert_eq!(children.next().unwrap().data, 12);
    /// assert_eq!(children.next().unwrap().data, 13);
    /// assert_eq!(children.next().unwrap().data, 14);
    /// assert_eq!(children.next().unwrap().data, 15);
    /// assert_eq!(tree.get(grandchild).unwrap().data, 10);
    /// assert!(children.next().is_none());
    /// ```
    pub fn children_mut<'a, T>(self, tree: &'a mut Tree<T>)
        -> ChildrenMut<'a, T> {
        let first_child = match tree.get(self) {
            Some(n) => n.first_child,
            None => panic!("Invalid token")
        };
        ChildrenMut {
            tree: tree as *mut Tree<T>,
            node_token: first_child,
            marker: PhantomData::default()
        }
    }

    /// Returns an iterator of tokens of subtree nodes of the given node.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    /// use atree::iter::TraversalOrder;
    ///
    /// let root_data = "Indo-European";
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    ///
    /// let first_child = root_token.append(&mut tree, "Romance");
    /// let second_child = root_token.append(&mut tree, "Germanic");
    /// let third_child = root_token.append(&mut tree, "Slavic");
    /// let first_grandchild = second_child.append(&mut tree, "English");
    /// let second_grandchild = second_child.append(&mut tree, "Icelandic");
    /// let fourth_child = root_token.append(&mut tree, "Celtic");
    ///
    /// let mut subtree = root_token.subtree_tokens(&tree, TraversalOrder::Pre);
    /// assert_eq!(subtree.next(), Some(root_token));
    /// assert_eq!(subtree.next(), Some(first_child));
    /// assert_eq!(subtree.next(), Some(second_child));
    /// assert_eq!(subtree.next(), Some(first_grandchild));
    /// assert_eq!(subtree.next(), Some(second_grandchild));
    /// assert_eq!(subtree.next(), Some(third_child));
    /// assert_eq!(subtree.next(), Some(fourth_child));
    /// assert!(subtree.next().is_none());
    ///
    /// let mut subtree = second_grandchild.subtree_tokens(&tree, TraversalOrder::Pre);
    /// assert_eq!(subtree.next(), Some(second_grandchild));
    /// assert!(subtree.next().is_none());
    /// ```
    pub fn subtree_tokens<'a, T>(self, tree: &'a Tree<T>, order: TraversalOrder)
        -> SubtreeTokens<'a, T> {
        let preord_tokens_next = |iter: &mut SubtreeTokens<T>| 
            depth_first_tokens_next(iter, preorder_next);
        let postord_tokens_next = |iter: &mut SubtreeTokens<T>| 
            depth_first_tokens_next(iter, postorder_next);
        match order {
            TraversalOrder::Pre => SubtreeTokens {
                tree,
                subtree_root: self,
                node_token: Some(self),
                branch: Branch::Child,
                curr_level: VecDeque::new(),  // unused field
                next_level: VecDeque::new(),  // unused field
                next: preord_tokens_next
            },
            TraversalOrder::Post => {
                let (node_token, branch) =
                    postorder_next(self, self, Branch::Child, tree);
                SubtreeTokens {
                    tree,
                    subtree_root: self,
                    node_token,
                    branch,
                    curr_level: VecDeque::new(),  // unused field
                    next_level: VecDeque::new(),  // unused field
                    next: postord_tokens_next
                }
            },
            TraversalOrder::Level => {
                SubtreeTokens {
                    tree,
                    subtree_root: self,  // unused field
                    node_token: None,  // unused field
                    branch: Branch::None,  // unused field
                    curr_level: std::iter::once(self).collect(),
                    next_level: VecDeque::new(),
                    next: breadth_first_tokens_next
                }
            }
        }
    }

    /// Returns an iterator of references of subtree nodes of the given node.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Tree;
    /// use atree::iter::TraversalOrder;
    ///
    /// let root_data = "Indo-European";
    /// let (mut tree, root_token) = Tree::with_data(root_data);
    ///
    /// root_token.append(&mut tree, "Romance");
    /// root_token.append(&mut tree, "Germanic");
    /// let third_child = root_token.append(&mut tree, "Slavic");
    /// root_token.append(&mut tree, "Celtic");
    /// third_child.append(&mut tree, "Polish");
    /// third_child.append(&mut tree, "Slovakian");
    ///
    /// let mut subtree = root_token.subtree(&tree, TraversalOrder::Pre);
    /// assert_eq!(subtree.next().unwrap().data, "Indo-European");
    /// assert_eq!(subtree.next().unwrap().data, "Romance");
    /// assert_eq!(subtree.next().unwrap().data, "Germanic");
    /// assert_eq!(subtree.next().unwrap().data, "Slavic");
    /// assert_eq!(subtree.next().unwrap().data, "Polish");
    /// assert_eq!(subtree.next().unwrap().data, "Slovakian");
    /// assert_eq!(subtree.next().unwrap().data, "Celtic");
    /// assert!(subtree.next().is_none());
    /// ```
    pub fn subtree<'a, T>(self, tree: &'a Tree<T>, order: TraversalOrder)
        -> Subtree<'a, T> {
        Subtree {
            tree,
            iter: self.subtree_tokens(tree, order)
        }
    }

    /// Returns an iterator of mutable references of subtree nodes of the given
    /// node.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node in the arena.
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
    /// root_token.append(&mut tree, 2usize);
    /// root_token.append(&mut tree, 3usize);
    /// let third_child = root_token.append(&mut tree, 4usize);
    /// root_token.append(&mut tree, 5usize);
    /// third_child.append(&mut tree, 10usize);
    /// third_child.append(&mut tree, 20usize);
    ///
    /// for x in root_token.subtree_mut(&mut tree, TraversalOrder::Pre) {
    ///     x.data += 100;
    /// }
    ///
    /// let mut subtree = root_token.subtree(&tree, TraversalOrder::Pre);
    /// assert_eq!(subtree.next().unwrap().data, 101);
    /// assert_eq!(subtree.next().unwrap().data, 102);
    /// assert_eq!(subtree.next().unwrap().data, 103);
    /// assert_eq!(subtree.next().unwrap().data, 104);
    /// assert_eq!(subtree.next().unwrap().data, 110);
    /// assert_eq!(subtree.next().unwrap().data, 120);
    /// assert_eq!(subtree.next().unwrap().data, 105);
    /// assert!(subtree.next().is_none());
    /// ```
    pub fn subtree_mut<'a, T>(self, tree: &'a mut Tree<T>, order: TraversalOrder)
        -> SubtreeMut<'a, T> {
        SubtreeMut {
            tree: tree as *mut Tree<T>,
            iter: self.subtree_tokens(tree, order),
            marker: PhantomData::default()
        }
    }

    /// Removes all descendants of the current node.
    pub (crate) fn remove_descendants<T>(self, tree: &mut Tree<T>) {
        // This will not silently fail since postorder_next will panic if self
        // isn't valid.  This won't do anything if self has no descendants, but
        // that's the intended behavior.
        if let (Some(mut token), mut branch) =
            postorder_next(self, self, Branch::Child, tree) {
            while branch != Branch::None {
                let (t, b) = postorder_next(token, self, branch, tree);
                tree.arena.remove(token);  // should not fail (not here anyway)
                token = t.unwrap();
                branch = b;
            }
            tree[self].first_child = None;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn subtree_tokens_postord() {
        let root_data = 1usize;
        let (mut tree, root_token) = Tree::with_data(root_data);
       
        let first_child = root_token.append(&mut tree, 2usize);
        let second_child = root_token.append(&mut tree, 3usize);
        let third_child = root_token.append(&mut tree, 4usize);
        let first_grandchild = first_child.append(&mut tree, 0usize);
        let fourth_child = root_token.append(&mut tree, 5usize);
        let second_grandchild = second_child.append(&mut tree, 10usize);
        let third_grandchild = second_child.append(&mut tree, 20usize);
        let great_grandchild = third_grandchild.append(&mut tree, 20usize);
       
        let mut subtree = root_token.subtree_tokens(&tree, TraversalOrder::Post);
        assert_eq!(subtree.next(), Some(first_grandchild));
        assert_eq!(subtree.next(), Some(first_child));
        assert_eq!(subtree.next(), Some(second_grandchild));
        assert_eq!(subtree.next(), Some(great_grandchild));
        assert_eq!(subtree.next(), Some(third_grandchild));
        assert_eq!(subtree.next(), Some(second_child));
        assert_eq!(subtree.next(), Some(third_child));
        assert_eq!(subtree.next(), Some(fourth_child));
        assert_eq!(subtree.next(), Some(root_token));
        assert!(subtree.next().is_none());
       
        let mut subtree = great_grandchild.subtree_tokens(&tree, TraversalOrder::Post);
        assert_eq!(subtree.next(), Some(great_grandchild));
        assert!(subtree.next().is_none());
    }

    #[test]
    fn subtree_tokens_levelord() {
        let root_data = 1usize;
        let (mut tree, root_token) = Tree::with_data(root_data);
       
        let first_child = root_token.append(&mut tree, 2usize);
        let second_child = root_token.append(&mut tree, 3usize);
        let third_child = root_token.append(&mut tree, 4usize);
        let first_grandchild = second_child.append(&mut tree, 10usize);
        let second_grandchild = second_child.append(&mut tree, 20usize);
        let fourth_child = root_token.append(&mut tree, 5usize);
       
        let mut subtree = root_token.subtree_tokens(&tree, TraversalOrder::Level);
        assert_eq!(subtree.next(), Some(root_token));
        assert_eq!(subtree.next(), Some(first_child));
        assert_eq!(subtree.next(), Some(second_child));
        assert_eq!(subtree.next(), Some(third_child));
        assert_eq!(subtree.next(), Some(fourth_child));
        assert_eq!(subtree.next(), Some(first_grandchild));
        assert_eq!(subtree.next(), Some(second_grandchild));
        assert!(subtree.next().is_none());
       
        let mut subtree = second_grandchild.subtree_tokens(&tree, TraversalOrder::Level);
        assert_eq!(subtree.next(), Some(second_grandchild));
        assert!(subtree.next().is_none());
    }

    #[test]
    fn subtree_postord() {
        let root_data = "Indo-European";
        let (mut tree, root_token) = Tree::with_data(root_data);
       
        root_token.append(&mut tree, "Romance");
        root_token.append(&mut tree, "Germanic");
        let third_child = root_token.append(&mut tree, "Celtic");
        root_token.append(&mut tree, "Slavic");
        third_child.append(&mut tree, "Ulster");
        third_child.append(&mut tree, "Gaulish");
       
        let mut subtree = root_token.subtree(&tree, TraversalOrder::Post);
        assert_eq!(subtree.next().unwrap().data, "Romance");
        assert_eq!(subtree.next().unwrap().data, "Germanic");
        assert_eq!(subtree.next().unwrap().data, "Ulster");
        assert_eq!(subtree.next().unwrap().data, "Gaulish");
        assert_eq!(subtree.next().unwrap().data, "Celtic");
        assert_eq!(subtree.next().unwrap().data, "Slavic");
        assert_eq!(subtree.next().unwrap().data, "Indo-European");
        assert!(subtree.next().is_none());
    }

    #[test]
    fn subtree_levelord() {
        let root_data = "Indo-European";
        let (mut tree, root_token) = Tree::with_data(root_data);
       
        root_token.append(&mut tree, "Romance");
        root_token.append(&mut tree, "Germanic");
        let third_child = root_token.append(&mut tree, "Slavic");
        root_token.append(&mut tree, "Hellenic");
        third_child.append(&mut tree, "Russian");
        third_child.append(&mut tree, "Ukrainian");
       
        let mut subtree = root_token.subtree(&tree, TraversalOrder::Level);
        assert_eq!(subtree.next().unwrap().data, "Indo-European");
        assert_eq!(subtree.next().unwrap().data, "Romance");
        assert_eq!(subtree.next().unwrap().data, "Germanic");
        assert_eq!(subtree.next().unwrap().data, "Slavic");
        assert_eq!(subtree.next().unwrap().data, "Hellenic");
        assert_eq!(subtree.next().unwrap().data, "Russian");
        assert_eq!(subtree.next().unwrap().data, "Ukrainian");
        assert!(subtree.next().is_none());
    }

    #[test]
    fn subtree_postord_mut() {
        let root_data = 1usize;
        let (mut tree, root_token) = Tree::with_data(root_data);
       
        root_token.append(&mut tree, 2usize);
        root_token.append(&mut tree, 3usize);
        let third_child = root_token.append(&mut tree, 4usize);
        root_token.append(&mut tree, 5usize);
        third_child.append(&mut tree, 10usize);
        third_child.append(&mut tree, 20usize);
       
        for x in root_token.subtree_mut(&mut tree, TraversalOrder::Post) {
            x.data += 100;
        }
       
        let mut subtree = root_token.subtree(&tree, TraversalOrder::Post);
        assert_eq!(subtree.next().unwrap().data, 102);
        assert_eq!(subtree.next().unwrap().data, 103);
        assert_eq!(subtree.next().unwrap().data, 110);
        assert_eq!(subtree.next().unwrap().data, 120);
        assert_eq!(subtree.next().unwrap().data, 104);
        assert_eq!(subtree.next().unwrap().data, 105);
        assert_eq!(subtree.next().unwrap().data, 101);
        assert!(subtree.next().is_none());
    }

    #[test]
    fn subtree_levelord_mut() {
        let root_data = 1usize;
        let (mut tree, root_token) = Tree::with_data(root_data);
       
        root_token.append(&mut tree, 2usize);
        root_token.append(&mut tree, 3usize);
        let third_child = root_token.append(&mut tree, 4usize);
        root_token.append(&mut tree, 5usize);
        third_child.append(&mut tree, 10usize);
        third_child.append(&mut tree, 20usize);
       
        for x in root_token.subtree_mut(&mut tree, TraversalOrder::Level) {
            x.data += 100;
        }
       
        let mut subtree = root_token.subtree(&tree, TraversalOrder::Level);
        assert_eq!(subtree.next().unwrap().data, 101);
        assert_eq!(subtree.next().unwrap().data, 102);
        assert_eq!(subtree.next().unwrap().data, 103);
        assert_eq!(subtree.next().unwrap().data, 104);
        assert_eq!(subtree.next().unwrap().data, 105);
        assert_eq!(subtree.next().unwrap().data, 110);
        assert_eq!(subtree.next().unwrap().data, 120);
        assert!(subtree.next().is_none());
    }

    #[test]
    fn remove_descendants() {
        let root_data = 1usize;
        let (mut tree, root_token) = Tree::with_data(root_data);

        let first_child = root_token.append(&mut tree, 2usize);
        let second_child = root_token.append(&mut tree, 3usize);
        let third_child = root_token.append(&mut tree, 4usize);
        let fourth_child = root_token.append(&mut tree, 5usize);
        let grandchild_1 = third_child.append(&mut tree, 10usize);
        third_child.append(&mut tree, 20usize);
        grandchild_1.append(&mut tree, 100usize);

        assert_eq!(tree.node_count(), 8);

        third_child.remove_descendants(&mut tree);

        let mut subtree = root_token.subtree_tokens(&tree, TraversalOrder::Pre);
        assert_eq!(subtree.next(), Some(root_token));
        assert_eq!(subtree.next(), Some(first_child));
        assert_eq!(subtree.next(), Some(second_child));
        assert_eq!(subtree.next(), Some(third_child));
        assert_eq!(subtree.next(), Some(fourth_child));
        assert!(subtree.next().is_none());

        println!("{:?}", tree.arena);
        assert_eq!(tree.node_count(), 5);
    }
}
