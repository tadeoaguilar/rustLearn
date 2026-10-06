//! Module 39 -- Compiler Internals. Reference solution.
//!
//! | File                | Exercise |
//! |---------------------|----------|
//! | `ex01_lint.rs`      | 1  A lint tool on the syntax tree (how clippy lints work) |
//! | `ex02_mir.rs`       | 2  Reading `rustc --emit=mir`: blocks, bounds and overflow checks |
//! | `ex03_borrowck.rs`  | 3  Borrow checker errors: JSON diagnostics, fixes |
//! | `ex04_desugar.rs`   | 4  Desugaring `for` and `?` |
//! | `bonus_unused.rs`   | bonus: an unused-variable check |
//! | `toolchain.rs`      | provided: running rustc on a snippet |

pub mod bonus_unused;
pub mod ex01_lint;
pub mod ex02_mir;
pub mod ex03_borrowck;
pub mod ex04_desugar;
pub mod toolchain;
