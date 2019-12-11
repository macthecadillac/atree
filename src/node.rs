use crate::tree::Tree;
use crate::token::Token;
use crate::iter::*;

#[derive(Default, Debug)]
pub struct Node<T> {
    pub data: T,
    pub (crate) token: Token,
    pub (crate) parent: Option<Token>,
    pub (crate) previous_sibling: Option<Token>,
    pub (crate) next_sibling: Option<Token>,
    pub (crate) first_child: Option<Token>,
}

impl<T> Node<T> {
    /// Returns the token of the given node.
    pub fn token(&self) -> Token { self.token }

    /// Returns an iterator of tokens of ancestor nodes.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node on the tree.
    ///
    /// # Examples:
    ///
    /// ```
    /// use arena_tree::Tree;
    /// use arena_tree::Node;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
    ///
    /// let next_node_token = root_token.append(&mut tree, 2usize);
    /// let third_node_token = next_node_token.append(&mut tree, 3usize);
    ///
    /// let third_node = &tree[third_node_token];
    /// let mut ancestors_tokens = third_node.ancestors_tokens(&tree);
    ///
    /// assert_eq!(ancestors_tokens.next(), Some(next_node_token));
    /// assert_eq!(ancestors_tokens.next(), Some(root_token));
    /// assert!(ancestors_tokens.next().is_none());
    /// ```
    pub fn ancestors_tokens<'a>(&self, arena: &'a Tree<T>)
        -> AncestorTokens<'a, T> {
        self.token.ancestors_tokens(arena)
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
    /// use arena_tree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
    ///
    /// let first_child_token = root_token.append(&mut tree, 2usize);
    /// let second_child_token = root_token.append(&mut tree, 3usize);
    /// let third_child_token = root_token.append(&mut tree, 4usize);
    /// root_token.append(&mut tree, 5usize);
    ///
    /// let third_child = &tree[third_child_token];
    /// let mut sibling_tokens = third_child.preceding_siblings_tokens(&tree);
    /// assert_eq!(sibling_tokens.next(), Some(second_child_token));
    /// assert_eq!(sibling_tokens.next(), Some(first_child_token));
    /// assert!(sibling_tokens.next().is_none());
    /// ```
    pub fn preceding_siblings_tokens<'a>(&self, arena: &'a Tree<T>)
        -> PrecedingSiblingTokens<'a, T> {
        self.token.preceding_siblings_tokens(arena)
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
    /// use arena_tree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
    ///
    /// root_token.append(&mut tree, 2usize);
    /// let second_child_token = root_token.append(&mut tree, 3usize);
    /// let third_child_token = root_token.append(&mut tree, 4usize);
    /// let fourth_child_token = root_token.append(&mut tree, 5usize);
    ///
    /// let second_child = &tree[second_child_token];
    /// let mut sibling_tokens = second_child.following_siblings_tokens(&tree);
    /// assert_eq!(sibling_tokens.next(), Some(third_child_token));
    /// assert_eq!(sibling_tokens.next(), Some(fourth_child_token));
    /// assert!(sibling_tokens.next().is_none());
    /// ```
    pub fn following_siblings_tokens<'a>(&self, arena: &'a Tree<T>)
        -> FollowingSiblingTokens<'a, T> {
        self.token.following_siblings_tokens(arena)
    }

    /// Returns an iterator of tokens of child nodes.
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node on the tree.
    ///
    /// # Examples:
    ///
    /// ```
    /// use arena_tree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
    ///
    /// let first_child_token = root_token.append(&mut tree, 2usize);
    /// let second_child_token = root_token.append(&mut tree, 3usize);
    /// let third_child_token = root_token.append(&mut tree, 4usize);
    /// let fourth_child_token = root_token.append(&mut tree, 5usize);
    ///
    /// let root = tree.root_node().unwrap();
    /// let mut children_tokens = root_token.children_tokens(&tree);
    /// assert_eq!(children_tokens.next(), Some(first_child_token));
    /// assert_eq!(children_tokens.next(), Some(second_child_token));
    /// assert_eq!(children_tokens.next(), Some(third_child_token));
    /// assert_eq!(children_tokens.next(), Some(fourth_child_token));
    /// assert!(children_tokens.next().is_none());
    /// ```
    pub fn children_tokens<'a>(&self, arena: &'a Tree<T>)
        -> ChildrenTokens<'a, T> {
        self.token.children_tokens(arena)
    }

    /// Returns an iterator of ancestor nodes.
    ///
    /// # Examples:
    ///
    /// ```
    /// use arena_tree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
    ///
    /// let next_node_token = root_token.append(&mut tree, 2usize);
    /// let third_node_token = next_node_token.append(&mut tree, 3usize);
    ///
    /// let third_node = &tree[third_node_token];
    /// let mut ancestors = third_node.ancestors(&tree);
    ///
    /// assert_eq!(ancestors.next().unwrap().data, 2usize);
    /// assert_eq!(ancestors.next().unwrap().data, 1usize);
    /// assert!(ancestors.next().is_none());
    /// ```
    pub fn ancestors<'a>(&self, arena: &'a Tree<T>)
        -> Ancestors<'a, T> {
        self.token.ancestors(arena)
    }

    /// Returns an iterator of references of sibling nodes following the current
    /// node.
    ///
    /// # Examples:
    ///
    /// ```
    /// use arena_tree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
    ///
    /// root_token.append(&mut tree, 2usize);
    /// let second_child_token = root_token.append(&mut tree, 3usize);
    /// root_token.append(&mut tree, 4usize);
    /// root_token.append(&mut tree, 5usize);
    ///
    /// let second_child = &tree[second_child_token];
    /// let mut siblings = second_child_token.following_siblings(&tree);
    /// assert_eq!(siblings.next().unwrap().data, 4usize);
    /// assert_eq!(siblings.next().unwrap().data, 5usize);
    /// assert!(siblings.next().is_none());
    /// ```
    pub fn following_siblings<'a>(&self, arena: &'a Tree<T>)
        -> FollowingSiblings<'a, T> {
        self.token.following_siblings(arena)
    }

    /// Returns an iterator of references of sibling nodes preceding the current
    /// node.
    ///
    /// # Examples:
    ///
    /// ```
    /// use arena_tree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
    ///
    /// root_token.append(&mut tree, 2usize);
    /// root_token.append(&mut tree, 3usize);
    /// let third_child_token = root_token.append(&mut tree, 4usize);
    /// root_token.append(&mut tree, 5usize);
    ///
    /// let third_child = &tree[third_child_token];
    /// let mut siblings = third_child.preceding_siblings(&tree);
    /// assert_eq!(siblings.next().unwrap().data, 3usize);
    /// assert_eq!(siblings.next().unwrap().data, 2usize);
    /// assert!(siblings.next().is_none());
    /// ```
    pub fn preceding_siblings<'a>(&self, arena: &'a Tree<T>)
        -> PrecedingSiblings<'a, T> {
        self.token.preceding_siblings(arena)
    }

    /// Returns an iterator of child node references.
    ///
    /// # Examples:
    ///
    /// ```
    /// use arena_tree::Tree;
    ///
    /// let root_data = 1usize;
    /// let (mut tree, root_token) = Tree::with_root(root_data);
    ///
    /// let first_child_token = root_token.append(&mut tree, 2usize);
    /// let second_child_token = root_token.append(&mut tree, 3usize);
    /// let third_child_token = root_token.append(&mut tree, 4usize);
    /// let fourth_child_token = root_token.append(&mut tree, 5usize);
    ///
    /// let root = tree.root_node().unwrap();
    /// let mut children = root.children(&tree);
    /// assert_eq!(children.next().unwrap().data, 2usize);
    /// assert_eq!(children.next().unwrap().data, 3usize);
    /// assert_eq!(children.next().unwrap().data, 4usize);
    /// assert_eq!(children.next().unwrap().data, 5usize);
    /// assert!(children.next().is_none());
    /// ```
    pub fn children<'a>(&self, arena: &'a Tree<T>) -> Children<'a, T> {
        self.token.children(arena)
    }

    /// Returns an iterator of tokens of child nodes (in pre-order).
    ///
    /// # Examples:
    ///
    /// ```
    /// use arena_tree::Tree;
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
    /// let root = &tree[root_token];
    /// let mut descendents = root.descendents_tokens(&tree);
    /// assert_eq!(descendents.next(), Some(first_child));
    /// assert_eq!(descendents.next(), Some(second_child));
    /// assert_eq!(descendents.next(), Some(first_grandchild));
    /// assert_eq!(descendents.next(), Some(second_grandchild));
    /// assert_eq!(descendents.next(), Some(third_child));
    /// assert_eq!(descendents.next(), Some(fourth_child));
    /// assert!(descendents.next().is_none());
    ///
    /// let second_child_node = &tree[second_child];
    /// let mut descendents = second_child_node.descendents_tokens(&tree);
    /// assert_eq!(descendents.next(), Some(first_grandchild));
    /// assert_eq!(descendents.next(), Some(second_grandchild));
    /// assert!(descendents.next().is_none());
    /// ```
    pub fn descendents_tokens(&self, arena: &Tree<T>) -> DescendentTokens {
        self.token.descendents_tokens(arena)
    }

    /// Returns an iterator of references of child nodes (in pre-order).
    ///
    /// # Panics:
    ///
    /// Panics if the token does not correspond to a node on the tree.
    ///
    /// # Examples:
    ///
    /// ```
    /// use arena_tree::Tree;
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
    /// let root = tree.root_node().unwrap();
    /// let mut descendents = root.descendents(&tree);
    /// assert_eq!(descendents.next().unwrap().data, 2);
    /// assert_eq!(descendents.next().unwrap().data, 3);
    /// assert_eq!(descendents.next().unwrap().data, 4);
    /// assert_eq!(descendents.next().unwrap().data, 10);
    /// assert_eq!(descendents.next().unwrap().data, 20);
    /// assert_eq!(descendents.next().unwrap().data, 5);
    /// assert!(descendents.next().is_none());
    /// ```
    pub fn descendents<'a>(&self, arena: &'a Tree<T>) -> Descendents<'a, T> {
        self.token.descendents(arena)
    }

    pub (crate) fn remove_descendents(&mut self, arena: &mut Tree<T>) {
        self.token.remove_descendents(arena)
    }
}
