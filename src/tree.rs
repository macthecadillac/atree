#![allow(clippy::uninit_assumed_init)]
use std::mem::MaybeUninit;
use std::collections::HashMap;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Default)]
pub struct NodeId(usize);

impl NodeId {
    fn inc(&mut self) { *self = NodeId(self.0 + 1) }
}

pub struct Node<'a, T> {
    data: T,
    arena: &'a mut Tree<'a, T>,
    next_sibling: Option<NodeId>,
    first_child: Option<NodeId>,
}

#[derive(Default)]
pub struct Tree<'a, T> {
    next_id: NodeId,
    arena: HashMap<NodeId, Node<'a, T>>
}

impl<'a, T> Tree<'a, T> {
    pub fn get(&'a self, indx: NodeId) -> Option<&'a Node<'a, T>> {
        self.arena.get(&indx)
    }

    pub fn get_mut(&'a mut self, indx: NodeId) -> Option<&'a mut Node<'a, T>> {
        self.arena.get_mut(&indx)
    }

    pub fn set(&'a mut self, indx: NodeId, node: Node<'a, T>) {
        if let Some(mut n) = self.arena.insert(indx, node) {
            n.remove_descendents()
        }
    }

    pub fn remove(&'a mut self, indx: NodeId) {
        if let Some(mut n) = self.arena.remove(&indx) {
            n.remove_descendents()
        }
    }

    pub fn shrink_to_fit(&'a mut self) { self.arena.shrink_to_fit() }
}

impl<'a, T> Node<'a, T> {
    pub fn append(&'a mut self, data: T) {
        let node_id = self.arena.next_id;
        self.arena.next_id.inc();
        if self.first_child.is_none() {
            self.first_child = Some(node_id)
        } else if let Some(indx) = self.children().last() {
            // should not fail
            let mut n = self.arena.get_mut(indx).unwrap();
            n.next_sibling = Some(node_id);
        }
        let node = Node {
            data,
            arena: self.arena,
            next_sibling: None,
            first_child: None
        };
        self.arena.set(node_id, node)
    }

    pub fn siblings(&'a self) -> Siblings<'a, T> { Siblings { node: self } }

    pub fn children(&'a self) -> Children<'a, T> { Children { node: self } }

    pub fn descendents(&'a self) -> Descendents {
        fn aux<'a, T>(node: &'a Node<'a, T>, acc: &mut Vec<NodeId>) {
            for child in node.children() {
                acc.push(child);
                let child_node = node.arena.get(child).unwrap();
                aux(child_node, acc)
            }
        }

        let mut nodes = Vec::new();
        aux(self, &mut nodes);
        Descendents { nodes, ptr: 0 }
    }

    pub fn remove_descendents(&mut self) {
        // we need to use a dummy node to get around mutable borrow after an
        // immutable borrow. That's the price (in verbosity) we pay for
        // including a reference to the tree itself from the node. The "unsafe"
        // is here is entirely safe since we never touch the uninitialized
        // fields
        let mut empty_arena: Tree<T>;
        let dummy_node = unsafe {
            empty_arena = MaybeUninit::uninit().assume_init();
            let data = MaybeUninit::uninit().assume_init();
            Node { data, arena: &mut empty_arena, ..*self }
        };

        let descendents = dummy_node.descendents();
        for node_id in descendents {
            self.arena.arena.remove(&node_id);
        }
    }
}

pub struct Descendents { nodes: Vec<NodeId>, ptr: usize }

impl Iterator for Descendents {
    type Item = NodeId;
    fn next(&mut self) -> Option<NodeId> {
        match self.nodes.get(self.ptr) {
            None => None,
            Some(&node_id) => {
                self.ptr += 1;
                Some(node_id)
            }
        }
    }
}

pub struct Siblings<'a, T> { node: &'a Node<'a, T> }

impl<'a, T> Iterator for Siblings<'a, T> {
    type Item = NodeId;
    fn next(&mut self) -> Option<NodeId> {
        let next_node = self.node.next_sibling;
        match next_node {
            Some(indx) => {
                // should not fail
                self.node = &self.node.arena.get(indx).unwrap();
                Some(indx)
            },
            None => None
        }
    }
}

pub struct Children<'a, T> { node: &'a Node<'a, T> }

impl<'a, T> Iterator for Children<'a, T> {
    type Item = NodeId;
    fn next(&mut self) -> Option<NodeId> {
        let next_node = self.node.first_child;
        match next_node {
            Some(indx) => {
                // should not fail
                self.node = &self.node.arena.get(indx).unwrap();
                Some(indx)
            },
            None => None
        }
    }
}
