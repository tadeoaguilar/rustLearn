# Getting Started with 07 · Collections & Iterators

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m07-collections-iterators-solution -- all
```

## What Is Already Here

```
07-collections-iterators/
├── README.md               # The concepts
├── exercises.md            # The work: 6 exercises plus a bonus
├── GETTING_STARTED.md      # This file
│
├── exercise/               # ← YOUR WORKSPACE (package m07-collections-iterators)
│   └── src/ex01_vectors.rs … bonus_pipeline.rs   (bodies are todo!())
│
├── solution/               # ← REFERENCE IMPLEMENTATION (m07-collections-iterators-solution)
│
└── tests/                  # ← 22 tests (m07-collections-iterators-tests)
```

A few skeleton functions return `impl Iterator` or `impl Fn`. A bare `todo!()`
doesn't compile there, so those have a placeholder line after the `todo!()`
(`std::iter::empty()`, `|| 0`). Replace both lines.

## The Commands You Need

```bash
# Run your own work
cargo run -p m07-collections-iterators -- 1

# Check your work
cargo test -p m07-collections-iterators-tests --features mine

# One exercise's tests (names start ex1_ … ex6_, bonus_)
cargo test -p m07-collections-iterators-tests --features mine ex5_

# The reference solution
cargo run -p m07-collections-iterators-solution -- 1       # Vec operations
cargo run -p m07-collections-iterators-solution -- 2       # HashMap, LRU cache
cargo run -p m07-collections-iterators-solution -- 3       # Iterator basics (watch laziness)
cargo run -p m07-collections-iterators-solution -- 4       # fold, scan, zip, flat_map
cargo run -p m07-collections-iterators-solution -- 5       # Custom iterators
cargo run -p m07-collections-iterators-solution -- 6       # Closures
cargo run -p m07-collections-iterators-solution -- bonus   # Sales pipeline
cargo run -p m07-collections-iterators-solution -- all     # Everything

cargo test -p m07-collections-iterators-tests              # always green
```

## Reproduce the Fibonacci Overflow

Paste the exercise's `Fibonacci` into `exercise/src/ex05_custom_iterators.rs`
unchanged, then:

```bash
cargo test -p m07-collections-iterators-tests --features mine ex5_fibonacci
```

The test calls `.count()`, which drives the iterator until it panics with
`attempt to add with overflow`. Fix it with `checked_add`.

## If You Get Stuck

1. **`a value of type Vec<_> cannot be built from an iterator over &i32`** — add `.copied()` or `.cloned()` before `collect`.
2. **`cannot borrow as mutable` when calling a closure** — the closure is `FnMut`; declare it `let mut f = ...`.
3. **`closure may outlive the current function`** — add `move` before the closure.
4. **A test fails only sometimes** — you're depending on HashMap order; sort or use `BTreeMap`.
5. Compare against `solution/src/` — same file and function names.
