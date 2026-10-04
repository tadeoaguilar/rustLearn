//! Module 07 -- Collections & Iterators. Reference solution.
//!
//! Iterators are lazy: `v.iter().map(f).filter(g)` does nothing until a
//! consumer (`collect`, `sum`, `for`, `find`...) pulls items through. And the
//! chain compiles to the same machine code as a hand-written loop.

pub mod bonus_pipeline;
pub mod ex01_vectors;
pub mod ex02_hashmaps;
pub mod ex03_iterator_basics;
pub mod ex04_advanced_iterators;
pub mod ex05_custom_iterators;
pub mod ex06_closures;
