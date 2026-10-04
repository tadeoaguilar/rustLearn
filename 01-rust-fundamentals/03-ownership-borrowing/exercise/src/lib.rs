//! Module 03 -- 03-ownership-borrowing. YOUR WORKSPACE.
//!
//! Each `exNN_*.rs` file has the signatures the tests expect, with `todo!()`
//! bodies. Replace each `todo!()` with your implementation, then:
//!
//!     cargo run  -p m03-ownership-borrowing -- <exercise number>
//!     cargo test -p m03-ownership-borrowing-tests --features mine
//!
//! Stuck? Compare with ../solution/src -- same file and function names.

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]
// While the bodies are todo!(), clippy suggests &mut [T] / &mut str for
// parameters that the real implementation needs as &mut Vec / &mut String.
#![allow(clippy::ptr_arg)]

pub mod bonus_simple_rc;
pub mod ex01_ownership;
pub mod ex02_references;
pub mod ex03_strings;
pub mod ex04_dangling;
pub mod ex05_clone_copy;
pub mod ex06_borrow_rules;
pub mod ex07_text_buffer;
pub mod ex08_collections;
