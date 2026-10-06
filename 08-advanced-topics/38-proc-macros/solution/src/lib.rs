//! Module 38 -- Procedural Macros. Reference solution.
//!
//! Three crates work together:
//!
//! | Crate | Holds |
//! |---|---|
//! | `core/` (`m38-proc-macros-core-solution`) | the expansions: tokens in, tokens out -- **the exercises** |
//! | `derive/` (`m38-proc-macros-derive-solution`) | the `proc-macro` crate: thin wrappers over core |
//! | this one | the runtime traits the generated code implements, and the macros re-exported |
//!
//! | Exercise | Macro | Core file | Runtime |
//! |---|---|---|---|
//! | 1 | `#[derive(ToJson)]` | `ex01_to_json.rs` | `json` |
//! | 2 | `#[derive(Table)]` | `ex02_table.rs` | `orm` |
//! | 3 | `state_machine!` | `ex03_state_machine.rs` | -- |
//! | 4 | `#[memoize]` | `ex04_memoize.rs` | -- |
//! | bonus | `#[derive(EnumIter)]` | `bonus_enum_iter.rs` | -- |

// Generated code names this crate by its absolute path; this alias makes
// that path work inside the crate itself too.
extern crate self as m38_proc_macros_solution;

pub mod json;
pub mod orm;

pub use m38_proc_macros_derive_solution::{EnumIter, Table, ToJson, memoize, state_machine};
