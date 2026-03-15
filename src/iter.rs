#![allow(clippy::match_bool)]
//! A module that contains different kinds of iterators.
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::mem;

use crate::Arena;
use crate::node::Node;
use crate::token::Token;

/// A flag for the branch the next iteration should take when traversing the
/// tree. See [`preorder_next`] and [`postorder_next`] for usage.
///
/// [`preorder_next`]: fn.preorder_next.html
/// [`postorder_next`]: fn.postorder_next.html
#[derive(Clone, Copy, PartialEq, Eq)]
pub (crate) enum Branch {
    /// The sibling branch
    Sibling,
    /// The child branch
    Child,
    /// End of iteration
    None
}

/// The order in which tree traversal takes place.
#[derive(Clone, Copy)]
pub enum TraversalOrder {
    /// Pre-order (depth-first traversal)
    Pre,
    /// Post-order (depth-first traversal)
    Post,
    /// Level-order (breadth-first traversal)
    Level
}

/// A helper function to find the next node in the tree during preorder
/// traversal. To be used with [`depth_first_tokens_next`].
///
/// [`depth_first_tokens_next`]: fn.depth_first_tokens_next.html
pub (crate) fn preorder_next<T>(mut node_token: Token,
                                root: Token,
                                mut branch: Branch,
                                arena: &Arena<T>)
    -> (Option<Token>, Branch) {
    loop {
        let node = match arena.get(node_token) {
            Some(n) => n,
            // Dead code: token pre-validated by depth_first_tokens_next
            None => panic!("Invalid token")
        };
        match branch {
            // Dead code: unreachable by construction
            Branch::None => panic!("Unreachable arm. Check code."),  // unreachable
            Branch::Child => match node.first_child {
                Some(token) => break (Some(token), Branch::Child),
                None => match node_token == root {
                    true => break (None, Branch::None),
                    false => branch = Branch::Sibling
                }
            },
            Branch::Sibling => match node.next_sibling {
                Some(token) => break (Some(token), Branch::Child),
                None => match node.parent {
                    // Dead code: parent == root check fires first
                    None => break (None, Branch::None),
                    Some(parent) => match parent == root {
                        true => break (None, Branch::None),
                        false => {
                            node_token = parent;
                            branch = Branch::Sibling;
                        }
                    }
                }
            }
        }
    }
}

/// A helper function to find the next node in the tree during postorder
/// traversal. To be used with [`depth_first_tokens_next`].
///
/// [`depth_first_tokens_next`]: fn.depth_first_tokens_next.html
pub (crate) fn postorder_next<T>(mut node_token: Token,
                                 root: Token,
                                 mut branch: Branch,
                                 arena: &Arena<T>)
    -> (Option<Token>, Branch) {
    let mut switch_branch = true;
    loop {
        let node = match arena.get(node_token) {
            Some(n) => n,
            // Dead code: token pre-validated by depth_first_tokens_next
            None => panic!("Invalid token")
        };
        match branch {
            // Dead code: postorder iterator stops before calling with Branch::None
            Branch::None => break (None, Branch::None),
            Branch::Child => match node.first_child {
                Some(token) => {
                    node_token = token;
                    switch_branch = false;
                },
                None => match switch_branch {
                    false => break (Some(node_token), Branch::Sibling),
                    true => match node_token == root {
                        true => break (Some(root), Branch::None),  // no descendants
                        false => branch = Branch::Sibling,
                    }
                }
            },
            Branch::Sibling => match node.next_sibling {
                Some(token) => {
                    switch_branch = false;
                    node_token = token;
                    branch = Branch::Child;
                },
                None => match node.parent {
                    // Dead code: parent == root check fires first
                    None => break (None, Branch::Child),
                    Some(parent) => match parent == root {
                        true => break (Some(root), Branch::None),
                        false => break (Some(parent), Branch::Sibling)
                    }
                }
            }
        }
    }
}


/// A function to be curried at the call-site. Used in [`subtree_tokens`] for
/// the construction of [`SubtreeTokens`].
///
/// [`subtree_tokens`]: ../struct.Token.html#method.subtree_tokens
/// [`SubtreeTokens`]: struct.SubtreeTokens.html
#[allow(clippy::type_complexity)]
pub (crate) fn depth_first_tokens_next<'a, T>(
    iter: &mut SubtreeTokens<'a, T>,
    func: fn(Token, Token, Branch, &Arena<T>) -> (Option<Token>, Branch)
) -> Option<Token> {
    match iter.node_token {
        None => None,
        Some(token) => match iter.arena.get(token) {
            // Dead code: token produced by prior traversal on live arena
            None => panic!("Stale token: {:?} is not found in \
                            the arena. Check code", token),
            Some(_) => {
                let (next_node, branch) = func(
                    token,
                    iter.subtree_root,
                    iter.branch,
                    iter.arena
                );
                iter.node_token = next_node;
                iter.branch = branch;
                Some(token)
            }
        }
    }
}

