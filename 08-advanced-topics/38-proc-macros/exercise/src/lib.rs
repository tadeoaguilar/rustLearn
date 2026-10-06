//! Module 38 -- Procedural Macros. YOUR WORKSPACE.
//!
//! The exercises are in `core/src/` (package `m38-proc-macros-core`): each
//! expansion function there is `todo!()`. This crate (the runtime traits,
//! re-exporting the macros) and `derive/` (the thin `proc-macro` wrappers)
//! are provided.
//!
//!     cargo run  -p m38-proc-macros -- 1                          print what your expansion generates
//!     cargo test -p m38-proc-macros-tests --features mine,ex1     test Exercise 1 (see tests/Cargo.toml)
//!
//! Stuck? Compare with ../solution/core/src -- same file and function names.

// Generated code names this crate by its absolute path; this alias makes
// that path work inside the crate itself too.
extern crate self as m38_proc_macros;

pub mod json;
pub mod orm;

pub use m38_proc_macros_derive::{EnumIter, Table, ToJson, memoize, state_machine};
