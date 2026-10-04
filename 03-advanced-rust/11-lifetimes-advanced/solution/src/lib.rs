//! Module 11 -- Advanced Lifetimes. Reference solution.
//!
//! A lifetime annotation never makes anything live longer. It's a *claim*
//! about how the references in a signature relate ("the result borrows from
//! `x`, not `y`"), which the compiler checks inside the function and then
//! relies on at every call site.

pub mod bonus_str_split;
pub mod ex01_annotations;
pub mod ex02_structs;
pub mod ex03_parser;
pub mod ex04_iterators;
pub mod ex05_static;
pub mod ex06_hrtb;
pub mod ex07_fix_errors;