/// A function to be curried at the call-site. Used in [`subtree_tokens`] for
/// the construction of [`SubtreeTokens`].
///
/// [`subtree_tokens`]: ../struct.Token.html#method.subtree_tokens
/// [`SubtreeTokens`]: struct.SubtreeTokens.html
pub (crate) fn breadth_first_tokens_next<'a, T> (iter: &mut SubtreeTokens<'a, T>)
    -> Option<Token> {
    match iter.curr_level.pop_front() {
        Some(token) => {
            iter.next_level.extend(token.children_tokens(iter.arena));
            Some(token)
        },
        None => match iter.next_level.is_empty() {
            true => None,
            false => {
                mem::swap(&mut iter.curr_level, &mut iter.next_level);
                iter.next()
            }
        }
    }
}

/// An iterator of tokens of the subtree nodes of a given node.
///
/// This `struct` is created by the `subtree_tokens` methods on [`Token`]
/// and [`Node`]. See their documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.subtree_tokens
/// [`Node`]: ../struct.Node.html#method.subtree_tokens
pub struct SubtreeTokens<'a, T> {
    pub (crate) arena: &'a Arena<T>,
    pub (crate) subtree_root: Token,
    pub (crate) node_token: Option<Token>,
    pub (crate) branch: Branch,
    pub (crate) curr_level: VecDeque<Token>,
    pub (crate) next_level: VecDeque<Token>,
    pub (crate) next: fn(&mut SubtreeTokens<T>) -> Option<Token>
}

impl<'a, T> Iterator for SubtreeTokens<'a, T> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> { (self.next)(self) }
}

/// An iterator of references of the subtree nodes of a given node.
///
/// This `struct` is created by the `subtree` methods on [`Token`]
/// and [`Node`]. See their documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.subtree
/// [`Node`]: ../struct.Node.html#method.subtree
pub struct Subtree<'a, T> {
    pub (crate) arena: &'a Arena<T>,
    pub (crate) iter: SubtreeTokens<'a, T>
}

impl<'a, T> Iterator for Subtree<'a, T> {
    type Item = &'a Node<T>;
    fn next(&mut self) -> Option<&'a Node<T>> {
        match self.iter.next() {
            Some(node_token) => self.arena.get(node_token),
            None => None
        }
    }
}

/// An iterator of mutable references of the subtree nodes of a given node.
///
/// This `struct` is created by the [`subtree_mut`] method on `Token`. See
/// its documentation for more.
///
/// [`subtree_mut`]: ../struct.Token.html#method.subtree_mut
pub struct SubtreeMut<'a, T: 'a> {
    pub (crate) arena: *mut Arena<T>,
    pub (crate) iter: SubtreeTokens<'a, T>,
    pub (crate) marker: PhantomData<&'a mut T>
}

impl<'a, T> Iterator for SubtreeMut<'a, T> {
    type Item = &'a mut Node<T>;
    fn next(&mut self) -> Option<&'a mut Node<T>> {
        match self.iter.next() {
            None => None,
            Some(node_token) => {
                let arena = unsafe { self.arena.as_mut().unwrap() };
                match arena.get_mut(node_token) {
                    Some(node) => Some(node),
                    // Dead code: token produced by live SubtreeTokens iterator
                    None => None
                }
            }
        }
    }
}

unsafe impl<T: Sync> Sync for SubtreeMut<'_, T> {}
unsafe impl<T: Send> Send for SubtreeMut<'_, T> {}

/// An iterator of tokens of siblings that follow a given node.
///
/// This `struct` is created by the `following_siblings_tokens` methods on
/// [`Token`] and [`Node`]. See their documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.following_siblings_tokens
/// [`Node`]: ../struct.Node.html#method.following_siblings_tokens
pub struct FollowingSiblingTokens<'a, T> {
    pub (crate) arena: &'a Arena<T>,
    pub (crate) node_token: Option<Token>
}

/// An iterator of tokens of siblings that precede a given node.
///
/// This `struct` is created by the `preceding_siblings_tokens` methods on
/// [`Token`] and [`Node`]. See their documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.preceding_siblings_tokens
/// [`Node`]: ../struct.Node.html#method.preceding_siblings_tokens
pub struct PrecedingSiblingTokens<'a, T> {
    pub (crate) arena: &'a Arena<T>,
    pub (crate) node_token: Option<Token>
}

/// An iterator of tokens of the children of a given node.
///
/// This `struct` is created by the `children_tokens` methods on
/// [`Token`] and [`Node`]. See their documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.children_tokens
/// [`Node`]: ../struct.Node.html#method.children_tokens
pub struct ChildrenTokens<'a, T> {
    pub (crate) arena: &'a Arena<T>,
    pub (crate) node_token: Option<Token>
}

