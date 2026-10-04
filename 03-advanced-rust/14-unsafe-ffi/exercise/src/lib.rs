//! Module 14 -- 14-unsafe-ffi. YOUR WORKSPACE.
//!
//! Each `exNN_*.rs` file has the signatures the tests expect, with `todo!()`
//! bodies. Replace each `todo!()` with your implementation, then:
//!
//!     cargo run  -p m14-unsafe-ffi -- <exercise number>
//!     cargo test -p m14-unsafe-ffi-tests --features mine
//!
//! Stuck? Compare with ../solution/src -- same file and function names.

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]
// Rust 2024 makes this lint warn by default; deny it to be sure every unsafe
// operation inside an `unsafe fn` sits in its own explicit `unsafe {}` block.
#![deny(unsafe_op_in_unsafe_fn)]

pub mod ex01_raw_pointers;
pub mod ex02_libc;
pub mod ex03_c_library;
pub mod ex04_rust_from_c;
pub mod ex05_stack_vec;
pub mod ex06_allocator;
pub mod ex07_system;
