//! Module 39 -- 39-compiler-internals. YOUR WORKSPACE.
//!
//! Each `exNN_*.rs` file has the signatures the tests expect, with `todo!()`
//! bodies. Replace each `todo!()` with your implementation, then:
//!
//!     cargo run  -p m39-compiler-internals -- <exercise number>
//!     cargo test -p m39-compiler-internals-tests --features mine
//!
//! Stuck? Compare with ../solution/src -- same file and function names.

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]

pub mod bonus_unused;
pub mod ex01_lint;
pub mod ex02_mir;
pub mod ex03_borrowck;
pub mod ex04_desugar;
pub mod toolchain;
