# Getting Started with 09 · Testing

## Quick Start

All commands run from the **repository root**.

```bash
cargo run  -p m09-testing          # the buggy library -- looks fine, doesn't it?
cargo test -p m09-testing          # 0 tests: writing them is your job
```

## What Is Already Here

```
09-testing/
├── README.md               # The concepts
├── exercises.md            # The work: 7 exercises plus a bonus (bug list at the end, folded)
├── GETTING_STARTED.md      # This file
│
├── exercise/               # ← YOUR WORKSPACE (package m09-testing)
│   ├── Cargo.toml          #   proptest, mockall, criterion already added
│   ├── src/                #   WORKING library with 6 bugs; empty `mod tests` blocks
│   ├── benches/words.rs    #   Exercise 7 skeleton
│   └── tests/              #   (create it) Exercises 3 and 6
│
├── solution/               # ← REFERENCE (m09-testing-solution)
│   ├── src/                #   fixed code, marked BUG FIXED, with full unit tests
│   ├── tests/bank.rs, tests/common/mod.rs, tests/properties.rs
│   ├── benches/words.rs
│   └── ANSWERS.md
│
└── tests/                  # ← ANSWER KEY: 14 acceptance tests (m09-testing-tests)
```

## The Commands You Need

```bash
# Your tests
cargo test -p m09-testing                     # everything
cargo test -p m09-testing --lib               # unit tests only
cargo test -p m09-testing --test bank         # one integration test file
cargo test -p m09-testing --doc               # doc tests (after removing doctest = false)
cargo test -p m09-testing median              # tests whose name contains "median"
cargo test -p m09-testing -- --nocapture      # show println! output from tests

# The answer key: which bugs are still there?
cargo test -p m09-testing-tests --features mine

# Benchmarks
cargo bench -p m09-testing
cargo bench -p m09-testing-solution
open target/criterion/report/index.html       # HTML report (macOS; xdg-open on Linux)

# The reference suite
cargo test -p m09-testing-solution
```

## Suggested Workflow

1. Read a module of `exercise/src`, then its exercise in `exercises.md`.
2. Write tests for what the doc comments *promise* — not for what the code does.
3. A test fails? Decide whether the test or the code is wrong. Fix the code, keep the test.
4. After each exercise, run the answer key and see the bug count drop.

## When a Property Test Fails

proptest prints the smallest failing input it could find, e.g.

```text
Test failed: 90 -> LXXXX; minimal failing input: n = 90
```

and saves it in `proptest-regressions/` so the next run tries it first. Commit
those files in a real project; here they're just noise you can delete.

## Coverage (Bonus)

```bash
cargo install cargo-llvm-cov
cargo llvm-cov -p m09-testing-solution --html --open
```

## If You Get Stuck

1. **`cannot find type MockWeatherApi`** — it only exists in `#[cfg(test)]` code, and only with `#[cfg_attr(test, mockall::automock)]` on the trait.
2. **`use of undeclared crate m09_testing` in tests/** — integration tests use the package's library name with underscores: `m09_testing`.
3. **Integration test can't see a function** — it's private; test it from a unit test instead.
4. **`expected ... found NaN`** — `NaN != NaN`; check `is_nan()` or, better, why you got one.
5. Compare with `solution/` — every fix is marked `BUG FIXED`.
