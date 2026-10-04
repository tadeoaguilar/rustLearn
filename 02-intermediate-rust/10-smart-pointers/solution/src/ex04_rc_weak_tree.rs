//! Exercise 4: Rc and Weak -- a tree with parent links.
//!
//! Ownership flows *down*: a parent owns its children through `Rc`. The
//! parent link points *up* with `Weak`, which doesn't keep the parent alive.
//! If it were an `Rc`, parent and child would own each other: a cycle, and
//! neither count could ever reach zero -- a memory leak.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub struct TreeNode {
    pub value: String,
    // RefCell: we set the parent *after* the node is already shared in an Rc.
    parent: RefCell<Weak<TreeNode>>,
    children: RefCell<Vec<Rc<TreeNode>>>,
}

impl TreeNode {
    pub fn new(value: &str) -> Rc<TreeNode> {
        Rc::new(TreeNode {
            value: value.to_string(),
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(Vec::new()),
        })
    }

    pub fn add_child(parent: &Rc<TreeNode>, child: Rc<TreeNode>) {
        *child.parent.borrow_mut() = Rc::downgrade(parent);
        parent.children.borrow_mut().push(child);
    }

    /// `upgrade` turns the Weak into an Rc if the parent is still alive.
    pub fn parent(&self) -> Option<Rc<TreeNode>> {
        self.parent.borrow().upgrade()
    }

    pub fn children(&self) -> Vec<Rc<TreeNode>> {
        self.children.borrow().clone() // clones the Rcs, not the nodes
    }

    pub fn depth(&self) -> usize {
        let mut depth = 0;
        let mut current = self.parent();
        while let Some(node) = current {
            depth += 1;
            current = node.parent();
        }
        depth
    }

    /// "root/usr/bin": walk up collecting values, then reverse.
    pub fn path(&self) -> String {
        let mut parts = vec![self.value.clone()];
        let mut current = self.parent();
        while let Some(node) = current {
            parts.push(node.value.clone());
            current = node.parent();
        }
        parts.reverse();
        parts.join("/")
    }

    /// Depth-first search; returns a new owner of the found node.
    pub fn find(node: &Rc<TreeNode>, value: &str) -> Option<Rc<TreeNode>> {
        if node.value == value {
            return Some(Rc::clone(node));
        }
        node.children
            .borrow()
            .iter()
            .find_map(|child| TreeNode::find(child, value))
    }
}

/// /usr/bin, /usr/lib, /home
pub fn sample_tree() -> Rc<TreeNode> {
    let root = TreeNode::new("root");
    let usr = TreeNode::new("usr");
    TreeNode::add_child(&root, Rc::clone(&usr));
    TreeNode::add_child(&usr, TreeNode::new("bin"));
    TreeNode::add_child(&usr, TreeNode::new("lib"));
    TreeNode::add_child(&root, TreeNode::new("home"));
    root
}

pub fn run() {
    let root = sample_tree();
    let bin = TreeNode::find(&root, "bin").unwrap();
    println!("{} at depth {}", bin.path(), bin.depth());

    let usr = bin.parent().unwrap();
    println!(
        "usr: strong = {} (root's child list + our `usr` variable), weak = {} (bin's and lib's parent links)",
        Rc::strong_count(&usr),
        Rc::weak_count(&usr)
    );
    drop(usr);

    drop(root);
    println!(
        "after dropping root, bin.parent() = {:?}",
        bin.parent().map(|p| p.value.clone())
    );
    println!("(usr was freed: only Weak links pointed to it once root's child list was gone)");
}
