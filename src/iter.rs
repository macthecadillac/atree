//! A module that contains different kinds of iterators defined on the tree.
use std::marker::PhantomData;

use crate::Tree;
use crate::node::Node;
use crate::token::Token;

pub (crate) enum Branch { Sibling, Child }

/// An iterator of tokens of descendants of a given node.
///
/// This `struct` is created by the `descendants_tokens` methods on [`Token`]
/// and [`Node`]. See their documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.descendants_tokens
/// [`Node`]: ../struct.Node.html#method.descendants_tokens
pub struct DescendantTokens<'a, T> {
    pub (crate) tree: &'a Tree<T>,
    pub (crate) subtree_root: Token,
    pub (crate) node_token: Option<Token>,
}

impl<'a, T> Iterator for DescendantTokens<'a, T> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        // preorder traversal
        let aux = |mut node_token, root, mut branch, tree: &Tree<T>|
            loop {
                let node = &tree[node_token];
                match branch {
                    Branch::Child => match node.first_child {
                        Some(token) => break Some(token),
                        None => branch = Branch::Sibling
                    }
                    Branch::Sibling => {
                        match node.next_sibling {
                            Some(token) => break Some(token),
                            None => match node.parent {
                                None => break None,
                                Some(parent) => if parent == root {
                                    break None
                                } else {
                                    node_token = parent;
                                    branch = Branch::Sibling;
                                }
                            }
                        }
                    }
                }
            };

        match self.node_token{
            Some(token) => {
                match self.tree.get(token) {
                    Some(_) => {
                        self.node_token = aux(token, self.subtree_root,
                                              Branch::Child, self.tree);
                        Some(token)
                    },
                    None => panic!("Stale token: {:?} is not found in \
                                    the tree. Check code", token)
                }
            },
            None => None
        }
    }
}

/// An iterator of references of descendants of a given node.
///
/// This `struct` is created by the `descendants` methods on [`Token`]
/// and [`Node`]. See their documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.descendants
/// [`Node`]: ../struct.Node.html#method.descendants
pub struct Descendants<'a, T> {
    pub (crate) tree: &'a Tree<T>,
    pub (crate) descendants: DescendantTokens<'a, T>
}

impl<'a, T> Iterator for Descendants<'a, T> {
    type Item = &'a Node<T>;
    fn next(&mut self) -> Option<&'a Node<T>> {
        match self.descendants.next() {
            Some(node_token) => self.tree.get(node_token),
            None => None
        }
    }
}

/// An iterator of mutable references of descendants of a given node.
///
/// This `struct` is created by the `descendants_mut` method on [`Token`]. See
/// its documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.descendants_mut
pub struct DescendantsMut<'a, T: 'a> {
    pub (crate) tree: *mut Tree<T>,
    pub (crate) descendants: DescendantTokens<'a, T>,
    pub (crate) marker: PhantomData<&'a mut T>
}

impl<'a, T> Iterator for DescendantsMut<'a, T> {
    type Item = &'a mut Node<T>;
    fn next(&mut self) -> Option<&'a mut Node<T>> {
        match self.descendants.next() {
            Some(node_token) => {
                let tree = unsafe { self.tree.as_mut().unwrap() };
                match tree.get_mut(node_token) {
                    Some(node) => Some(node),
                    None => None
                }
            },
            None => None
        }
    }
}

/// An iterator of tokens of siblings that follow a given node.
///
/// This `struct` is created by the `following_siblings_tokens` methods on
/// [`Token`] and [`Node`]. See their documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.following_siblings_tokens
/// [`Node`]: ../struct.Node.html#method.following_siblings_tokens
pub struct FollowingSiblingTokens<'a, T> {
    pub (crate) tree: &'a Tree<T>,
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
    pub (crate) tree: &'a Tree<T>,
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
    pub (crate) tree: &'a Tree<T>,
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
    pub (crate) tree: &'a Tree<T>,
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
/// This `struct` is created by the `preceding_siblings_mut` method on
/// [`Token`]. See its documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.preceding_siblings_mut
pub struct PrecedingSiblingsMut<'a, T: 'a> {
    pub (crate) tree: *mut Tree<T>,
    pub (crate) node_token: Option<Token>,
    pub (crate) marker: PhantomData<&'a mut T>
}

/// An iterator of mutable references to siblings that follow a given node.
///
/// This `struct` is created by the `following_siblings_mut` method on
/// [`Token`]. See its documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.following_siblings_mut
pub struct FollowingSiblingsMut<'a, T: 'a> {
    pub (crate) tree: *mut Tree<T>,
    pub (crate) node_token: Option<Token>,
    pub (crate) marker: PhantomData<&'a mut T>
}

/// An iterator of mutable references to the children of a given node.
///
/// This `struct` is created by the `children_mut` method on
/// [`Token`]. See its documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.children_mut
pub struct ChildrenMut<'a, T: 'a> {
    pub (crate) tree: *mut Tree<T>,
    pub (crate) node_token: Option<Token>,
    pub (crate) marker: PhantomData<&'a mut T>
}

/// An iterator of mutable references to the ancestors of a given node.
///
/// This `struct` is created by the `ancestors_mut` method on
/// [`Token`]. See its documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.ancestors_mut
pub struct AncestorsMut<'a, T: 'a> {
    pub (crate) tree: *mut Tree<T>,
    pub (crate) node_token: Option<Token>,
    pub (crate) marker: PhantomData<&'a mut T>
}

macro_rules! iterator {
    (@token struct $name:ident > $field:ident) => {
        impl<'a, T> Iterator for $name<'a, T> {
            type Item = Token;
            fn next(&mut self) -> Option<Token> {
                match self.node_token {
                    Some(curr_node_token) => {
                        match self.tree.get(curr_node_token) {
                            Some(curr_node) => {
                                self.node_token = curr_node.$field;
                                Some(curr_node_token)
                            },
                            None => panic!("Stale token: {:?} is not found in \
                                            the tree. Check code", curr_node_token)
                        }
                    },
                    None => None
                }
            }
        }
    };

    // perhaps fold this into the @token branch since this can be implemented with
    // largely the same code with one less Tree::get (one less look-up should
    // translate to more performant code)
    (@node struct $name:ident) => {
        impl<'a, T> Iterator for $name<'a, T> {
            type Item = &'a Node<T>;
            fn next(&mut self) -> Option<&'a Node<T>> {
                match self.token_iter.next() {
                    Some(node_token) => self.token_iter.tree.get(node_token),
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
                    Some(curr_node_token) => {
                        let tree = unsafe { self.tree.as_mut().unwrap() };
                        match tree.get_mut(curr_node_token) {
                            Some(curr_node) => {
                                self.node_token = curr_node.$field;
                                Some(curr_node)
                            },
                            None => None
                        }
                    },
                    None => None
                }
            }
        }
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
