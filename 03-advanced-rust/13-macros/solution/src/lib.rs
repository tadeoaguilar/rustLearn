//! Module 13 -- Macros. Reference solution.
//!
//! Declarative macros (`macro_rules!`) are in ex01-ex05. `#[macro_export]`
//! puts each one at the *crate root*, whatever module defines it:
//! `m13_macros_solution::hashmap!`, not `...::ex01_basics::hashmap!`.
//!
//! The procedural macros live in the `derive/` crate (proc-macro crates can
//! export nothing else) and are re-exported here, so users need one dependency.

pub use m13_macros_derive_solution::{Builder, Describe, timed};

pub mod bonus_attribute;
pub mod ex01_basics;
pub mod ex02_items;
pub mod ex03_validation;
pub mod ex04_dsl;
pub mod ex05_hygiene;
pub mod ex06_derive;
pub mod ex07_builder;