/// An iterator of tokens of the ancestors of a given node.
///
/// This `struct` is created by the `ancestors_tokens` methods on
/// [`Token`] and [`Node`]. See their documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.ancestors_tokens
/// [`Node`]: ../struct.Node.html#method.ancestors_tokens
pub struct AncestorTokens<'a, T> {
    pub (crate) arena: &'a Arena<T>,
    pub (crate) node_token: Option<Token>
}

/// An iterator of references to siblings that precede a given node.
///
/// This `struct` is created by the `preceding_siblings` methods on
/// [`Token`] and [`Node`]. See their documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.preceding_siblings
/// [`Node`]: ../struct.Node.html#method.preceding_siblings
pub struct PrecedingSiblings<'a, T> {
    pub (crate) token_iter: PrecedingSiblingTokens<'a, T>
}

/// An iterator of references to siblings that follow a given node.
///
/// This `struct` is created by the `following_siblings` methods on
/// [`Token`] and [`Node`]. See their documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.following_siblings
/// [`Node`]: ../struct.Node.html#method.following_siblings
pub struct FollowingSiblings<'a, T> {
    pub (crate) token_iter: FollowingSiblingTokens<'a, T>
}

/// An iterator of references to the children of a given node.
///
/// This `struct` is created by the `children` methods on
/// [`Token`] and [`Node`]. See their documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.children
/// [`Node`]: ../struct.Node.html#method.children
pub struct Children<'a, T> {
    pub (crate) token_iter: ChildrenTokens<'a, T>
}

/// An iterator of references to the ancestors of a given node.
///
/// This `struct` is created by the `ancestors` methods on
/// [`Token`] and [`Node`]. See their documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.ancestors
/// [`Node`]: ../struct.Node.html#method.ancestors
pub struct Ancestors<'a, T> {
    pub (crate) token_iter: AncestorTokens<'a, T>
}

/// An iterator of mutable references to siblings that precede a given node.
///
/// This `struct` is created by the [`preceding_siblings_mut`] method on
/// `Token`. See its documentation for more.
///
/// [`preceding_siblings_mut`]: ../struct.Token.html#method.preceding_siblings_mut
pub struct PrecedingSiblingsMut<'a, T: 'a> {
    pub (crate) arena: *mut Arena<T>,
    pub (crate) node_token: Option<Token>,
    pub (crate) marker: PhantomData<&'a mut T>
}

/// An iterator of mutable references to siblings that follow a given node.
///
/// This `struct` is created by the [`following_siblings_mut`] method on
/// `Token`. See its documentation for more.
///
/// [`following_siblings_mut`]: ../struct.Token.html#method.following_siblings_mut
pub struct FollowingSiblingsMut<'a, T: 'a> {
    pub (crate) arena: *mut Arena<T>,
    pub (crate) node_token: Option<Token>,
    pub (crate) marker: PhantomData<&'a mut T>
}

/// An iterator of mutable references to the children of a given node.
///
/// This `struct` is created by the [`children_mut`] method on
/// `Token`. See its documentation for more.
///
/// [`children_mut`]: ../struct.Token.html#method.children_mut
pub struct ChildrenMut<'a, T: 'a> {
    pub (crate) arena: *mut Arena<T>,
    pub (crate) node_token: Option<Token>,
    pub (crate) marker: PhantomData<&'a mut T>
}

/// An iterator of mutable references to the ancestors of a given node.
///
/// This `struct` is created by the [`ancestors_mut`] method on
/// `Token`. See its documentation for more.
///
/// [`ancestors_mut`]: ../struct.Token.html#method.ancestors_mut
pub struct AncestorsMut<'a, T: 'a> {
    pub (crate) arena: *mut Arena<T>,
    pub (crate) node_token: Option<Token>,
    pub (crate) marker: PhantomData<&'a mut T>
}

