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
    /// use itree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
    ///
    /// let next_node_token = root_token.append(&mut tree, 2usize);
    /// next_node_token.append(&mut tree, 3usize);
    /// let mut descendents = root_token.descendents(&tree);
    ///
    /// assert_eq!(descendents.next().unwrap().data, 2usize);
    /// assert_eq!(descendents.next().unwrap().data, 3usize);
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

    /// Returns an iterator of tokens of ancestor nodes.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node on the tree.
    ///
    /// # Examples:
    ///
    /// ```
    /// use itree::Tree;
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
    /// use itree::Tree;
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
    /// use itree::Tree;
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
    /// use itree::Tree;
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
    /// use itree::Tree;
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
    /// use itree::Tree;
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
    /// use itree::Tree;
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
    /// use itree::Tree;
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
    /// use itree::Tree;
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
    /// use itree::Tree;
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
    /// use itree::Tree;
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
    /// use itree::Tree;
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

    /// Returns an iterator of tokens of descendent nodes (in pre-order).
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node on the tree.
    ///
    /// # Examples:
    ///
    /// ```
    /// use itree::Tree;
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
    /// let mut descendents = root_token.descendents_tokens(&tree);
    /// assert_eq!(descendents.next(), Some(first_child));
    /// assert_eq!(descendents.next(), Some(second_child));
    /// assert_eq!(descendents.next(), Some(first_grandchild));
    /// assert_eq!(descendents.next(), Some(second_grandchild));
    /// assert_eq!(descendents.next(), Some(third_child));
    /// assert_eq!(descendents.next(), Some(fourth_child));
    /// assert!(descendents.next().is_none());
    ///
    /// let mut descendents = second_child.descendents_tokens(&tree);
    /// assert_eq!(descendents.next(), Some(first_grandchild));
    /// assert_eq!(descendents.next(), Some(second_grandchild));
    /// assert!(descendents.next().is_none());
    /// ```
    pub fn descendents_tokens<T>(self, tree: &Tree<T>) -> DescendentTokens {
        fn aux<T>(token: Token, tree: &Tree<T>, acc: &mut Vec<Token>) {
            for child in token.children_tokens(tree) {
                acc.push(child);
                aux(child, tree, acc)
            }
        }

        let mut nodes = Vec::new();
        aux(self, tree, &mut nodes);
        DescendentTokens { nodes, ptr: 0 }
    }

    /// Returns an iterator of references of descendent nodes (in pre-order).
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node on the tree.
    ///
    /// # Examples:
    ///
    /// ```
    /// use itree::Tree;
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
    /// let mut descendents = root_token.descendents(&tree);
    /// assert_eq!(descendents.next().unwrap().data, 2);
    /// assert_eq!(descendents.next().unwrap().data, 3);
    /// assert_eq!(descendents.next().unwrap().data, 4);
    /// assert_eq!(descendents.next().unwrap().data, 10);
    /// assert_eq!(descendents.next().unwrap().data, 20);
    /// assert_eq!(descendents.next().unwrap().data, 5);
    /// assert!(descendents.next().is_none());
    /// ```
    pub fn descendents<'a, T>(self, tree: &'a Tree<T>) -> Descendents<'a, T> {
        Descendents { tree, descendents: self.descendents_tokens(tree) }
    }

    /// Removes all descendents of the current node.
    pub (crate) fn remove_descendents<T>(self, tree: &mut Tree<T>) {
        for node_token in self.descendents_tokens(tree) {
            tree.arena.remove(node_token);
        }
        match tree.get_mut(self) {
            Some(node) => node.first_child = None,
            None => panic!("Invalid token")
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn remove_descendents() {
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

        third_child.remove_descendents(&mut tree);

        let mut descendents = root_token.descendents_tokens(&tree);
        assert_eq!(descendents.next(), Some(first_child));
        assert_eq!(descendents.next(), Some(second_child));
        assert_eq!(descendents.next(), Some(third_child));
        assert_eq!(descendents.next(), Some(fourth_child));
        assert!(descendents.next().is_none());

        assert_eq!(tree.node_count(), 5);
    }
}
