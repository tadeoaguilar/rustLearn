//! Exercise 2: SIMD -- one instruction, many lanes.
//!
//! Three ways to make a loop process several elements per instruction:
//!
//! 1. **Let the compiler do it.** A plain loop with one accumulator can't be
//!    vectorized for floats (reordering additions changes the result, so the
//!    compiler won't). Eight independent accumulators make the reordering
//!    explicit, and LLVM vectorizes that.
//! 2. **Intrinsics** (`std::arch`): `_mm_*` for SSE2 (every x86-64 CPU has
//!    it), `_mm256_*` for AVX2 (most CPUs since 2013) -- chosen at *runtime*
//!    with `is_x86_feature_detected!`, so one binary runs everywhere.
//! 3. **Portable fallback** for other architectures (on aarch64 the
//!    compiler turns the unrolled loop into NEON code).
//!
//! Floating-point results differ in the last bits between versions (the
//! additions happen in a different order); integer results are exact.

/// The obvious loop.
pub fn dot_scalar(a: &[f32], b: &[f32]) -> f32 {
    todo!("Exercise 2")
}

/// Eight accumulators over exact chunks, then the remainder: vectorizable.
pub fn dot_unrolled(a: &[f32], b: &[f32]) -> f32 {
    todo!("Exercise 2")
}

/// Which explicit SIMD path this CPU gets.
pub fn simd_level() -> &'static str {
    todo!("Exercise 2")
}

/// The fastest dot product available on this CPU.
pub fn dot_simd(a: &[f32], b: &[f32]) -> f32 {
    todo!("Exercise 2")
}

/// How many times `needle` occurs in `haystack`, the obvious way.
pub fn count_byte_scalar(haystack: &[u8], needle: u8) -> usize {
    todo!("Exercise 2")
}

/// The same, 16 bytes per comparison (SSE2) where available.
pub fn count_byte_simd(haystack: &[u8], needle: u8) -> usize {
    todo!("Exercise 2")
}

#[cfg(target_arch = "x86_64")]
mod x86 {
    use std::arch::x86_64::*;

    /// # Safety
    /// The CPU must support SSE2 (all x86-64 CPUs do).
    #[target_feature(enable = "sse2")]
    pub unsafe fn dot_sse2(a: &[f32], b: &[f32]) -> f32 {
        todo!("Exercise 2")
    }

    /// # Safety
    /// The CPU must support AVX2 and FMA (check with `is_x86_feature_detected!`).
    #[target_feature(enable = "avx2,fma")]
    pub unsafe fn dot_avx2(a: &[f32], b: &[f32]) -> f32 {
        todo!("Exercise 2")
    }

    /// # Safety
    /// The CPU must support SSE2.
    #[target_feature(enable = "sse2")]
    pub unsafe fn count_byte_sse2(haystack: &[u8], needle: u8) -> usize {
        todo!("Exercise 2")
    }
}
