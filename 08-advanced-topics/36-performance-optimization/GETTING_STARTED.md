# Getting Started with 36 · Performance Optimization

## Quick Start

All commands run from the **repository root**. Always `--release`:

```bash
cargo run --release -p m36-performance-optimization-solution -- all
```

times every version of every exercise on this machine -- naive vs fast
word counting, scalar vs unrolled vs intrinsic dot products, four matrix
multiplication orders, branchy vs branchless loops -- and shows the SWAR
tricks.

## What Is Already Here

```
36-performance-optimization/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/src/              # ← YOUR WORKSPACE (package m36-performance-optimization)
│   ├── ex01_hotpath.rs        #   Ex 1  (top_words_naive, sample_text provided)
│   ├── ex02_simd.rs           #   Ex 2
│   ├── ex03_cache.rs          #   Ex 3  (Matrix::zeros/sample/get provided)
│   ├── ex04_branches.rs       #   Ex 4  (random_values provided)
│   ├── bonus_swar.rs          #   bonus
│   └── main.rs                #   provided: the timing harness
├── solution/                  # ← REFERENCE + ANSWERS.md (measured numbers)
│   └── benches/perf.rs        #   criterion benchmarks (optional)
└── tests/                     # ← 13 tests (m36-performance-optimization-tests)
```

## The Commands You Need

```bash
cargo test -p m36-performance-optimization-tests --features mine ex2_
cargo test -p m36-performance-optimization-tests --features mine
cargo run --release -p m36-performance-optimization -- 2
cargo test -p m36-performance-optimization-tests                    # the solution: always green
```

## Profiling and Reading Assembly (Optional)

```bash
cargo install flamegraph            # needs dtrace (macOS, may ask for sudo) or perf (Linux)
cargo flamegraph -p m36-performance-optimization-solution -- 1
cargo install cargo-show-asm
cargo asm -p m36-performance-optimization-solution --lib count_below_branchy
cargo bench -p m36-performance-optimization-solution               # criterion reports in target/criterion/
```

None of these are needed for the tests.

## If You Get Stuck

1. **`top_words_fast` disagrees on ties** -- sort by count descending, then word ascending, in both the selection and the final sort.
2. **`select_nth_unstable_by` panics** -- the index must be `< len`; handle `n == 0` and `n >= len` first.
3. **SIMD results differ slightly** -- expected for floats; the tests allow a relative error of 1e-3.
4. **"use of unsafe function requires unsafe block"** -- `#[target_feature]` functions are `unsafe` to call; justify it with the feature check.
5. **Your fast version is slower** -- are you in `--release`?
6. **SWAR counts too many** -- the plain zero-byte mask has false positives above a real zero; counting needs the exact per-byte test.
7. Compare with `solution/src/` -- same file and function names.
