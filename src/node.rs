use crate::Tree;
use crate::token::Token;
use crate::iter::*;

#[derive(Default)]
pub struct Node<T> {
    pub data: T,
    pub (crate) token: Token,
    pub (crate) parent: Option<Token>,
    pub (crate) previous_sibling: Option<Token>,
    pub (crate) next_sibling: Option<Token>,
    pub (crate) first_child: Option<Token>,
}

impl<T> Node<T> {
    pub fn token(&self) -> Token { self.token }

    pub fn ancestors_tokens<'a>(&self, arena: &'a Tree<T>)
        -> AncestorTokens<'a, T> {
        self.token.ancestors_tokens(arena)
    }

    pub fn preceding_siblings_tokens<'a>(&self, arena: &'a Tree<T>)
        -> PrecedingSiblingTokens<'a, T> {
        self.token.preceding_siblings_tokens(arena)
    }

    pub fn following_siblings_tokens<'a>(&self, arena: &'a Tree<T>)
        -> FollowingSiblingTokens<'a, T> {
        self.token.following_siblings_tokens(arena)
    }

    pub fn children_tokens<'a>(&self, arena: &'a Tree<T>)
        -> ChildrenTokens<'a, T> {
        self.token.children_tokens(arena)
    }

    pub fn ancestors<'a>(&self, arena: &'a Tree<T>)
        -> Ancestors<'a, T> {
        self.token.ancestors(arena)
    }

    pub fn following_siblings<'a>(&self, arena: &'a Tree<T>)
        -> FollowingSiblings<'a, T> {
        self.token.following_siblings(arena)
    }

    pub fn preceding_siblings<'a>(&self, arena: &'a Tree<T>)
        -> PrecedingSiblings<'a, T> {
        self.token.preceding_siblings(arena)
    }

    pub fn children<'a>(&self, arena: &'a Tree<T>) -> Children<'a, T> {
        self.token.children(arena)
    }

    pub fn descendents_tokens(&self, arena: &Tree<T>) -> DescendentTokens {
        self.token.descendents_tokens(arena)
    }

    pub fn descendents<'a>(&self, arena: &'a Tree<T>) -> Descendents<'a, T> {
        self.token.descendents(arena)
    }

    pub fn remove_descendents(&mut self, arena: &mut Tree<T>) {
        self.token.remove_descendents(arena)
    }
}
