//! Module 18 -- 18-databases. YOUR WORKSPACE.
//!
//! Each `exNN_*.rs` file has the signatures the tests expect, with `todo!()`
//! bodies. Replace each `todo!()` with your implementation, then:
//!
//!     cargo run  -p m18-databases -- <exercise number>
//!     cargo test -p m18-databases-tests --features mine
//!
//! Stuck? Compare with ../solution/src -- same file and function names.

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]

pub mod db;
pub mod indexes;
pub mod keyset;
pub mod n_plus_one;
pub mod queries;
pub mod repository;
pub mod transactions;
