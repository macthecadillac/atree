#![allow(clippy::match_bool)]
//! A module that contains different kinds of iterators defined on the tree.
use std::marker::PhantomData;

use crate::Tree;
use crate::node::Node;
use crate::token::Token;

#[derive(Clone, Copy)]
pub (crate) enum Branch { Sibling, Child }

fn preorder_next<T>(mut node_token: Token,
                    root: Token,
                    mut branch: Branch,
                    tree: &Tree<T>)
    -> (Option<Token>, Branch) {
    loop {
        let node = &tree[node_token];
        match branch {
            Branch::Child => match node.first_child {
                Some(token) => break (Some(token), Branch::Child),
                None => branch = Branch::Sibling
            },
            Branch::Sibling => match node.next_sibling {
                Some(token) => break (Some(token), Branch::Child),
                None => match node.parent {
                    None => break (None, Branch::Child),
                    Some(parent) => match parent == root {
                        true => break (None, Branch::Child),
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

pub (crate) fn postorder_next<T>(mut node_token: Token,
                                 root: Token,
                                 mut branch: Branch,
                                 tree: &Tree<T>)
    -> (Option<Token>, Branch) {
    let mut switch_branch = true;
    loop {
        let node = &tree[node_token];
        match branch {
            Branch::Child => match node.first_child {
                Some(token) => {
                    node_token = token;
                    switch_branch = false;
                },
                None => match switch_branch {
                    true => branch = Branch::Sibling,
                    false => break (Some(node_token), Branch::Sibling)
                }
            },
            Branch::Sibling => match node.next_sibling {
                Some(token) => {
                    switch_branch = false;
                    node_token = token;
                    branch = Branch::Child;
                },
                None => match node.parent {
                    None => break (None, Branch::Child),
                    Some(parent) => match parent == root {
                        true => break (None, Branch::Child),
                        false => break (Some(parent), Branch::Sibling)
                    }
                }
            }
        }
    }
}

/// An iterator of tokens of descendants of a given node in pre-order.
///
/// This `struct` is created by the `descendants_tokens` methods on [`Token`]
/// and [`Node`]. See their documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.descendants_tokens
/// [`Node`]: ../struct.Node.html#method.descendants_tokens
pub struct DescendantsTokensPreord<'a, T> {
    pub (crate) tree: &'a Tree<T>,
    pub (crate) subtree_root: Token,
    pub (crate) node_token: Option<Token>,
    pub (crate) branch: Branch
}

/// An iterator of tokens of descendants of a given node in post-order.
///
/// This `struct` is created by the `descendants_tokens` methods on [`Token`]
/// and [`Node`]. See their documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.descendants_tokens
/// [`Node`]: ../struct.Node.html#method.descendants_tokens
pub struct DescendantsTokensPostord<'a, T> {
    pub (crate) tree: &'a Tree<T>,
    pub (crate) subtree_root: Token,
    pub (crate) node_token: Option<Token>,
    pub (crate) branch: Branch
}

/// An iterator of references of descendants of a given node in pre-order.
///
/// This `struct` is created by the `descendants` methods on [`Token`]
/// and [`Node`]. See their documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.descendants
/// [`Node`]: ../struct.Node.html#method.descendants
pub struct DescendantsPreord<'a, T> {
    pub (crate) tree: &'a Tree<T>,
    pub (crate) descendants: DescendantsTokensPreord<'a, T>
}

/// An iterator of references of descendants of a given node in post-order.
///
/// This `struct` is created by the `descendants` methods on [`Token`]
/// and [`Node`]. See their documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.descendants
/// [`Node`]: ../struct.Node.html#method.descendants
pub struct DescendantsPostord<'a, T> {
    pub (crate) tree: &'a Tree<T>,
    pub (crate) descendants: DescendantsTokensPostord<'a, T>
}

/// An iterator of mutable references of descendants of a given node in
/// pre-order.
///
/// This `struct` is created by the `descendants_mut` method on [`Token`]. See
/// its documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.descendants_mut
pub struct DescendantsMutPreord<'a, T: 'a> {
    pub (crate) tree: *mut Tree<T>,
    pub (crate) descendants: DescendantsTokensPreord<'a, T>,
    pub (crate) marker: PhantomData<&'a mut T>
}

/// An iterator of mutable references of descendants of a given node in
/// post-order.
///
/// This `struct` is created by the `descendants_mut` method on [`Token`]. See
/// its documentation for more.
///
/// [`Token`]: ../struct.Token.html#method.descendants_mut
pub struct DescendantsMutPostord<'a, T: 'a> {
    pub (crate) tree: *mut Tree<T>,
    pub (crate) descendants: DescendantsTokensPostord<'a, T>,
    pub (crate) marker: PhantomData<&'a mut T>
}

macro_rules! descendant_iter {
    (@token struct $name:ident > $func:ident, $field:ident) => {
        impl <'a, T> Iterator for $name<'a, T> {
            type Item = Token;
            fn next(&mut self) -> Option<Token> {
                match self.node_token {
                    None => None,
                    Some(token) => match self.tree.get(token) {
                        None => panic!("Stale token: {:?} is not found in \
                                        the tree. Check code", token),
                        Some(_) => {
                            let (next_node, branch) = $func(
                                token,
                                self.$field,
                                self.branch,
                                self.tree
                            );
                            self.node_token = next_node;
                            self.branch = branch;
                            Some(token)
                        }
                    }
                }
            }
        }
    };

    (@node struct $name:ident) => {
        impl<'a, T> Iterator for $name<'a, T> {
            type Item = &'a Node<T>;
            fn next(&mut self) -> Option<&'a Node<T>> {
                match self.descendants.next() {
                    Some(node_token) => self.tree.get(node_token),
                    None => None
                }
            }
        }
    };

    (@mut struct $name:ident) => {
        impl<'a, T> Iterator for $name<'a, T> {
            type Item = &'a mut Node<T>;
            fn next(&mut self) -> Option<&'a mut Node<T>> {
                match self.descendants.next() {
                    None => None,
                    Some(node_token) => {
                        let tree = unsafe { self.tree.as_mut().unwrap() };
                        match tree.get_mut(node_token) {
                            Some(node) => Some(node),
                            None => None
                        }
                    }
                }
            }
        }
    }
}

descendant_iter!(@token struct DescendantsTokensPreord > preorder_next, subtree_root);
descendant_iter!(@token struct DescendantsTokensPostord > postorder_next, subtree_root);
descendant_iter!(@node struct DescendantsPreord);
descendant_iter!(@node struct DescendantsPostord);
descendant_iter!(@mut struct DescendantsMutPreord);
descendant_iter!(@mut struct DescendantsMutPostord);

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
                    None => None,
                    Some(token) => match self.tree.get(token) {
                        None => panic!("Stale token: {:?} is not found in \
                                        the tree. Check code", token),
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
                    None => None,
                    Some(curr_node_token) => {
                        let tree = unsafe { self.tree.as_mut().unwrap() };
                        match tree.get_mut(curr_node_token) {
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
