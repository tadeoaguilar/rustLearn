//! Module 16 -- 16-web-frameworks. YOUR WORKSPACE.
//!
//! Each `exNN_*.rs` file has the signatures the tests expect, with `todo!()`
//! bodies. Replace each `todo!()` with your implementation, then:
//!
//!     cargo run  -p m16-web-frameworks -- <exercise number>
//!     cargo test -p m16-web-frameworks-tests --features mine
//!
//! Stuck? Compare with ../solution/src -- same file and function names.

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]

pub mod actix_app;
pub mod axum_app;
pub mod blog;
pub mod loadtest;
pub mod middleware;
pub mod notes;
