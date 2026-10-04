//! Module 13 -- Macros. YOUR WORKSPACE.
//!
//! Exercises 1-5: write `macro_rules!` macros in ex01_basics.rs ... ex05_hygiene.rs.
//! Exercises 6-7 and the bonus: write procedural macros in ../derive/src/lib.rs.
//!
//! Every macro you write needs `#[macro_export]`, which places it at the crate
//! root: the tests call `m13_macros::square!`, not `m13_macros::ex01_basics::square!`.
//!
//! The tests for each exercise only compile once that exercise's macros
//! exist, so they're switched on one exercise at a time:
//!
//!     cargo test -p m13-macros-tests --features mine,ex1
//!     cargo test -p m13-macros-tests --features mine,ex1,ex2,ex3
//!
//! Stuck? Compare with ../solution -- same files, same names.

#![allow(unused)]

pub use m13_macros_derive::{Builder, Describe, timed};

pub mod bonus_attribute;
pub mod ex01_basics;
pub mod ex02_items;
pub mod ex03_validation;
pub mod ex04_dsl;
pub mod ex05_hygiene;
pub mod ex06_derive;
pub mod ex07_builder;
