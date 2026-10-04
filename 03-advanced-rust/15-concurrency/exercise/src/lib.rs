//! Module 15 -- 15-concurrency. YOUR WORKSPACE.
//!
//! Each `exNN_*.rs` file has the signatures the tests expect, with `todo!()`
//! bodies. Replace each `todo!()` with your implementation, then:
//!
//!     cargo run  -p m15-concurrency -- <exercise number>
//!     cargo test -p m15-concurrency-tests --features mine
//!
//! Stuck? Compare with ../solution/src -- same file and function names.

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]

pub mod bonus_philosophers;
pub mod ex01_threads;
pub mod ex02_channels;
pub mod ex03_shared_state;
pub mod ex04_atomics;
pub mod ex05_thread_pool;
pub mod ex06_rayon;
pub mod ex07_crawler;
pub mod ex08_lock_free;
