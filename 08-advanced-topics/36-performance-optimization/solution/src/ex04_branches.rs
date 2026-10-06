//! Exercise 4: branches and bounds checks.
//!
//! A CPU guesses which way each branch goes and runs ahead; a wrong guess
//! throws away ~15-20 cycles of work. On random data an `if` in a hot loop
//! is a coin flip per element. Writing the condition as *arithmetic* (a
//! `bool` is 0 or 1) or as a conditional move removes the branch. And every
//! `v[i]` is a bounds check -- usually free, but it can block
//! vectorization; iterators avoid it by construction.

/// Count elements below `threshold` with an `if` per element.
pub fn count_below_branchy(values: &[i32], threshold: i32) -> usize {
    let mut count = 0;
    for &v in values {
        if v < threshold {
            count += 1;
        }
    }
    count
}

/// The same with no branch: add the comparison's 0 or 1.
pub fn count_below_branchless(values: &[i32], threshold: i32) -> usize {
    values.iter().map(|&v| (v < threshold) as usize).sum()
}

/// The first index whose value is `>= target` in a sorted slice (like
/// `partition_point(|v| v < target)`), with a fixed number of steps and no
/// unpredictable branch: the conditional becomes a select.
pub fn lower_bound_branchless(sorted: &[i32], target: i32) -> usize {
    if sorted.is_empty() {
        return 0;
    }
    let mut base = 0usize;
    let mut len = sorted.len();
    while len > 1 {
        let half = len / 2;
        // `if` on a value -> a conditional move, not a jump
        base = if sorted[base + half - 1] < target {
            base + half
        } else {
            base
        };
        len -= half;
    }
    base + (sorted[base] < target) as usize
}

/// Sum by index: a bounds check per access (in principle -- see the README).
// Indexing on purpose: this is the version being compared.
#[allow(clippy::needless_range_loop)]
pub fn sum_indexed(values: &[u64]) -> u64 {
    let mut sum = 0u64;
    for i in 0..values.len() {
        sum = sum.wrapping_add(values[i]);
    }
    sum
}

/// Sum with an iterator: no bounds checks, four independent accumulators.
pub fn sum_iter(values: &[u64]) -> u64 {
    let chunks = values.chunks_exact(4);
    let rest = chunks.remainder();
    let mut acc = [0u64; 4];
    for chunk in chunks {
        for k in 0..4 {
            acc[k] = acc[k].wrapping_add(chunk[k]);
        }
    }
    rest.iter()
        .chain(&acc)
        .fold(0u64, |s, v| s.wrapping_add(*v))
}

/// Deterministic pseudo-random values in `0..range`.
pub fn random_values(n: usize, range: i32, seed: u64) -> Vec<i32> {
    let mut s = seed;
    (0..n)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            (s % range as u64) as i32
        })
        .collect()
}
