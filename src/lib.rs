// TODO: add tree merging capabilities
// TODO: add tree spliting functions
use std::collections::HashMap;
use std::iter;
use std::marker::PhantomData;

// use ahash for performance reasons
extern crate ahash;
use ahash::ABuildHasher;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Default)]
pub struct Token(usize);

impl Token {
    fn inc(&mut self) { *self = Token(self.0 + 1) }

    /// Insert data into a new node and append to the existing node.
    /// ```
    /// use arena_tree;
    /// let root_data = 1usize;
    /// let mut tree = Tree::with_root(root_data);
    /// let root_token = tree.root_node().unwrap().token();
    ///
    /// root_token.append(&mut tree, 2usize);
    /// let root_node = tree.root_node().unwrap();
    /// assert_eq!(root_node.descendents(&tree).next().unwrap().data, 2usize)
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
            arena.arena.remove(&node_token);
        }
    }
}

pub struct Node<T> {
    pub data: T,
    token: Token,
    parent: Option<Token>,
    previous_sibling: Option<Token>,
    next_sibling: Option<Token>,
    first_child: Option<Token>,
}

#[derive(Default)]
pub struct Tree<T> {
    next_token: Token,
    arena: HashMap<Token, Node<T>, ABuildHasher>
}

impl<T> Tree<T> {
    pub fn root_node(&self) -> Option<&Node<T>> { self.get(Token(0)) }

    pub fn root_node_mut(&mut self) -> Option<&mut Node<T>> {
        self.get_mut(Token(0))
    }

    /// Create tree with data at the root node
    pub fn with_root(data: T) -> Self {
        let root_node = Node {
            data,
            parent: None,
            previous_sibling: None,
            token: Token(0),
            next_sibling: None,
            first_child: None
        };

        Tree {
            next_token: Token(1),
            arena: iter::once((Token(0), root_node)).collect()
        }
    }

    /// Create root node on tree with data. Will erase all data if tree wasn't
    /// emtpy to begin with
    pub fn initialize(&mut self, data: T) { *self = Tree::with_root(data) }

    pub fn get(&self, indx: Token) -> Option<&Node<T>> {
        self.arena.get(&indx)
    }

    pub fn get_mut(&mut self, indx: Token) -> Option<&mut Node<T>> {
        self.arena.get_mut(&indx)
    }

    /// will remove all descendents of a node if it wasn't empty
    pub fn set(&mut self, indx: Token, node: Node<T>) {
        if let Some(mut n) = self.arena.insert(indx, node) {
            n.remove_descendents(self)
        }
    }

    /// Remove node and all its descendents
    pub fn remove(&mut self, indx: Token) {
        if let Some(mut n) = self.arena.remove(&indx) {
            n.remove_descendents(self)
        }
    }

    pub fn shrink_to_fit(&mut self) { self.arena.shrink_to_fit() }
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

pub struct DescendentTokens { nodes: Vec<Token>, ptr: usize }

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
    arena: &'a Tree<T>,
    descendents: DescendentTokens
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
    arena: &'a Tree<T>,
    node_token: Option<Token>
}

pub struct PrecedingSiblingTokens<'a, T> {
    arena: &'a Tree<T>,
    node_token: Option<Token>
}

pub struct ChildrenTokens<'a, T> {
    arena: &'a Tree<T>,
    node_token: Option<Token>
}

pub struct AncestorTokens<'a, T> {
    arena: &'a Tree<T>,
    node_token: Option<Token>
}

pub struct PrecedingSiblings<'a, T> {
    token_iter: PrecedingSiblingTokens<'a, T>
}

pub struct FollowingSiblings<'a, T> {
    token_iter: FollowingSiblingTokens<'a, T>
}

pub struct Children<'a, T> { token_iter: ChildrenTokens<'a, T> }

pub struct Ancestors<'a, T> { token_iter: AncestorTokens<'a, T> }

pub struct PrecedingSiblingsMut<'a, T: 'a> {
    arena: *mut Tree<T>,
    node_token: Option<Token>,
    marker: PhantomData<&'a mut T>
}

pub struct FollowingSiblingsMut<'a, T: 'a> {
    arena: *mut Tree<T>,
    node_token: Option<Token>,
    marker: PhantomData<&'a mut T>
}

pub struct ChildrenMut<'a, T: 'a> {
    arena: *mut Tree<T>,
    node_token: Option<Token>,
    marker: PhantomData<&'a mut T>
}

pub struct AncestorsMut<'a, T: 'a> {
    arena: *mut Tree<T>,
    node_token: Option<Token>,
    marker: PhantomData<&'a mut T>
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