/// A macro that implements the `Iterator` trait on iterators (aside from ones
/// related to subtree traversal.
macro_rules! iterator {
    (@token struct $name:ident > $field:ident) => {
        impl<'a, T> Iterator for $name<'a, T> {
            type Item = Token;
            fn next(&mut self) -> Option<Token> {
                match self.node_token {
                    None => None,
                    Some(token) => match self.arena.get(token) {
                        // Dead code: token produced by prior traversal on live arena
                        None => panic!("Stale token: {:?} is not found in \
                                        the arena. Check code", token),
                        Some(curr_node) => {
                            self.node_token = curr_node.$field;
                            Some(token)
                        }
                    }
                }
            }
        }
    };

    // perhaps fold this into the @token branch since this can be implemented with
    // largely the same code with one less Arena::get (one less look-up should
    // translate to more performant code)
    (@node struct $name:ident) => {
        impl<'a, T> Iterator for $name<'a, T> {
            type Item = &'a Node<T>;
            fn next(&mut self) -> Option<&'a Node<T>> {
                match self.token_iter.next() {
                    Some(node_token) => self.token_iter.arena.get(node_token),
                    None => None
                }
            }
        }
    };

    (@mut struct $name:ident > $field:ident) => {
        impl<'a, T> Iterator for $name<'a, T> {
            type Item = &'a mut Node<T>;
            fn next(&mut self) -> Option<&'a mut Node<T>> {
                match self.node_token {
                    None => None,
                    Some(curr_node_token) => {
                        let arena = unsafe { self.arena.as_mut().unwrap() };
                        match arena.get_mut(curr_node_token) {
                            None => None,
                            Some(curr_node) => {
                                self.node_token = curr_node.$field;
                                Some(curr_node)
                            }
                        }
                    }
                }
            }
        }

        unsafe impl<T: Sync> Sync for $name<'_, T> {}
        unsafe impl<T: Send> Send for $name<'_, T> {}
    }
}

iterator!(@token struct FollowingSiblingTokens > next_sibling);
iterator!(@token struct PrecedingSiblingTokens > previous_sibling);
iterator!(@token struct ChildrenTokens > next_sibling);
iterator!(@token struct AncestorTokens > parent);
iterator!(@node struct PrecedingSiblings);
iterator!(@node struct FollowingSiblings);
iterator!(@node struct Children);
iterator!(@node struct Ancestors);
iterator!(@mut struct PrecedingSiblingsMut > previous_sibling);
iterator!(@mut struct FollowingSiblingsMut > next_sibling);
iterator!(@mut struct ChildrenMut > next_sibling);
iterator!(@mut struct AncestorsMut > parent);

#[cfg(test)]
mod test {
    use super::*;
    use crate::Arena;

    #[test]
    fn subtree_mut_returns_none_when_exhausted() {
        let (mut arena, root) = Arena::with_data(1usize);
        root.append(&mut arena, 2usize);
        let mut iter = root.subtree_mut(&mut arena, TraversalOrder::Pre);
        // consume all items
        while iter.next().is_some() {}
        // further calls should return None
        assert!(iter.next().is_none());
    }

    #[test]
    fn ancestors_of_root_is_empty() {
        let (arena, root) = Arena::with_data(1usize);
        let mut ancestors = root.ancestors_tokens(&arena);
        assert!(ancestors.next().is_none());
    }

    #[test]
    fn children_of_leaf_is_empty() {
        let (mut arena, root) = Arena::with_data(1usize);
        let leaf = root.append(&mut arena, 2usize);
        let mut children = leaf.children_tokens(&arena);
        assert!(children.next().is_none());
    }

    #[test]
    fn following_siblings_of_last_is_empty() {
        let (mut arena, root) = Arena::with_data(1usize);
        root.append(&mut arena, 2usize);
        let last = root.append(&mut arena, 3usize);
        let mut siblings = last.following_siblings_tokens(&arena);
        assert!(siblings.next().is_none());
    }

    #[test]
    fn preceding_siblings_of_first_is_empty() {
        let (mut arena, root) = Arena::with_data(1usize);
        let first = root.append(&mut arena, 2usize);
        root.append(&mut arena, 3usize);
        let mut siblings = first.preceding_siblings_tokens(&arena);
        assert!(siblings.next().is_none());
    }

    #[test]
    fn postorder_ancestor_return_path() {
        let (mut arena, root) = Arena::with_data(0usize);
        let a = root.append(&mut arena, 1usize);
        let b = a.append(&mut arena, 2usize);
        let c = b.append(&mut arena, 3usize);
        let d = root.append(&mut arena, 4usize);

        let result: Vec<_> = root.subtree_tokens(&arena, TraversalOrder::Post).collect();
        assert_eq!(result, vec![c, b, a, d, root]);
    }

    #[test]
    fn breadth_first_level_swap_covered() {
        // Build a tree with 2 levels so breadth-first hits the level-swap branch
        let (mut arena, root) = Arena::with_data(1usize);
        let c1 = root.append(&mut arena, 2usize);
        let c2 = root.append(&mut arena, 3usize);
        c1.append(&mut arena, 4usize);
        c2.append(&mut arena, 5usize);

        let tokens: Vec<_> = root.subtree_tokens(&arena, TraversalOrder::Level).collect();
        assert_eq!(tokens[0], root);
        assert_eq!(tokens[1], c1);
        assert_eq!(tokens[2], c2);
        // grandchildren follow after level swap
        assert_eq!(tokens.len(), 5);
    }
}
