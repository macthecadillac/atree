// TODO: see if the custom arena provides better performance than HashMap (it
// should)
// TODO: add tree merging capabilities
// TODO: add tree spliting functions

mod arena;
pub mod iter;
mod node;
mod token;
use arena::Arena;
pub use token::Token;
pub use node::Node;

#[derive(Default)]
pub struct Tree<T> {
    next_token: Token,
    arena: Arena<Node<T>>
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
            token: Token::default(),
            next_sibling: None,
            first_child: None
        };
        let mut arena = Arena::new();
        arena.insert(root_node);
        Tree { next_token: Token(1), arena }
    }

    /// Create root node on tree with data. Will erase all data if tree wasn't
    /// emtpy to begin with
    pub fn initialize(&mut self, data: T) { *self = Tree::with_root(data) }

    pub fn get(&self, indx: Token) -> Option<&Node<T>> {
        self.arena.get(indx)
    }

    pub fn get_mut(&mut self, indx: Token) -> Option<&mut Node<T>> {
        self.arena.get_mut(indx)
    }

    /// will remove all descendents of a node if it wasn't empty
    pub fn set(&mut self, indx: Token, node: Node<T>) {
        if let Some(mut n) = self.arena.set(indx, node) {
            n.remove_descendents(self)
        }
    }

    /// Remove node and all its descendents
    pub fn remove(&mut self, indx: Token) {
        if let Some(mut n) = self.arena.remove(indx) {
            n.remove_descendents(self)
        }
    }

    // pub fn shrink_to_fit(&mut self) { self.arena.shrink_to_fit() }
}
