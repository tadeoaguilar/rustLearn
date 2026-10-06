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
    assert_eq!(a.len(), b.len());
    let mut sum = 0.0;
    for i in 0..a.len() {
        sum += a[i] * b[i];
    }
    sum
}

/// Eight accumulators over exact chunks, then the remainder: vectorizable.
pub fn dot_unrolled(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    let mut acc = [0.0f32; 8];
    let (chunks_a, chunks_b) = (a.chunks_exact(8), b.chunks_exact(8));
    let (rest_a, rest_b) = (chunks_a.remainder(), chunks_b.remainder());
    for (ca, cb) in chunks_a.zip(chunks_b) {
        for k in 0..8 {
            acc[k] += ca[k] * cb[k];
        }
    }
    let tail: f32 = rest_a.iter().zip(rest_b).map(|(x, y)| x * y).sum();
    acc.iter().sum::<f32>() + tail
}

/// Which explicit SIMD path this CPU gets.
pub fn simd_level() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            return "avx2+fma";
        }
        "sse2"
    }
    #[cfg(not(target_arch = "x86_64"))]
    "portable"
}

/// The fastest dot product available on this CPU.
pub fn dot_simd(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: we just checked the CPU supports AVX2 and FMA.
            return unsafe { x86::dot_avx2(a, b) };
        }
        // SAFETY: SSE2 is part of the x86-64 baseline.
        unsafe { x86::dot_sse2(a, b) }
    }
    #[cfg(not(target_arch = "x86_64"))]
    dot_unrolled(a, b)
}

/// How many times `needle` occurs in `haystack`, the obvious way.
pub fn count_byte_scalar(haystack: &[u8], needle: u8) -> usize {
    haystack.iter().filter(|&&b| b == needle).count()
}

/// The same, 16 bytes per comparison (SSE2) where available.
pub fn count_byte_simd(haystack: &[u8], needle: u8) -> usize {
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: SSE2 is part of the x86-64 baseline.
        unsafe { x86::count_byte_sse2(haystack, needle) }
    }
    #[cfg(not(target_arch = "x86_64"))]
    count_byte_scalar(haystack, needle)
}

#[cfg(target_arch = "x86_64")]
mod x86 {
    use std::arch::x86_64::*;

    /// # Safety
    /// The CPU must support SSE2 (all x86-64 CPUs do).
    #[target_feature(enable = "sse2")]
    pub unsafe fn dot_sse2(a: &[f32], b: &[f32]) -> f32 {
        let n = a.len() / 4 * 4;
        let mut acc = _mm_setzero_ps();
        let mut i = 0;
        while i < n {
            // SAFETY: i + 4 <= n <= len; unaligned loads are allowed.
            let (x, y) = unsafe {
                (
                    _mm_loadu_ps(a.as_ptr().add(i)),
                    _mm_loadu_ps(b.as_ptr().add(i)),
                )
            };
            acc = _mm_add_ps(acc, _mm_mul_ps(x, y));
            i += 4;
        }
        let mut lanes = [0.0f32; 4];
        // SAFETY: `lanes` has room for 4 floats.
        unsafe { _mm_storeu_ps(lanes.as_mut_ptr(), acc) };
        lanes.iter().sum::<f32>() + a[n..].iter().zip(&b[n..]).map(|(x, y)| x * y).sum::<f32>()
    }

    /// # Safety
    /// The CPU must support AVX2 and FMA (check with `is_x86_feature_detected!`).
    #[target_feature(enable = "avx2,fma")]
    pub unsafe fn dot_avx2(a: &[f32], b: &[f32]) -> f32 {
        let n = a.len() / 16 * 16;
        // two accumulators hide the FMA latency
        let (mut acc0, mut acc1) = (_mm256_setzero_ps(), _mm256_setzero_ps());
        let mut i = 0;
        while i < n {
            // SAFETY: i + 16 <= n <= len.
            unsafe {
                acc0 = _mm256_fmadd_ps(
                    _mm256_loadu_ps(a.as_ptr().add(i)),
                    _mm256_loadu_ps(b.as_ptr().add(i)),
                    acc0,
                );
                acc1 = _mm256_fmadd_ps(
                    _mm256_loadu_ps(a.as_ptr().add(i + 8)),
                    _mm256_loadu_ps(b.as_ptr().add(i + 8)),
                    acc1,
                );
            }
            i += 16;
        }
        let mut lanes = [0.0f32; 8];
        // SAFETY: `lanes` has room for 8 floats.
        unsafe { _mm256_storeu_ps(lanes.as_mut_ptr(), _mm256_add_ps(acc0, acc1)) };
        lanes.iter().sum::<f32>() + a[n..].iter().zip(&b[n..]).map(|(x, y)| x * y).sum::<f32>()
    }

    /// # Safety
    /// The CPU must support SSE2.
    #[target_feature(enable = "sse2")]
    pub unsafe fn count_byte_sse2(haystack: &[u8], needle: u8) -> usize {
        let target = _mm_set1_epi8(needle as i8);
        let n = haystack.len() / 16 * 16;
        let mut count = 0usize;
        let mut i = 0;
        while i < n {
            // SAFETY: i + 16 <= n <= len; unaligned load.
            let chunk = unsafe { _mm_loadu_si128(haystack.as_ptr().add(i) as *const __m128i) };
            // 0xFF in each matching byte; movemask packs the top bits into an int
            let mask = _mm_movemask_epi8(_mm_cmpeq_epi8(chunk, target));
            count += mask.count_ones() as usize;
            i += 16;
        }
        count + haystack[n..].iter().filter(|&&b| b == needle).count()
    }
}
