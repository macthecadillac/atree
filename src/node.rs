// mutable iterators are impossible for Node<T> due to borrow checking rules
use crate::arena::Arena;
use crate::token::Token;
use crate::iter::*;

/// A node holds data in the arena. `Node<T>` can be accessed by indexing
/// [`Arena<T>`] with [`Token`], using the [`get`] or [`get_mut`] methods of
/// `Arena<T>`, or through tree iterators.
///
/// [`Arena<T>`]: struct.Arena.html
/// [`Token`]: struct.Token.html
/// [`get`]: struct.Arena.html#method.get
/// [`get_mut`]: struct.Arena.html#method.get_mut
#[derive(Debug, Clone)]
pub struct Node<T> {
    /// The `data` field.
    pub data: T,
    /// The token that refers to the current node.
    pub (crate) token: Token,
    /// The parent node.
    pub (crate) parent: Option<Token>,
    /// The "previous sibling" node.
    pub (crate) previous_sibling: Option<Token>,
    /// The "next sibling" node.
    pub (crate) next_sibling: Option<Token>,
    /// The "first child" node.
    pub (crate) first_child: Option<Token>,
}

impl<T> Node<T> {
    /// Returns the token of the given node.
    pub fn token(&self) -> Token { self.token }

    /// Checks whether a given node is actually a leaf.
    pub fn is_leaf(&self) -> bool { self.first_child.is_none() }

