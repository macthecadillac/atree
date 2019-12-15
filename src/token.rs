use std::collections::VecDeque;
use std::marker::PhantomData;

use crate::arena::Arena;
use crate::iter::*;
use crate::node::Node;
use crate::tree::Tree;

/// A `Token` is a handle to a node on the tree.
#[derive(Clone, Copy, Eq, PartialEq, Default, Debug)]
pub struct Token{
    pub (crate) index: usize
}

impl Token {
    /// Creates a new node with the given data and append to the given node.
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
    /// let next_node_token = root_token.append(&mut tree, 2usize);
    /// next_node_token.append(&mut tree, 3usize);
    /// let mut descendants = root_token.descendants_preord(&tree);
    ///
    /// assert_eq!(descendants.next().unwrap().data, 2usize);
    /// assert_eq!(descendants.next().unwrap().data, 3usize);
    /// ```
    // TODO: find ways to put this under impl Node<T>
    pub fn append<T>(self, tree: &mut Tree<T>, data: T) -> Token {
        fn find_head<T>(arena: &mut Arena<T>) -> Token {
            match arena.head() {
                Some(head) => Token{ index: head },
                None => {
                    arena.reserve(arena.len());
                    find_head(arena)
                }
            }
        }

        let new_node_token = find_head(&mut tree.arena);
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
    ///
    /// let root_data = "John";
    /// let (mut tree1, root_token) = Tree::with_root(root_data);
    ///
    /// let node1 = root_token.append(&mut tree1, "Juan");
    /// let node2 = root_token.append(&mut tree1, "Giovanni");
    /// let grandchild1 = node1.append(&mut tree1, "Ivan");
    /// let grandchild2 = node2.append(&mut tree1, "Johann");
    ///
    /// let mut tree2 = tree1.clone();
    ///
    /// // append "node1" from tree2 under "node2" in tree1
    /// node2.append_subtree(&mut tree1, node1, &mut tree2);
    /// let mut descendants = node2.descendants_preord(&tree1);
    ///
    /// assert_eq!(descendants.next().unwrap().data, "Johann");
    /// assert_eq!(descendants.next().unwrap().data, "Juan");
    /// assert_eq!(descendants.next().unwrap().data, "Ivan");
    /// assert!(descendants.next().is_none());
    /// ```
    pub fn append_subtree<T>(self, self_tree: &mut Tree<T>,
                             other_token: Token, other_tree: &Tree<T>)
        where T: Clone {
        let data = match other_tree.get(other_token) {
            Some(node) => node.data.clone(),
            None => panic!("Invalid token")
        };
        let new_node_token = self.append(self_tree, data);
        for child_token in other_token.children_tokens(&other_tree) {
            new_node_token.append_subtree(self_tree, child_token, other_tree)
        }
    }

    /// Returns an iterator of tokens of ancestor nodes.
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
    /// let next_node_token = root_token.append(&mut tree, 2usize);
    /// let third_node_token = next_node_token.append(&mut tree, 3usize);
    /// let mut ancestors_tokens = third_node_token.ancestors_tokens(&tree);
    ///
    /// assert_eq!(ancestors_tokens.next(), Some(next_node_token));
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
    /// let first_child_token = root_token.append(&mut tree, 2usize);
    /// let second_child_token = root_token.append(&mut tree, 3usize);
    /// let third_child_token = root_token.append(&mut tree, 4usize);
    /// root_token.append(&mut tree, 5usize);
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
    /// root_token.append(&mut tree, 2usize);
    /// let second_child_token = root_token.append(&mut tree, 3usize);
    /// let third_child_token = root_token.append(&mut tree, 4usize);
    /// let fourth_child_token = root_token.append(&mut tree, 5usize);
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
    /// let first_child_token = root_token.append(&mut tree, 2usize);
    /// let second_child_token = root_token.append(&mut tree, 3usize);
    /// let third_child_token = root_token.append(&mut tree, 4usize);
    /// let fourth_child_token = root_token.append(&mut tree, 5usize);
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

    /// Returns an iterator of ancestor nodes.
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
    /// let next_node_token = root_token.append(&mut tree, 2usize);
    /// let third_node_token = next_node_token.append(&mut tree, 3usize);
    /// let mut ancestors = third_node_token.ancestors(&tree);
    ///
    /// assert_eq!(ancestors.next().unwrap().data, 2usize);
    /// assert_eq!(ancestors.next().unwrap().data, 1usize);
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
    /// root_token.append(&mut tree, 2usize);
    /// root_token.append(&mut tree, 3usize);
    /// let third_child_token = root_token.append(&mut tree, 4usize);
    /// root_token.append(&mut tree, 5usize);
    ///
    /// let mut siblings = third_child_token.preceding_siblings(&tree);
    /// assert_eq!(siblings.next().unwrap().data, 3usize);
    /// assert_eq!(siblings.next().unwrap().data, 2usize);
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
    /// root_token.append(&mut tree, 2usize);
    /// let second_child_token = root_token.append(&mut tree, 3usize);
    /// root_token.append(&mut tree, 4usize);
    /// root_token.append(&mut tree, 5usize);
    ///
    /// let mut siblings = second_child_token.following_siblings(&tree);
    /// assert_eq!(siblings.next().unwrap().data, 4usize);
    /// assert_eq!(siblings.next().unwrap().data, 5usize);
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
    /// let first_child_token = root_token.append(&mut tree, 2usize);
    /// let second_child_token = root_token.append(&mut tree, 3usize);
    /// let third_child_token = root_token.append(&mut tree, 4usize);
    /// let fourth_child_token = root_token.append(&mut tree, 5usize);
    ///
    /// let mut children = root_token.children(&tree);
    /// assert_eq!(children.next().unwrap().data, 2usize);
    /// assert_eq!(children.next().unwrap().data, 3usize);
    /// assert_eq!(children.next().unwrap().data, 4usize);
    /// assert_eq!(children.next().unwrap().data, 5usize);
    /// assert!(children.next().is_none());
    /// ```
    pub fn children<'a, T>(self, tree: &'a Tree<T>) -> Children<'a, T> {
        Children { token_iter: self.children_tokens(tree) }
    }

    /// Returns an iterator of mutable ancestor node references.
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

    /// Returns an iterator of mutable references for sibling nodes that follow
    /// the current node.
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

    /// Returns an iterator of mutable references for sibling nodes that precede
    /// the current node.
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

    /// Returns an iterator of tokens of descendant nodes in pre-order.
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
    /// let first_child = root_token.append(&mut tree, 2usize);
    /// let second_child = root_token.append(&mut tree, 3usize);
    /// let third_child = root_token.append(&mut tree, 4usize);
    /// let first_grandchild = second_child.append(&mut tree, 10usize);
    /// let second_grandchild = second_child.append(&mut tree, 20usize);
    /// let fourth_child = root_token.append(&mut tree, 5usize);
    ///
    /// let mut descendants = root_token.descendants_tokens_preord(&tree);
    /// assert_eq!(descendants.next(), Some(first_child));
    /// assert_eq!(descendants.next(), Some(second_child));
    /// assert_eq!(descendants.next(), Some(first_grandchild));
    /// assert_eq!(descendants.next(), Some(second_grandchild));
    /// assert_eq!(descendants.next(), Some(third_child));
    /// assert_eq!(descendants.next(), Some(fourth_child));
    /// assert!(descendants.next().is_none());
    ///
    /// let mut descendants = second_child.descendants_tokens_preord(&tree);
    /// assert_eq!(descendants.next(), Some(first_grandchild));
    /// assert_eq!(descendants.next(), Some(second_grandchild));
    /// assert!(descendants.next().is_none());
    /// ```
    pub fn descendants_tokens_preord<'a, T>(self, tree: &'a Tree<T>)
        -> DescendantsTokensPreord<'a, T> {
        let first_child = match tree.get(self) {
            Some(n) => n.first_child,
            None => panic!("Invalid token")
        };
        DescendantsTokensPreord {
            tree,
            subtree_root: self,
            node_token: first_child,
            branch: crate::iter::Branch::Child
        }
    }

    /// Returns an iterator of references of descendant nodes in pre-order.
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
    /// root_token.append(&mut tree, 2usize);
    /// root_token.append(&mut tree, 3usize);
    /// let third_child = root_token.append(&mut tree, 4usize);
    /// root_token.append(&mut tree, 5usize);
    /// third_child.append(&mut tree, 10usize);
    /// third_child.append(&mut tree, 20usize);
    ///
    /// let mut descendants = root_token.descendants_preord(&tree);
    /// assert_eq!(descendants.next().unwrap().data, 2);
    /// assert_eq!(descendants.next().unwrap().data, 3);
    /// assert_eq!(descendants.next().unwrap().data, 4);
    /// assert_eq!(descendants.next().unwrap().data, 10);
    /// assert_eq!(descendants.next().unwrap().data, 20);
    /// assert_eq!(descendants.next().unwrap().data, 5);
    /// assert!(descendants.next().is_none());
    /// ```
    pub fn descendants_preord<'a, T>(self, tree: &'a Tree<T>)
        -> DescendantsPreord<'a, T> {
        DescendantsPreord {
            tree,
            descendants: self.descendants_tokens_preord(tree)
        }
    }

    /// Returns an iterator of mutable references of descendant nodes in
    /// pre-order.
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
    /// root_token.append(&mut tree, 2usize);
    /// root_token.append(&mut tree, 3usize);
    /// let third_child = root_token.append(&mut tree, 4usize);
    /// root_token.append(&mut tree, 5usize);
    /// third_child.append(&mut tree, 10usize);
    /// third_child.append(&mut tree, 20usize);
    ///
    /// for x in root_token.descendants_mut_preord(&mut tree) {
    ///     x.data += 100;
    /// }
    ///
    /// let mut descendants = root_token.descendants_preord(&tree);
    /// assert_eq!(descendants.next().unwrap().data, 102);
    /// assert_eq!(descendants.next().unwrap().data, 103);
    /// assert_eq!(descendants.next().unwrap().data, 104);
    /// assert_eq!(descendants.next().unwrap().data, 110);
    /// assert_eq!(descendants.next().unwrap().data, 120);
    /// assert_eq!(descendants.next().unwrap().data, 105);
    /// assert!(descendants.next().is_none());
    /// ```
    pub fn descendants_mut_preord<'a, T>(self, tree: &'a mut Tree<T>)
        -> DescendantsMutPreord<'a, T> {
        DescendantsMutPreord {
            tree: tree as *mut Tree<T>,
            descendants: self.descendants_tokens_preord(tree),
            marker: PhantomData::default()
        }
    }

    /// Returns an iterator of tokens of descendant nodes in post-order.
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
    /// let first_child = root_token.append(&mut tree, 2usize);
    /// let second_child = root_token.append(&mut tree, 3usize);
    /// let third_child = root_token.append(&mut tree, 4usize);
    /// let first_grandchild = second_child.append(&mut tree, 10usize);
    /// let second_grandchild = second_child.append(&mut tree, 20usize);
    /// let great_grandchild = second_grandchild.append(&mut tree, 20usize);
    /// let fourth_child = root_token.append(&mut tree, 5usize);
    ///
    /// let mut descendants = root_token.descendants_tokens_postord(&tree);
    /// assert_eq!(descendants.next(), Some(first_child));
    /// assert_eq!(descendants.next(), Some(first_grandchild));
    /// assert_eq!(descendants.next(), Some(great_grandchild));
    /// assert_eq!(descendants.next(), Some(second_grandchild));
    /// assert_eq!(descendants.next(), Some(second_child));
    /// assert_eq!(descendants.next(), Some(third_child));
    /// assert_eq!(descendants.next(), Some(fourth_child));
    /// assert!(descendants.next().is_none());
    ///
    /// let mut descendants = second_child.descendants_tokens_postord(&tree);
    /// assert_eq!(descendants.next(), Some(first_grandchild));
    /// assert_eq!(descendants.next(), Some(great_grandchild));
    /// assert_eq!(descendants.next(), Some(second_grandchild));
    /// assert!(descendants.next().is_none());
    /// ```
    pub fn descendants_tokens_postord<'a, T>(self, tree: &'a Tree<T>)
        -> DescendantsTokensPostord<'a, T> {
        let first_child = match tree.get(self) {
            Some(n) => n.first_child,
            None => panic!("Invalid token")
        };
        DescendantsTokensPostord {
            tree,
            subtree_root: self,
            node_token: first_child,
            branch: crate::iter::Branch::Child
        }
    }

    /// Returns an iterator of references of descendant nodes in post-order.
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
    /// root_token.append(&mut tree, 2usize);
    /// root_token.append(&mut tree, 3usize);
    /// let third_child = root_token.append(&mut tree, 4usize);
    /// root_token.append(&mut tree, 5usize);
    /// third_child.append(&mut tree, 10usize);
    /// third_child.append(&mut tree, 20usize);
    ///
    /// let mut descendants = root_token.descendants_postord(&tree);
    /// assert_eq!(descendants.next().unwrap().data, 2);
    /// assert_eq!(descendants.next().unwrap().data, 3);
    /// assert_eq!(descendants.next().unwrap().data, 10);
    /// assert_eq!(descendants.next().unwrap().data, 20);
    /// assert_eq!(descendants.next().unwrap().data, 4);
    /// assert_eq!(descendants.next().unwrap().data, 5);
    /// assert!(descendants.next().is_none());
    /// ```
    pub fn descendants_postord<'a, T>(self, tree: &'a Tree<T>)
        -> DescendantsPostord<'a, T> {
        DescendantsPostord {
            tree,
            descendants: self.descendants_tokens_postord(tree)
        }
    }

    /// Returns an iterator of mutable references of descendant nodes in
    /// post-order.
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
    /// root_token.append(&mut tree, 2usize);
    /// root_token.append(&mut tree, 3usize);
    /// let third_child = root_token.append(&mut tree, 4usize);
    /// root_token.append(&mut tree, 5usize);
    /// third_child.append(&mut tree, 10usize);
    /// third_child.append(&mut tree, 20usize);
    ///
    /// for x in root_token.descendants_mut_postord(&mut tree) {
    ///     x.data += 100;
    /// }
    ///
    /// let mut descendants = root_token.descendants_postord(&tree);
    /// assert_eq!(descendants.next().unwrap().data, 102);
    /// assert_eq!(descendants.next().unwrap().data, 103);
    /// assert_eq!(descendants.next().unwrap().data, 110);
    /// assert_eq!(descendants.next().unwrap().data, 120);
    /// assert_eq!(descendants.next().unwrap().data, 104);
    /// assert_eq!(descendants.next().unwrap().data, 105);
    /// assert!(descendants.next().is_none());
    /// ```
    pub fn descendants_mut_postord<'a, T>(self, tree: &'a mut Tree<T>)
        -> DescendantsMutPostord<'a, T> {
        DescendantsMutPostord {
            tree: tree as *mut Tree<T>,
            descendants: self.descendants_tokens_postord(tree),
            marker: PhantomData::default()
        }
    }

    /// Returns an iterator of tokens of descendant nodes in level-order.
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
    /// let first_child = root_token.append(&mut tree, 2usize);  // 1
    /// let second_child = root_token.append(&mut tree, 3usize);  // 2
    /// let third_child = root_token.append(&mut tree, 4usize);  // 3
    /// let first_grandchild = second_child.append(&mut tree, 10usize);  // 4
    /// let second_grandchild = second_child.append(&mut tree, 20usize);  // 5
    /// let fourth_child = root_token.append(&mut tree, 5usize);  // 6
    ///
    /// let mut descendants = root_token.descendants_tokens_levelord(&tree);
    /// assert_eq!(descendants.next(), Some(first_child));  // 1
    /// assert_eq!(descendants.next(), Some(second_child));  // 2
    /// assert_eq!(descendants.next(), Some(third_child));  // 3
    /// assert_eq!(descendants.next(), Some(fourth_child));  //4
    /// assert_eq!(descendants.next(), Some(first_grandchild));  // 5
    /// assert_eq!(descendants.next(), Some(second_grandchild));  // 6
    /// assert!(descendants.next().is_none());
    /// ```
    pub fn descendants_tokens_levelord<'a, T>(self, tree: &'a Tree<T>)
        -> DescendantsTokensLevelord<'a, T> {
        DescendantsTokensLevelord {
            tree,
            curr_level: self.children_tokens(tree).collect(),
            next_level: VecDeque::new()
        }
    }

    /// Returns an iterator of references of descendant nodes in level-order.
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
    /// root_token.append(&mut tree, 2usize);
    /// root_token.append(&mut tree, 3usize);
    /// let third_child = root_token.append(&mut tree, 4usize);
    /// root_token.append(&mut tree, 5usize);
    /// third_child.append(&mut tree, 10usize);
    /// third_child.append(&mut tree, 20usize);
    ///
    /// let mut descendants = root_token.descendants_levelord(&tree);
    /// assert_eq!(descendants.next().unwrap().data, 2);
    /// assert_eq!(descendants.next().unwrap().data, 3);
    /// assert_eq!(descendants.next().unwrap().data, 4);
    /// assert_eq!(descendants.next().unwrap().data, 5);
    /// assert_eq!(descendants.next().unwrap().data, 10);
    /// assert_eq!(descendants.next().unwrap().data, 20);
    /// assert!(descendants.next().is_none());
    /// ```
    pub fn descendants_levelord<'a, T>(self, tree: &'a Tree<T>)
        -> DescendantsLevelord<'a, T> {
        DescendantsLevelord {
            tree,
            descendants: self.descendants_tokens_levelord(tree)
        }
    }

    /// Returns an iterator of mutable references of descendant nodes in
    /// level-order.
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
    /// root_token.append(&mut tree, 2usize);
    /// root_token.append(&mut tree, 3usize);
    /// let third_child = root_token.append(&mut tree, 4usize);
    /// root_token.append(&mut tree, 5usize);
    /// third_child.append(&mut tree, 10usize);
    /// third_child.append(&mut tree, 20usize);
    ///
    /// for x in root_token.descendants_mut_levelord(&mut tree) {
    ///     x.data += 100;
    /// }
    ///
    /// let mut descendants = root_token.descendants_levelord(&tree);
    /// assert_eq!(descendants.next().unwrap().data, 102);
    /// assert_eq!(descendants.next().unwrap().data, 103);
    /// assert_eq!(descendants.next().unwrap().data, 104);
    /// assert_eq!(descendants.next().unwrap().data, 105);
    /// assert_eq!(descendants.next().unwrap().data, 110);
    /// assert_eq!(descendants.next().unwrap().data, 120);
    /// assert!(descendants.next().is_none());
    /// ```
    pub fn descendants_mut_levelord<'a, T>(self, tree: &'a mut Tree<T>)
        -> DescendantsMutLevelord<'a, T> {
        DescendantsMutLevelord {
            tree: tree as *mut Tree<T>,
            descendants: self.descendants_tokens_levelord(tree),
            marker: PhantomData::default()
        }
    }

    /// Removes all descendants of the current node.
    pub (crate) fn remove_descendants<T>(self, tree: &mut Tree<T>) {
        match tree.get(self) {
            None => panic!("Invalid token"),
            Some(node) => match node.first_child {
                None => (),
                Some(child) => {
                    let (t, mut branch) =
                        postorder_next(child, self, Branch::Child, tree);
                    if let Some(mut token) = t {
                        loop {
                            let (t, b) = postorder_next(token, self, branch, tree);
                            tree.arena.remove(token);
                            match t {
                                None => break,
                                Some(t) => {
                                    token = t;
                                    branch = b;
                                }
                            }
                        }
                    }
                    tree.arena.remove(child);
                    tree[self].first_child = None
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn remove_descendants() {
        let root_data = 1usize;
        let (mut tree, root_token) = Tree::with_root(root_data);

        let first_child = root_token.append(&mut tree, 2usize);
        let second_child = root_token.append(&mut tree, 3usize);
        let third_child = root_token.append(&mut tree, 4usize);
        let fourth_child = root_token.append(&mut tree, 5usize);
        let grandchild_1 = third_child.append(&mut tree, 10usize);
        third_child.append(&mut tree, 20usize);
        grandchild_1.append(&mut tree, 100usize);

        assert_eq!(tree.node_count(), 8);

        third_child.remove_descendants(&mut tree);

        let mut descendants = root_token.descendants_tokens_preord(&tree);
        assert_eq!(descendants.next(), Some(first_child));
        assert_eq!(descendants.next(), Some(second_child));
        assert_eq!(descendants.next(), Some(third_child));
        assert_eq!(descendants.next(), Some(fourth_child));
        assert!(descendants.next().is_none());

        assert_eq!(tree.node_count(), 5);
    }
}
