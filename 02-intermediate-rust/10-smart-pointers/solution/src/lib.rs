//! Module 10 -- Smart Pointers. Reference solution.
//!
//! A smart pointer is a struct that owns or points to data *and* adds
//! behaviour: `Box` puts it on the heap, `Rc`/`Arc` count owners, `RefCell`
//! and `Mutex` check borrows at runtime, `Weak` points without owning.
//! What makes them "pointers" is the `Deref` trait (Exercise 3); what makes
//! them "smart" is usually `Drop`.

pub mod bonus_cow;
pub mod ex01_box;
pub mod ex02_linked_list;
pub mod ex03_deref_drop;
pub mod ex04_rc_weak_tree;
pub mod ex05_refcell;
pub mod ex06_graph;
pub mod ex07_shared_cache;
