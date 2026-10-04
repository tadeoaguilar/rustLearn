# Getting Started with 08 · Modules & Crates

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m08-modules-crates-solution --all-features -- all
```

## What Is Already Here

```
08-modules-crates/
├── README.md                   # The concepts
├── exercises.md                # The work: 7 exercises plus a bonus
├── GETTING_STARTED.md          # This file
│
├── exercise/                   # ← YOUR WORKSPACE (package m08-modules-crates)
│   ├── Cargo.toml              #   features already declared; doctest = false for now
│   ├── crates/money/           #   Exercise 5: package m08-money (skeleton)
│   └── src/
│       ├── legacy_bookstore.rs #   ← the working monolith you refactor FROM
│       ├── lib.rs              #   module tree + re-exports
│       ├── catalog.rs, catalog/book.rs, catalog/isbn.rs
│       ├── cart.rs, pricing.rs, prelude.rs, report.rs, rounding.rs
│       └── main.rs             #   ready-made runner
│
├── solution/                   # ← REFERENCE IMPLEMENTATION (m08-modules-crates-solution)
│   ├── crates/money/           #   m08-money-solution
│   ├── ANSWERS.md
│   └── src/
│
└── tests/                      # ← 13 tests (m08-modules-crates-tests)
```

## The Commands You Need

```bash
# Run your own work
cargo run -p m08-modules-crates -- all

# Check your work
cargo test -p m08-modules-crates-tests --features mine

# One exercise's tests (names start ex1_ … ex5_, bonus_)
cargo test -p m08-modules-crates-tests --features mine ex2_

# The reference solution
cargo run -p m08-modules-crates-solution -- 1                        # catalog and cart
cargo run -p m08-modules-crates-solution -- 2                        # validation
cargo run -p m08-modules-crates-solution -- 4                        # features in this build
cargo run -p m08-modules-crates-solution --all-features -- 4         # ...with JSON
cargo run -p m08-modules-crates-solution --no-default-features -- 4  # ...without discounts
cargo run -p m08-modules-crates-solution -- 5                        # the money crate
cargo run -p m08-modules-crates-solution --features report -- bonus  # inventory report

cargo test -p m08-modules-crates-tests                               # always green
```

## Exercise 4: Test Every Feature Combination

```bash
cargo build -p m08-modules-crates --no-default-features
cargo build -p m08-modules-crates --no-default-features --features json
cargo build -p m08-modules-crates --features report
cargo build -p m08-modules-crates --all-features
```

Watch for `unused` warnings with `--no-default-features` — code only the
disabled feature needs must be gated too (that's why `mod rounding;` has a
`#[cfg]`).

`cargo install cargo-hack` then `cargo hack check -p m08-modules-crates
--feature-powerset` tries every combination for you.

**Careful with `--all-features` on the tests crate**: it also turns on `mine`,
so `cargo test -p m08-modules-crates-tests --all-features` tests *your* code.

## Exercise 6: Docs and Doc Tests

```bash
cargo doc -p m08-modules-crates-solution --all-features --open
cargo test -p m08-modules-crates-solution --all-features --doc
```

For your own crate, delete the `[lib] doctest = false` lines from
`exercise/Cargo.toml` (and `exercise/crates/money/Cargo.toml`) when everything
is implemented, then:

```bash
cargo test -p m08-modules-crates --all-features --doc
```

## If You Get Stuck

1. **`file not found for module`** — the file must be at the path `mod` implies: `catalog/book.rs` for `mod book;` inside `catalog.rs`.
2. **`unresolved import`** — inside the crate use `crate::...`; from a child, the parent is `super::...`.
3. **`field is private` / `constructor is private`** — that's Exercise 2 working; add a constructor or getter.
4. **`function is never used` only with `--no-default-features`** — gate it with the same `#[cfg(feature = ...)]` as its caller.
5. **`the trait Serialize is not implemented for Money`** — the `json` feature must forward to the money crate: `"m08-money/serde"`.
6. Compare against `solution/` — same file and function names.
