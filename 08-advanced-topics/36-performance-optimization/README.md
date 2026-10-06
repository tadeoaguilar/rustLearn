# 36 · Performance Optimization

## Overview

Rust is fast by default, but "fast by default" still leaves factors of 2
to 10 on the table in hot loops -- in allocation, hashing, memory access
patterns and vectorization. Getting them back is a method, not a bag of
tricks: measure, find the hot spot, change one thing, verify the result is
unchanged and the time is lower. This module applies that method to five
problems, including two where the honest answer is "the compiler already
did it".

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Hot path | Allocation, hashing, partial sorting -- 3.9x |
| 2 | SIMD | Auto-vectorization, intrinsics, runtime detection -- up to 7x |
| 3 | Cache | Loop order, transposition, tiling -- 3x |
| 4 | Branches | Branch-free code, bounds checks -- and checking the assembly |
| 5 | Binary size | Release profile settings, measured -- -39% |
| Bonus | SWAR | Eight bytes per `u64` |

## Key Concepts

### The method

1. A **benchmark** you trust (criterion, or best-of-N with `black_box`)
2. A **profile** (`cargo flamegraph`, samply, Instruments, perf)
3. One change at a time, with **tests** proving the output is identical
4. **Release mode**, always

### What usually costs

| Cost | Fix |
|---|---|
| allocation in a loop | reuse buffers, `with_capacity`, borrow instead of own |
| hashing | a faster hasher for trusted keys |
| sorting everything | `select_nth_unstable`, `BinaryHeap` for top-k |
| strided memory access | walk memory in order; transpose; tile |
| serial dependency chains | several independent accumulators |
| unpredictable branches | arithmetic or `select` instead of `if` |

### SIMD in Rust

```rust
if is_x86_feature_detected!("avx2") {
    unsafe { dot_avx2(a, b) }               // #[target_feature(enable = "avx2,fma")]
} else {
    dot_portable(a, b)
}
```

Detect at runtime, compile per-feature functions, keep a fallback. `std::simd`
(portable SIMD) is still nightly-only.

### Release profile settings

`opt-level`, `lto`, `codegen-units`, `panic`, `strip` -- see the
`min-size` profile in the root `Cargo.toml`, and the measured table in
[ANSWERS.md](solution/ANSWERS.md). For speed rather than size: `lto =
"fat"`, `codegen-units = 1`, and `-C target-cpu=native` when the binary
only runs on the machine that built it.

## Common Pitfalls

1. **Benchmarking debug builds** -- 10-50x slower and not representative
2. **Optimizing without a profile** -- the hot spot is rarely where you guess
3. **Letting the optimizer delete the benchmark** -- wrap inputs/outputs in `black_box`
4. **Changing results while optimizing** -- keep the slow version as the test oracle
5. **Comparing floats exactly** after reordering -- results differ in the last bits
6. **Assuming a trick helps** -- measure; the compiler may already do it
7. **`target-cpu=native` in distributed binaries** -- crashes on older CPUs

## Running This Module

```bash
cargo run  --release -p m36-performance-optimization -- 1                # your code (1-4, bonus, all)
cargo test -p m36-performance-optimization-tests --features mine         # test your code
cargo run  --release -p m36-performance-optimization-solution -- all     # timings of every version
cargo bench -p m36-performance-optimization-solution                     # criterion (optional, a few minutes)
cargo test -p m36-performance-optimization-tests                         # 13 tests against the solution
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the exercises in
[the phase README](../README.md) (profile and optimize a slow algorithm,
SIMD, cache usage, binary size).

- **Timings aren't asserted by tests** (they'd be flaky); results are.
- **Explicit SIMD is x86-64 only.** On other architectures (Apple Silicon)
  `dot_simd` and `count_byte_simd` use the portable versions, which LLVM
  auto-vectorizes to NEON; hand-written NEON wasn't added because it
  couldn't be compiled and tested on the machine this was built on.
- **Exercise 5 has no automated test**: binary size depends on the
  platform and toolchain. The measured numbers are in ANSWERS.md.
- **Profile-guided optimization (PGO)** and inline assembly are described
  in the README's resources, not exercised.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[37 · WebAssembly](../37-wasm/)
