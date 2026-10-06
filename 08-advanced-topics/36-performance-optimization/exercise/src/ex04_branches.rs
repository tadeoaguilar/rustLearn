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
    todo!("Exercise 4")
}

/// The same with no branch: add the comparison's 0 or 1.
pub fn count_below_branchless(values: &[i32], threshold: i32) -> usize {
    todo!("Exercise 4")
}

/// The first index whose value is `>= target` in a sorted slice (like
/// `partition_point(|v| v < target)`), with a fixed number of steps and no
/// unpredictable branch: the conditional becomes a select.
pub fn lower_bound_branchless(sorted: &[i32], target: i32) -> usize {
    todo!("Exercise 4")
}

/// Sum by index: a bounds check per access.
pub fn sum_indexed(values: &[u64]) -> u64 {
    todo!("Exercise 4")
}

/// Sum with an iterator: no bounds checks, four independent accumulators.
pub fn sum_iter(values: &[u64]) -> u64 {
    todo!("Exercise 4")
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
