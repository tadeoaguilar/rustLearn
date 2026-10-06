//! Module 36 -- 36-performance-optimization. YOUR WORKSPACE.
//!
//! Each `exNN_*.rs` file has the signatures the tests expect, with `todo!()`
//! bodies. Replace each `todo!()` with your implementation, then:
//!
//!     cargo run  -p m36-performance-optimization -- <exercise number>
//!     cargo test -p m36-performance-optimization-tests --features mine
//!
//! Stuck? Compare with ../solution/src -- same file and function names.

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]

pub mod bonus_swar;
pub mod ex01_hotpath;
pub mod ex02_simd;
pub mod ex03_cache;
pub mod ex04_branches;
