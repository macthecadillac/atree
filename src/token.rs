use std::marker::PhantomData;

use crate::{Node, Tree};
use crate::iter::*;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Default)]
pub struct Token(pub (crate) usize);

impl Token {
    fn inc(&mut self) { *self = Token(self.0 + 1) }

    /// Insert data into a new node and append to the existing node.
    /// ```
    /// use arena_tree::Tree;
    ///
    /// let root_data = 1usize;
    /// let mut tree = Tree::with_root(root_data);
    /// let root_token = tree.root_node().unwrap().token();
    ///
    /// let next_node_token = root_token.append(&mut tree, 2usize);
    /// next_node_token.append(&mut tree, 3usize);
    /// let root_node = tree.root_node().unwrap();
    /// let mut descendents = root_node.descendents(&tree);
    /// assert_eq!(descendents.next().unwrap().data, 2usize);
    /// assert_eq!(descendents.next().unwrap().data, 3usize);
    /// ```
    // TODO: find ways to put this under impl Node<T>
    pub fn append<T>(self, arena: &mut Tree<T>, data: T) -> Token {
        let new_node_token = arena.next_token;
        arena.next_token.inc();
        let previous_sibling = match self.children_mut(arena).last() {
            Some(last_child) => {
                last_child.next_sibling = Some(new_node_token);
                Some(new_node_token)
            },
            None => {
                let curr_node = arena.get_mut(self).unwrap();
                curr_node.first_child = Some(new_node_token);
                None
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
        arena.set(new_node_token, node);
        new_node_token
    }

    pub fn ancestors_tokens<'a, T>(self, arena: &'a Tree<T>)
        -> AncestorTokens<'a, T> {
        AncestorTokens { arena, node_token: Some(self) }
    }

    pub fn preceding_siblings_tokens<'a, T>(self, arena: &'a Tree<T>)
        -> PrecedingSiblingTokens<'a, T> {
        PrecedingSiblingTokens { arena, node_token: Some(self) }
    }

    pub fn following_siblings_tokens<'a, T>(self, arena: &'a Tree<T>)
        -> FollowingSiblingTokens<'a, T> {
        FollowingSiblingTokens { arena, node_token: Some(self) }
    }

    pub fn children_tokens<'a, T>(self, arena: &'a Tree<T>)
        -> ChildrenTokens<'a, T> {
        let first_child = match arena.get(self) {
            Some(n) => n.first_child,
            None => None
        };
        ChildrenTokens { arena, node_token: first_child }
    }

    pub fn ancestors<'a, T>(self, arena: &'a Tree<T>) -> Ancestors<'a, T> {
        Ancestors { token_iter: self.ancestors_tokens(arena) }
    }

    pub fn preceding_siblings<'a, T>(self, arena: &'a Tree<T>)
        -> PrecedingSiblings<'a, T> {
        PrecedingSiblings { token_iter: self.preceding_siblings_tokens(arena) }
    }

    pub fn following_siblings<'a, T>(self, arena: &'a Tree<T>)
        -> FollowingSiblings<'a, T> {
        FollowingSiblings { token_iter: self.following_siblings_tokens(arena) }
    }

    pub fn children<'a, T>(self, arena: &'a Tree<T>) -> Children<'a, T> {
        Children { token_iter: self.children_tokens(arena) }
    }

    pub fn ancesters_mut<'a, T>(self, arena: &'a mut Tree<T>)
        -> AncestorsMut<'a, T> {
        AncestorsMut {
            arena: arena as *mut Tree<T>,
            node_token: Some(self),
            marker: PhantomData::default()
        }
    }

    pub fn following_siblings_mut<'a, T>(self, arena: &'a mut Tree<T>)
        -> FollowingSiblingsMut<'a, T> {
        FollowingSiblingsMut {
            arena: arena as *mut Tree<T>,
            node_token: Some(self),
            marker: PhantomData::default()
        }
    }

    pub fn preceding_siblings_mut<'a, T>(self, arena: &'a mut Tree<T>)
        -> PrecedingSiblingsMut<'a, T> {
        PrecedingSiblingsMut {
            arena: arena as *mut Tree<T>,
            node_token: Some(self),
            marker: PhantomData::default()
        }
    }

    pub fn children_mut<'a, T>(self, arena: &'a mut Tree<T>)
        -> ChildrenMut<'a, T> {
        let first_child = match arena.get(self) {
            Some(n) => n.first_child,
            None => None
        };
        ChildrenMut {
            arena: arena as *mut Tree<T>,
            node_token: first_child,
            marker: PhantomData::default()
        }
    }

    pub fn descendents_tokens<T>(self, arena: &Tree<T>) -> DescendentTokens {
        fn aux<T>(token: Token, arena: &Tree<T>, acc: &mut Vec<Token>) {
            for child in token.children_tokens(arena) {
                acc.push(child);
                aux(child, arena, acc)
            }
        }

        let mut nodes = Vec::new();
        aux(self, arena, &mut nodes);
        DescendentTokens { nodes, ptr: 0 }
    }

    pub fn descendents<'a, T>(self, arena: &'a Tree<T>) -> Descendents<'a, T> {
        Descendents { arena, descendents: self.descendents_tokens(arena) }
    }

    pub fn remove_descendents<T>(&mut self, arena: &mut Tree<T>) {
        let descendents = self.descendents_tokens(arena);
        for node_token in descendents {
            arena.arena.remove(node_token);
        }
    }
}

