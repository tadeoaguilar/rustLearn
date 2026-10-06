//! Bonus: a tree in an arena -- indices instead of `Rc<RefCell<..>>`.
//!
//! A tree with parent links is awkward with references (cycles) and slow
//! with `Rc<RefCell<..>>` (an allocation and a refcount per node). Storing
//! nodes in one `Vec` and linking them by index is the usual Rust answer:
//! one allocation, cache-friendly, no lifetimes, and the borrow checker is
//! happy because nobody holds a reference into the `Vec` while it grows.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub usize);

#[derive(Debug, Clone)]
pub struct Node<T> {
    pub value: T,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
}

#[derive(Debug, Clone)]
pub struct Tree<T> {
    nodes: Vec<Node<T>>,
}

impl<T> Tree<T> {
    /// A tree with just a root.
    pub fn new(root: T) -> Self {
        todo!("Bonus")
    }

    pub fn root(&self) -> NodeId {
        todo!("Bonus")
    }

    pub fn len(&self) -> usize {
        todo!("Bonus")
    }

    pub fn is_empty(&self) -> bool {
        todo!("Bonus")
    }

    pub fn add_child(&mut self, parent: NodeId, value: T) -> NodeId {
        todo!("Bonus")
    }

    pub fn node(&self, id: NodeId) -> &Node<T> {
        todo!("Bonus")
    }

    pub fn value_mut(&mut self, id: NodeId) -> &mut T {
        todo!("Bonus")
    }

    /// Edges from the root (the root has depth 0).
    pub fn depth(&self, id: NodeId) -> usize {
        todo!("Bonus")
    }

    /// `id`, its parent, ..., the root.
    pub fn path_to_root(&self, id: NodeId) -> Vec<NodeId> {
        todo!("Bonus")
    }

    /// Pre-order traversal of `id`'s subtree (iterative: no recursion depth limit).
    pub fn descendants(&self, id: NodeId) -> Vec<NodeId> {
        todo!("Bonus")
    }
}

impl Tree<u64> {
    /// Sum of the values in `id`'s subtree.
    pub fn subtree_sum(&self, id: NodeId) -> u64 {
        todo!("Bonus")
    }
}
