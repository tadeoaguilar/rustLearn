//! Module 36 -- Performance Optimization. Reference solution.
//!
//! Every function here comes in versions that compute the same result at
//! different speeds; the tests check they agree, the demo and the benches
//! (`cargo bench`) measure them.
//!
//! | File                 | Exercise |
//! |----------------------|----------|
//! | `ex01_hotpath.rs`    | 1  profile-guided: allocation, hashing, partial sorting |
//! | `ex02_simd.rs`       | 2  SIMD: auto-vectorization, SSE2/AVX2 intrinsics, runtime detection |
//! | `ex03_cache.rs`      | 3  cache-friendly matrix multiplication: loop order, transposition, tiling |
//! | `ex04_branches.rs`   | 4  branch misprediction and bounds checks |
//! | `bonus_swar.rs`      | bonus: SIMD within a register |
//!
//! Binary size (the outline's "reduce binary size by 50%") is measured with
//! the root workspace's `min-size` profile -- see the README.

pub mod bonus_swar;
pub mod ex01_hotpath;
pub mod ex02_simd;
pub mod ex03_cache;
pub mod ex04_branches;
