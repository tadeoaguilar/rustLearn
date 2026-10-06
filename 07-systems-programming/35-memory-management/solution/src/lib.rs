//! Module 35 -- Memory Management. Reference solution.
//!
//! | File                     | Exercise |
//! |--------------------------|----------|
//! | `ex01_bump.rs`           | 1  a bump allocator, and one usable as the global allocator |
//! | `ex02_pool.rs`           | 2  a generational slab, and an object pool |
//! | `ex03_counting.rs`       | 3  a counting allocator: profiling allocations |
//! | `ex04_layout.rs`         | 4  sizes, alignment, padding, niches |
//! | `ex05_soa.rs`            | 5  cache-friendly layouts: AoS vs SoA, row vs column order |
//! | `bonus_arena_tree.rs`    | bonus: a tree in an arena |

pub mod bonus_arena_tree;
pub mod ex01_bump;
pub mod ex02_pool;
pub mod ex03_counting;
pub mod ex04_layout;
pub mod ex05_soa;
