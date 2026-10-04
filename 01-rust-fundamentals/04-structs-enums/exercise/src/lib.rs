//! Module 04 -- 04-structs-enums. YOUR WORKSPACE.
//!
//! Each `exNN_*.rs` file has the signatures the tests expect, with `todo!()`
//! bodies. Replace each `todo!()` with your implementation, then:
//!
//!     cargo run  -p m04-structs-enums -- <exercise number>
//!     cargo test -p m04-structs-enums-tests --features mine
//!
//! Stuck? Compare with ../solution/src -- same file and function names.

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]
// While the bodies are todo!(), clippy suggests &mut [T] / &mut str for
// parameters that the real implementation needs as &mut Vec / &mut String.
#![allow(clippy::ptr_arg)]

pub mod bonus_json;
pub mod ex01_structs;
pub mod ex02_tuple_unit_structs;
pub mod ex03_methods;
pub mod ex04_enums;
pub mod ex05_option;
pub mod ex06_patterns;
pub mod ex07_result;
pub mod ex08_game_state;
pub mod ex09_shapes;
