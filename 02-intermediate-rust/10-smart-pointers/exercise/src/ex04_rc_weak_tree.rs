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
        todo!("Exercise 4")
    }

    pub fn add_child(parent: &Rc<TreeNode>, child: Rc<TreeNode>) {
        todo!("Exercise 4")
    }

    /// `upgrade` turns the Weak into an Rc if the parent is still alive.
    pub fn parent(&self) -> Option<Rc<TreeNode>> {
        todo!("Exercise 4")
    }

    pub fn children(&self) -> Vec<Rc<TreeNode>> {
        todo!("Exercise 4")
    }

    pub fn depth(&self) -> usize {
        todo!("Exercise 4")
    }

    /// "root/usr/bin": walk up collecting values, then reverse.
    pub fn path(&self) -> String {
        todo!("Exercise 4")
    }

    /// Depth-first search; returns a new owner of the found node.
    pub fn find(node: &Rc<TreeNode>, value: &str) -> Option<Rc<TreeNode>> {
        todo!("Exercise 4")
    }
}

/// /usr/bin, /usr/lib, /home
pub fn sample_tree() -> Rc<TreeNode> {
    todo!("Exercise 4")
}

pub fn run() {
    todo!("Exercise 4")
}
