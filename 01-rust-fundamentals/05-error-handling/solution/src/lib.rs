//! Module 05 -- Error Handling. Reference solution.
//!
//! Two kinds of failure:
//!
//! * **Bugs** -- "this can't happen" happened. `panic!`: stop, loudly.
//! * **Expected failures** -- the file is missing, the input isn't a number.
//!   Return `Result` and let the caller decide.
//!
//! Every function that touches the file system takes a path parameter instead
//! of hard-coding "username.txt" or "config.txt", so the tests can point it at
//! a temporary directory.

pub mod bonus_combinators;
pub mod ex01_panic;
pub mod ex02_result;
pub mod ex03_question_mark;
pub mod ex04_custom_errors;
pub mod ex05_converting_errors;
pub mod ex06_option_to_result;
pub mod ex07_recovery;
pub mod ex08_config_parser;
pub mod ex09_thiserror;
