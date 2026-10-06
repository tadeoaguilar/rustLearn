//! Module 33 -- 33-embedded-rust. YOUR WORKSPACE.
//!
//! Each `exNN_*.rs` file has the signatures the tests expect, with `todo!()`
//! bodies. Replace each `todo!()` with your implementation, then:
//!
//!     cargo run  -p m33-embedded-rust -- <exercise number>
//!     cargo test -p m33-embedded-rust-tests --features mine
//!
//! Stuck? Compare with ../solution/src -- same file and function names.

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]
#![no_std]

pub mod bonus_pid;
pub mod ex01_blink;
pub mod ex02_sensor;
pub mod ex03_framing;
pub mod ex04_registers;
pub mod ex05_input;
