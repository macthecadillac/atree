use std::collections::HashMap;
use std::marker::PhantomData;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Default)]
pub struct NodeId(usize);

impl NodeId {
    fn inc(&mut self) { *self = NodeId(self.0 + 1) }
}

pub struct Node<T> {
    pub data: T,
    id: NodeId,
    next_sibling: Option<NodeId>,
    first_child: Option<NodeId>,
}

#[derive(Default)]
pub struct Tree<T> {
    next_id: NodeId,
    arena: HashMap<NodeId, Node<T>>
}

impl<T> Tree<T> {
    pub fn get(&self, indx: NodeId) -> Option<&Node<T>> {
        self.arena.get(&indx)
    }

    pub fn get_mut(&mut self, indx: NodeId) -> Option<&mut Node<T>> {
        self.arena.get_mut(&indx)
    }

    pub fn set(&mut self, indx: NodeId, node: Node<T>) {
        if let Some(mut n) = self.arena.insert(indx, node) {
            n.remove_descendents(self)
        }
    }

    pub fn remove(&mut self, indx: NodeId) {
        if let Some(mut n) = self.arena.remove(&indx) {
            n.remove_descendents(self)
        }
    }

    pub fn shrink_to_fit(&mut self) { self.arena.shrink_to_fit() }
}

impl<T> Node<T> {
    pub fn append(&mut self, arena: &mut Tree<T>, data: T) {
        let node_id = arena.next_id;
        arena.next_id.inc();
        if self.first_child.is_none() {
            self.first_child = Some(node_id)
        } else if let Some(last_child) = self.children_mut(arena).last() {
            last_child.next_sibling = Some(node_id);
        }
        let node = Node {
            data,
            id: node_id,
            next_sibling: None,
            first_child: None
        };
        arena.set(node_id, node)
    }

    pub fn siblings_ids<'a>(&self, arena: &'a Tree<T>) -> SiblingIDs<'a, T> {
        SiblingIDs { arena, node_id: Some(self.id) }
    }

    pub fn children_ids<'a>(&self, arena: &'a Tree<T>) -> ChildrenIDs<'a, T> {
        ChildrenIDs { arena, node_id: self.first_child }
    }

    pub fn siblings<'a>(&self, arena: &'a Tree<T>) -> Siblings<'a, T> {
        Siblings { id_iter: self.siblings_ids(arena) }
    }

    pub fn children<'a>(&self, arena: &'a Tree<T>) -> Children<'a, T> {
        Children { id_iter: self.children_ids(arena) }
    }

    pub fn children_mut<'a>(&self, arena: &'a mut Tree<T>) -> ChildrenMut<'a, T> {
        ChildrenMut {
            arena: arena as *mut Tree<T>,
            node_id: self.first_child,
            marker: PhantomData::default()
        }
    }

    pub fn siblings_mut<'a>(&self, arena: &'a mut Tree<T>) -> SiblingsMut<'a, T> {
        SiblingsMut {
            arena: arena as *mut Tree<T>,
            node_id: Some(self.id),
            marker: PhantomData::default()
        }
    }

    pub fn descendents_ids(&self, arena: &Tree<T>) -> DescendentIDs {
        fn aux<T>(node: &Node<T>, arena: &Tree<T>, acc: &mut Vec<NodeId>) {
            for child in node.children_ids(arena) {
                acc.push(child);
                let child_node = arena.get(child).unwrap();
                aux(child_node, arena, acc)
            }
        }

        let mut nodes = Vec::new();
        aux(self, arena, &mut nodes);
        DescendentIDs { nodes, ptr: 0 }
    }

    pub fn remove_descendents(&mut self, arena: &mut Tree<T>) {
        let descendents = self.descendents_ids(arena);
        for node_id in descendents {
            arena.arena.remove(&node_id);
        }
    }
}

pub struct DescendentIDs { nodes: Vec<NodeId>, ptr: usize }

impl Iterator for DescendentIDs {
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

pub struct Descendents<'a, T> { arena: &'a Tree<T>, descendents: DescendentIDs }

impl<'a, T> Iterator for Descendents<'a, T> {
    type Item = &'a Node<T>;
    fn next(&mut self) -> Option<&'a Node<T>> {
        match self.descendents.next() {
            Some(node_id) => self.arena.get(node_id),
            None => None
        }
    }
}

pub struct SiblingIDs<'a, T> { arena: &'a Tree<T>, node_id: Option<NodeId> }

pub struct ChildrenIDs<'a, T> { arena: &'a Tree<T>, node_id: Option<NodeId> }

pub struct Siblings<'a, T> { id_iter: SiblingIDs<'a, T> }

pub struct Children<'a, T> { id_iter: ChildrenIDs<'a, T> }

pub struct SiblingsMut<'a, T: 'a> {
    arena: *mut Tree<T>,
    node_id: Option<NodeId>,
    marker: PhantomData<&'a mut T>
}

pub struct ChildrenMut<'a, T: 'a> {
    arena: *mut Tree<T>,
    node_id: Option<NodeId>,
    marker: PhantomData<&'a mut T>
}

macro_rules! iterator {
    (@id struct $name:ident) => {
        impl<'a, T> Iterator for $name<'a, T> {
            type Item = NodeId;
            fn next(&mut self) -> Option<NodeId> {
                match self.node_id {
                    Some(curr_node_id) => {
                        // unwrap should not fail
                        let curr_node = self.arena.get(curr_node_id).unwrap();
                        self.node_id = curr_node.next_sibling;
                        Some(curr_node_id)
                    },
                    None => None
                }
            }
        }
    };

    // perhaps fold this into the @id branch since this can be implemented with
    // largely the same code with one less Tree::get
    (@node struct $name:ident) => {
        impl<'a, T> Iterator for $name<'a, T> {
            type Item = &'a Node<T>;
            fn next(&mut self) -> Option<&'a Node<T>> {
                match self.id_iter.next() {
                    Some(node_id) => self.id_iter.arena.get(node_id),
                    None => None
                }
            }
        }
    };

    (@mut struct $name:ident) => {
        impl<'a, T> Iterator for $name<'a, T> {
            type Item = &'a mut Node<T>;
            fn next(&mut self) -> Option<&'a mut Node<T>> {
                match self.node_id {
                    Some(curr_node_id) => {
                        let arena = unsafe { self.arena.as_mut().unwrap() };
                        // unwrap should not fail
                        let curr_node = arena.get_mut(curr_node_id).unwrap();
                        self.node_id = curr_node.next_sibling;
                        Some(curr_node)
                    },
                    None => None
                }
            }
        }
    }
}

iterator!(@id struct SiblingIDs);
iterator!(@id struct ChildrenIDs);
iterator!(@node struct Siblings);
iterator!(@node struct Children);
iterator!(@mut struct SiblingsMut);
iterator!(@mut struct ChildrenMut);
