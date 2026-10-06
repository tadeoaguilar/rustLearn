# Exercises: Performance Optimization

Performance work is a loop: **measure**, find where the time goes, change
one thing, **measure again**. Every exercise here pairs a straightforward
version with faster ones that must compute exactly the same result -- the
tests check the results, the demo and the benches check the speed.

**Setup**: no dependencies (criterion for the optional benches). Always
measure release builds:
`cargo run --release -p m36-performance-optimization -- <n>`.
Optional profilers: `cargo install flamegraph` (`cargo flamegraph`),
`samply`, or Instruments on macOS; `cargo install cargo-show-asm` to read
the generated assembly.

---

## Exercise 1: Fix the Hot Path

**Difficulty**: Medium
**Time**: 1.5 hours

**Learning Objectives**:
- Read a profile and find what costs time
- Avoid allocations in inner loops
- Choose a hasher and a partial sort

`top_words_naive` (provided) counts words and returns the top `n`. Profile
it on `sample_text(1_000_000)` and write `top_words_fast` with **identical**
output:

1. `Fnv1a`: implement `Hasher` (offset basis `0xcbf29ce484222325`, prime
   `0x100000001b3`; per byte: xor, then multiply).
2. Scan bytes, lowercasing into one reused buffer; look up by `&str` and
   allocate a `String` only for new words.
3. Find the top `n` with `select_nth_unstable_by`, then sort only those.

**Question**: why is the standard `HashMap` hasher slower than FNV, and
when must you keep it anyway?

---

## Exercise 2: SIMD

**Difficulty**: Hard
**Time**: 2 hours

**Learning Objectives**:
- Write loops the compiler can vectorize
- Use `std::arch` intrinsics with runtime feature detection
- Keep portable fallbacks

1. `dot_unrolled`: eight independent accumulators over `chunks_exact(8)`.
2. `dot_simd`: on x86-64, AVX2+FMA if `is_x86_feature_detected!` says so
   (in a `#[target_feature(enable = "avx2,fma")] unsafe fn`), else SSE2;
   elsewhere, `dot_unrolled`. `simd_level` reports which.
3. `count_byte_simd`: SSE2 compares 16 bytes at once (`_mm_cmpeq_epi8`,
   `_mm_movemask_epi8`, `count_ones`); scalar fallback.

**Question**: why doesn't the compiler vectorize `dot_scalar` by itself?

---

## Exercise 3: Cache-Friendly Matrix Multiplication

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- See the cost of strided memory access
- Reorder loops, transpose, tile

All four must return the same matrix:

1. `multiply_naive` (i, j, k), `multiply_ikj` (i, k, j),
   `Matrix::transposed` and `multiply_transposed`.
2. `multiply_blocked(a, b, tile)`: i, k, j over tiles, any `n` (partial
   edge tiles).

---

## Exercise 4: Branches and Bounds Checks -- Measure First

**Difficulty**: Easy
**Time**: 45 minutes

**Learning Objectives**:
- Write branch-free code
- Check what the compiler already does before "optimizing"

1. `count_below_branchy` / `count_below_branchless` (add `(v < t) as usize`).
2. `lower_bound_branchless` (equal to `partition_point(|v| v < target)`).
3. `sum_indexed` / `sum_iter` (`chunks_exact(4)`, four accumulators).

Time them on random and sorted data -- and then look at the assembly
(`cargo asm`). You may find the compiler already did your job.

---

## Exercise 5: Reduce the Binary Size

**Difficulty**: Easy
**Time**: 20 minutes

The root `Cargo.toml` defines a `min-size` profile. Compare:

```bash
cargo build --release          -p m36-performance-optimization --bin m36-performance-optimization
cargo build --profile min-size -p m36-performance-optimization --bin m36-performance-optimization
ls -l target/release/m36-performance-optimization target/min-size/m36-performance-optimization
```

Try each setting on its own (`CARGO_PROFILE_RELEASE_STRIP=true cargo build
--release ...`) and record what each one saves. What would you have to
give up to get below half the original size?

---

## Bonus: SWAR

**Difficulty**: Medium
**Time**: 45 minutes

`zero_byte_mask`, `splat`, `find_byte` and an exact `count_byte`, eight
bytes per `u64`, with no SIMD instructions at all.
