use std::marker::PhantomData;

use crate::Tree;
use crate::node::Node;
use crate::token::Token;

pub struct DescendentTokens {
    pub (crate) nodes: Vec<Token>,
    pub (crate) ptr: usize
}

impl Iterator for DescendentTokens {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        match self.nodes.get(self.ptr) {
            None => None,
            Some(&node_token) => {
                self.ptr += 1;
                Some(node_token)
            }
        }
    }
}

pub struct Descendents<'a, T> {
    pub (crate) arena: &'a Tree<T>,
    pub (crate) descendents: DescendentTokens
}

impl<'a, T> Iterator for Descendents<'a, T> {
    type Item = &'a Node<T>;
    fn next(&mut self) -> Option<&'a Node<T>> {
        match self.descendents.next() {
            Some(node_token) => self.arena.get(node_token),
            None => None
        }
    }
}

pub struct FollowingSiblingTokens<'a, T> {
    pub (crate) arena: &'a Tree<T>,
    pub (crate) node_token: Option<Token>
}

pub struct PrecedingSiblingTokens<'a, T> {
    pub (crate) arena: &'a Tree<T>,
    pub (crate) node_token: Option<Token>
}

pub struct ChildrenTokens<'a, T> {
    pub (crate) arena: &'a Tree<T>,
    pub (crate) node_token: Option<Token>
}

pub struct AncestorTokens<'a, T> {
    pub (crate) arena: &'a Tree<T>,
    pub (crate) node_token: Option<Token>
}

pub struct PrecedingSiblings<'a, T> {
    pub (crate) token_iter: PrecedingSiblingTokens<'a, T>
}

pub struct FollowingSiblings<'a, T> {
    pub (crate) token_iter: FollowingSiblingTokens<'a, T>
}

pub struct Children<'a, T> {
    pub (crate) token_iter: ChildrenTokens<'a, T>
}

pub struct Ancestors<'a, T> {
    pub (crate) token_iter: AncestorTokens<'a, T>
}

pub struct PrecedingSiblingsMut<'a, T: 'a> {
    pub (crate) arena: *mut Tree<T>,
    pub (crate) node_token: Option<Token>,
    pub (crate) marker: PhantomData<&'a mut T>
}

pub struct FollowingSiblingsMut<'a, T: 'a> {
    pub (crate) arena: *mut Tree<T>,
    pub (crate) node_token: Option<Token>,
    pub (crate) marker: PhantomData<&'a mut T>
}

pub struct ChildrenMut<'a, T: 'a> {
    pub (crate) arena: *mut Tree<T>,
    pub (crate) node_token: Option<Token>,
    pub (crate) marker: PhantomData<&'a mut T>
}

pub struct AncestorsMut<'a, T: 'a> {
    pub (crate) arena: *mut Tree<T>,
    pub (crate) node_token: Option<Token>,
    pub (crate) marker: PhantomData<&'a mut T>
}

macro_rules! iterator {
    (@id struct $name:ident > $field:ident) => {
        impl<'a, T> Iterator for $name<'a, T> {
            type Item = Token;
            fn next(&mut self) -> Option<Token> {
                match self.node_token {
                    Some(curr_node_token) => {
                        // unwrap should not fail
                        let curr_node = self.arena.get(curr_node_token).unwrap();
                        self.node_token = curr_node.$field;
                        Some(curr_node_token)
                    },
                    None => None
                }
            }
        }
    };

    // perhaps fold this into the @id branch since this can be implemented with
    // largely the same code with one less Tree::get (one less look-up should
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
                    Some(curr_node_token) => {
                        let arena = unsafe { self.arena.as_mut().unwrap() };
                        match arena.get_mut(curr_node_token) {
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

iterator!(@id struct FollowingSiblingTokens > next_sibling);
iterator!(@id struct PrecedingSiblingTokens > previous_sibling);
iterator!(@id struct ChildrenTokens > first_child);
iterator!(@id struct AncestorTokens > parent);
iterator!(@node struct PrecedingSiblings);
iterator!(@node struct FollowingSiblings);
iterator!(@node struct Children);
iterator!(@node struct Ancestors);
iterator!(@mut struct PrecedingSiblingsMut > previous_sibling);
iterator!(@mut struct FollowingSiblingsMut > next_sibling);
iterator!(@mut struct ChildrenMut > first_child);
iterator!(@mut struct AncestorsMut > parent);
