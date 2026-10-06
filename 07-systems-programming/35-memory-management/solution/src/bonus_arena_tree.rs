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
        Tree {
            nodes: vec![Node {
                value: root,
                parent: None,
                children: Vec::new(),
            }],
        }
    }

    pub fn root(&self) -> NodeId {
        NodeId(0)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        false // there is always a root
    }

    pub fn add_child(&mut self, parent: NodeId, value: T) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node {
            value,
            parent: Some(parent),
            children: Vec::new(),
        });
        self.nodes[parent.0].children.push(id);
        id
    }

    pub fn node(&self, id: NodeId) -> &Node<T> {
        &self.nodes[id.0]
    }

    pub fn value_mut(&mut self, id: NodeId) -> &mut T {
        &mut self.nodes[id.0].value
    }

    /// Edges from the root (the root has depth 0).
    pub fn depth(&self, id: NodeId) -> usize {
        self.path_to_root(id).len() - 1
    }

    /// `id`, its parent, ..., the root.
    pub fn path_to_root(&self, id: NodeId) -> Vec<NodeId> {
        let mut path = vec![id];
        let mut current = id;
        while let Some(parent) = self.nodes[current.0].parent {
            path.push(parent);
            current = parent;
        }
        path
    }

    /// Pre-order traversal of `id`'s subtree (iterative: no recursion depth limit).
    pub fn descendants(&self, id: NodeId) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut stack = vec![id];
        while let Some(n) = stack.pop() {
            out.push(n);
            stack.extend(self.nodes[n.0].children.iter().rev());
        }
        out
    }
}

impl Tree<u64> {
    /// Sum of the values in `id`'s subtree.
    pub fn subtree_sum(&self, id: NodeId) -> u64 {
        self.descendants(id)
            .iter()
            .map(|n| self.nodes[n.0].value)
            .sum()
    }
}
