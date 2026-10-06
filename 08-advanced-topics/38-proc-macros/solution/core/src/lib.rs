//! Module 38 -- the macro expansions. Each function turns input tokens into
//! output tokens, or a `syn::Error` that becomes a compile error pointing at
//! the offending code. `krate` is the path generated code uses to reach the
//! runtime crate (`::m38_proc_macros_solution` for the solution).
//!
//! | File                   | Exercise |
//! |------------------------|----------|
//! | `ex01_to_json.rs`      | 1  `#[derive(ToJson)]`: a serialization derive with attributes |
//! | `ex02_table.rs`        | 2  `#[derive(Table)]`: an ORM-style derive generating SQL |
//! | `ex03_state_machine.rs`| 3  `state_machine! { .. }`: a DSL with its own grammar |
//! | `ex04_memoize.rs`      | 4  `#[memoize]`: an attribute macro rewriting a function |
//! | `bonus_enum_iter.rs`   | bonus: `#[derive(EnumIter)]` |
//! | `attrs.rs`             | provided: helper-attribute parsing shared by 1 and 2 |

pub mod attrs;
pub mod bonus_enum_iter;
pub mod ex01_to_json;
pub mod ex02_table;
pub mod ex03_state_machine;
pub mod ex04_memoize;