    /// Returns an iterator of tokens of ancestor nodes.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Arena;
    /// use atree::Node;
    ///
    /// let root_data = "Indo-European";
    /// let (mut arena, root_token) = Arena::with_data(root_data);
    ///
    /// let next_node_token = root_token.append(&mut arena, "Germanic");
    /// let third_node_token = next_node_token.append(&mut arena, "English");
    ///
    /// let third_node = &arena[third_node_token];
    /// let mut ancestors_tokens = third_node.ancestors_tokens(&arena);
    ///
    /// assert_eq!(ancestors_tokens.next(), Some(next_node_token));
    /// assert_eq!(ancestors_tokens.next(), Some(root_token));
    /// assert!(ancestors_tokens.next().is_none());
    /// ```
    pub fn ancestors_tokens<'a>(&self, arena: &'a Arena<T>)
        -> AncestorTokens<'a, T> {
        self.token.ancestors_tokens(arena)
    }

    /// Returns an iterator of tokens of siblings preceding the current node.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Arena;
    ///
    /// let root_data = "Indo-European";
    /// let (mut arena, root_token) = Arena::with_data(root_data);
    ///
    /// let first_child_token = root_token.append(&mut arena, "Germanic");
    /// let second_child_token = root_token.append(&mut arena, "Romance");
    /// let third_child_token = root_token.append(&mut arena, "Slavic");
    /// root_token.append(&mut arena, "Hellenic");
    ///
    /// let third_child = &arena[third_child_token];
    /// let mut sibling_tokens = third_child.preceding_siblings_tokens(&arena);
    /// assert_eq!(sibling_tokens.next(), Some(second_child_token));
    /// assert_eq!(sibling_tokens.next(), Some(first_child_token));
    /// assert!(sibling_tokens.next().is_none());
    /// ```
    pub fn preceding_siblings_tokens<'a>(&self, arena: &'a Arena<T>)
        -> PrecedingSiblingTokens<'a, T> {
        self.token.preceding_siblings_tokens(arena)
    }

    /// Returns an iterator of tokens of siblings following the current node.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Arena;
    ///
    /// let root_data = "Indo-European";
    /// let (mut arena, root_token) = Arena::with_data(root_data);
    ///
    /// root_token.append(&mut arena, "Romance");
    /// let second_child_token = root_token.append(&mut arena, "Germanic");
    /// let third_child_token = root_token.append(&mut arena, "Slavic");
    /// let fourth_child_token = root_token.append(&mut arena, "Hellenic");
    ///
    /// let second_child = &arena[second_child_token];
    /// let mut sibling_tokens = second_child.following_siblings_tokens(&arena);
    /// assert_eq!(sibling_tokens.next(), Some(third_child_token));
    /// assert_eq!(sibling_tokens.next(), Some(fourth_child_token));
    /// assert!(sibling_tokens.next().is_none());
    /// ```
    pub fn following_siblings_tokens<'a>(&self, arena: &'a Arena<T>)
        -> FollowingSiblingTokens<'a, T> {
        self.token.following_siblings_tokens(arena)
    }

    /// Returns an iterator of tokens of child nodes in the order of insertion.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Arena;
    ///
    /// let root_data = "Indo-European";
    /// let (mut arena, root_token) = Arena::with_data(root_data);
    ///
    /// let first_child_token = root_token.append(&mut arena, "Romance");
    /// let second_child_token = root_token.append(&mut arena, "Germanic");
    /// let third_child_token = root_token.append(&mut arena, "Slavic");
    /// let fourth_child_token = root_token.append(&mut arena, "Hellenic");
    ///
    /// let root = &arena[root_token];
    /// let mut children_tokens = root_token.children_tokens(&arena);
    /// assert_eq!(children_tokens.next(), Some(first_child_token));
    /// assert_eq!(children_tokens.next(), Some(second_child_token));
    /// assert_eq!(children_tokens.next(), Some(third_child_token));
    /// assert_eq!(children_tokens.next(), Some(fourth_child_token));
    /// assert!(children_tokens.next().is_none());
    /// ```
    pub fn children_tokens<'a>(&self, arena: &'a Arena<T>)
        -> ChildrenTokens<'a, T> {
        self.token.children_tokens(arena)
    }

    /// Returns an iterator of references of ancestor nodes.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Arena;
    ///
    /// let root_data = "Indo-European";
    /// let (mut arena, root_token) = Arena::with_data(root_data);
    ///
    /// let next_node_token = root_token.append(&mut arena, "Germanic");
    /// let third_node_token = next_node_token.append(&mut arena, "Swedish");
    ///
    /// let third_node = &arena[third_node_token];
    /// let mut ancestors = third_node.ancestors(&arena);
    ///
    /// assert_eq!(ancestors.next().unwrap().data, "Germanic");
    /// assert_eq!(ancestors.next().unwrap().data, "Indo-European");
    /// assert!(ancestors.next().is_none());
    /// ```
    pub fn ancestors<'a>(&self, arena: &'a Arena<T>)
        -> Ancestors<'a, T> {
        self.token.ancestors(arena)
    }

    /// Returns an iterator of references of sibling nodes following the current
    /// node.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Arena;
    ///
    /// let root_data = "Indo-European";
    /// let (mut arena, root_token) = Arena::with_data(root_data);
    ///
    /// root_token.append(&mut arena, "Romance");
    /// let second_child_token = root_token.append(&mut arena, "Germanic");
    /// root_token.append(&mut arena, "Slavic");
    /// root_token.append(&mut arena, "Hellenic");
    ///
    /// let second_child = &arena[second_child_token];
    /// let mut siblings = second_child_token.following_siblings(&arena);
    /// assert_eq!(siblings.next().unwrap().data, "Slavic");
    /// assert_eq!(siblings.next().unwrap().data, "Hellenic");
    /// assert!(siblings.next().is_none());
    /// ```
    pub fn following_siblings<'a>(&self, arena: &'a Arena<T>)
        -> FollowingSiblings<'a, T> {
        self.token.following_siblings(arena)
    }

    /// Returns an iterator of references of sibling nodes preceding the current
    /// node.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Arena;
    ///
    /// let root_data = "Indo-European";
    /// let (mut arena, root_token) = Arena::with_data(root_data);
    ///
    /// root_token.append(&mut arena, "Romance");
    /// root_token.append(&mut arena, "Germanic");
    /// let third_child_token = root_token.append(&mut arena, "Slavic");
    /// root_token.append(&mut arena, "Celtic");
    ///
    /// let third_child = &arena[third_child_token];
    /// let mut siblings = third_child.preceding_siblings(&arena);
    /// assert_eq!(siblings.next().unwrap().data, "Germanic");
    /// assert_eq!(siblings.next().unwrap().data, "Romance");
    /// assert!(siblings.next().is_none());
    /// ```
    pub fn preceding_siblings<'a>(&self, arena: &'a Arena<T>)
        -> PrecedingSiblings<'a, T> {
        self.token.preceding_siblings(arena)
    }

    /// Returns an iterator of child node references in the order of insertion.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Arena;
    ///
    /// let root_data = "Indo-European";
    /// let (mut arena, root_token) = Arena::with_data(root_data);
    ///
    /// let first_child_token = root_token.append(&mut arena, "Germanic");
    /// let second_child_token = root_token.append(&mut arena, "Romance");
    /// let third_child_token = root_token.append(&mut arena, "Slavic");
    /// let fourth_child_token = root_token.append(&mut arena, "Celtic");
    ///
    /// let root = &arena[root_token];
    /// let mut children = root.children(&arena);
    /// assert_eq!(children.next().unwrap().data, "Germanic");
    /// assert_eq!(children.next().unwrap().data, "Romance");
    /// assert_eq!(children.next().unwrap().data, "Slavic");
    /// assert_eq!(children.next().unwrap().data, "Celtic");
    /// assert!(children.next().is_none());
    /// ```
    pub fn children<'a>(&self, arena: &'a Arena<T>) -> Children<'a, T> {
        self.token.children(arena)
    }

    /// Returns an iterator of tokens of subtree nodes of the given node.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Arena;
    /// use atree::iter::TraversalOrder;
    ///
    /// let root_data = "Indo-European";
    /// let (mut arena, root_token) = Arena::with_data(root_data);
    ///
    /// let first_child = root_token.append(&mut arena, "Romance");
    /// let second_child = root_token.append(&mut arena, "Germanic");
    /// let third_child = root_token.append(&mut arena, "Slavic");
    /// let first_grandchild = second_child.append(&mut arena, "English");
    /// let second_grandchild = second_child.append(&mut arena, "Icelandic");
    /// let fourth_child = root_token.append(&mut arena, "Celtic");
    ///
    /// let root = &arena[root_token];
    /// let mut subtree = root.subtree_tokens(&arena, TraversalOrder::Pre);
    /// assert_eq!(subtree.next(), Some(root_token));
    /// assert_eq!(subtree.next(), Some(first_child));
    /// assert_eq!(subtree.next(), Some(second_child));
    /// assert_eq!(subtree.next(), Some(first_grandchild));
    /// assert_eq!(subtree.next(), Some(second_grandchild));
    /// assert_eq!(subtree.next(), Some(third_child));
    /// assert_eq!(subtree.next(), Some(fourth_child));
    /// assert!(subtree.next().is_none());
    ///
    /// let second_child_node = &arena[second_child];
    /// let mut subtree = second_child_node.subtree_tokens(&arena, TraversalOrder::Pre);
    /// assert_eq!(subtree.next(), Some(second_child));
    /// assert_eq!(subtree.next(), Some(first_grandchild));
    /// assert_eq!(subtree.next(), Some(second_grandchild));
    /// assert!(subtree.next().is_none());
    /// ```
    pub fn subtree_tokens<'a>(&self, arena: &'a Arena<T>, order: TraversalOrder)
        -> SubtreeTokens<'a, T> {
        self.token.subtree_tokens(arena, order)
    }

    /// Returns an iterator of references of subtree nodes of the given node.
    ///
    /// # Examples:
    ///
    /// ```
    /// use atree::Arena;
    /// use atree::iter::TraversalOrder;
    ///
    /// let root_data = "Indo-European";
    /// let (mut arena, root_token) = Arena::with_data(root_data);
    ///
    /// root_token.append(&mut arena, "Romance");
    /// root_token.append(&mut arena, "Germanic");
    /// let third_child = root_token.append(&mut arena, "Slavic");
    /// root_token.append(&mut arena, "Celtic");
    /// third_child.append(&mut arena, "Polish");
    /// third_child.append(&mut arena, "Slovakian");
    ///
    /// let root = &arena[root_token];
    /// let mut subtree = root.subtree(&arena, TraversalOrder::Pre);
    /// assert_eq!(subtree.next().unwrap().data, "Indo-European");
    /// assert_eq!(subtree.next().unwrap().data, "Romance");
    /// assert_eq!(subtree.next().unwrap().data, "Germanic");
    /// assert_eq!(subtree.next().unwrap().data, "Slavic");
    /// assert_eq!(subtree.next().unwrap().data, "Polish");
    /// assert_eq!(subtree.next().unwrap().data, "Slovakian");
    /// assert_eq!(subtree.next().unwrap().data, "Celtic");
    /// assert!(subtree.next().is_none());
    /// ```
    pub fn subtree<'a>(&self, arena: &'a Arena<T>, order: TraversalOrder)
        -> Subtree<'a, T> {
        self.token.subtree(arena, order)
    }

    pub (crate) fn remove_descendants(&mut self, arena: &mut Arena<T>) {
        self.token.remove_descendants(arena)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn subtree_tokens_postord() {
        let root_data = 1usize;
        let (mut arena, root_token) = Arena::with_data(root_data);
       
        let first_child = root_token.append(&mut arena, 2usize);
        let second_child = root_token.append(&mut arena, 3usize);
        let third_child = root_token.append(&mut arena, 4usize);
        let first_grandchild = second_child.append(&mut arena, 10usize);
        let second_grandchild = second_child.append(&mut arena, 20usize);
        let fourth_child = root_token.append(&mut arena, 5usize);
       
        let root = &arena[root_token];
        let mut subtree = root.subtree_tokens(&arena, TraversalOrder::Post);
        assert_eq!(subtree.next(), Some(first_child));
        assert_eq!(subtree.next(), Some(first_grandchild));
        assert_eq!(subtree.next(), Some(second_grandchild));
        assert_eq!(subtree.next(), Some(second_child));
        assert_eq!(subtree.next(), Some(third_child));
        assert_eq!(subtree.next(), Some(fourth_child));
        assert_eq!(subtree.next(), Some(root_token));
        assert!(subtree.next().is_none());
       
        let second_child_node = &arena[second_child];
        let mut subtree = second_child_node.subtree_tokens(&arena, TraversalOrder::Post);
        assert_eq!(subtree.next(), Some(first_grandchild));
        assert_eq!(subtree.next(), Some(second_grandchild));
        assert_eq!(subtree.next(), Some(second_child));
        assert!(subtree.next().is_none());
    }

    #[test]
    fn subtree_postord() {
        let root_data = "Indo-European";
        let (mut arena, root_token) = Arena::with_data(root_data);
       
        root_token.append(&mut arena, "Romance");
        root_token.append(&mut arena, "Germanic");
        let third_child = root_token.append(&mut arena, "Celtic");
        root_token.append(&mut arena, "Slavic");
        third_child.append(&mut arena, "Ulster");
        third_child.append(&mut arena, "Gaulish");
       
        let root = &arena[root_token];
        let mut subtree = root.subtree(&arena, TraversalOrder::Post);
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
    fn subtree_tokens_levelord() {
        let root_data = 1usize;
        let (mut arena, root_token) = Arena::with_data(root_data);
       
        let first_child = root_token.append(&mut arena, 2usize);
        let second_child = root_token.append(&mut arena, 3usize);
        let third_child = root_token.append(&mut arena, 4usize);
        let first_grandchild = second_child.append(&mut arena, 10usize);
        let second_grandchild = second_child.append(&mut arena, 20usize);
        let fourth_child = root_token.append(&mut arena, 5usize);
       
        let root = &arena[root_token];
        let mut subtree = root.subtree_tokens(&arena, TraversalOrder::Level);
        assert_eq!(subtree.next(), Some(root_token));
        assert_eq!(subtree.next(), Some(first_child));
        assert_eq!(subtree.next(), Some(second_child));
        assert_eq!(subtree.next(), Some(third_child));
        assert_eq!(subtree.next(), Some(fourth_child));
        assert_eq!(subtree.next(), Some(first_grandchild));
        assert_eq!(subtree.next(), Some(second_grandchild));
        assert!(subtree.next().is_none());
    }

    #[test]
    fn subtree_levelord() {
        let root_data = "Indo-European";
        let (mut arena, root_token) = Arena::with_data(root_data);
       
        root_token.append(&mut arena, "Romance");
        root_token.append(&mut arena, "Germanic");
        let third_child = root_token.append(&mut arena, "Slavic");
        root_token.append(&mut arena, "Hellenic");
        third_child.append(&mut arena, "Russian");
        third_child.append(&mut arena, "Ukrainian");
       
        let root = &arena[root_token];
        let mut subtree = root.subtree(&arena, TraversalOrder::Level);
        assert_eq!(subtree.next().unwrap().data, "Indo-European");
        assert_eq!(subtree.next().unwrap().data, "Romance");
        assert_eq!(subtree.next().unwrap().data, "Germanic");
        assert_eq!(subtree.next().unwrap().data, "Slavic");
        assert_eq!(subtree.next().unwrap().data, "Hellenic");
        assert_eq!(subtree.next().unwrap().data, "Russian");
        assert_eq!(subtree.next().unwrap().data, "Ukrainian");
        assert!(subtree.next().is_none());
    }
}
