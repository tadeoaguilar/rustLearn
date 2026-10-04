# Getting Started with 01 · Getting Started

## Quick Start

All commands run from the **repository root** (the folder with the top-level
`Cargo.toml`). Every module is a member of one Cargo workspace, so there is
nothing to `cd` into.

```bash
rustc --version            # 1.87 or newer
cargo build -p m01-getting-started-solution
```

## What Is Already Here

```
01-getting-started/
├── README.md               # The concepts
├── exercises.md            # The work: 7 exercises plus a bonus
├── GETTING_STARTED.md      # This file
│
├── exercise/               # ← YOUR WORKSPACE. Write your code here.
│   ├── Cargo.toml          #   package m01-getting-started
│   └── src/
│       ├── main.rs         #   ready-made runner: `-- 1`, `-- 4`, `-- all` ...
│       ├── lib.rs
│       └── ex01_hello.rs … #   one file per exercise, each body is todo!()
│
├── solution/               # ← REFERENCE IMPLEMENTATION.
│   ├── Cargo.toml          #   package m01-getting-started-solution, with [lints]
│   ├── build.rs            #   Exercise 1 + Bonus: compiler version, build time
│   ├── rustfmt.toml        #   Exercise 5
│   ├── ANSWERS.md          #   written answers to every question
│   └── src/                #   same file and function names as exercise/
│
└── tests/                  # ← 15 tests, run against either crate
```

## The Commands You Need

```bash
# Run your own work, one exercise at a time
cargo run -p m01-getting-started -- 1

# Check your work (runs the tests against exercise/)
cargo test -p m01-getting-started-tests --features mine

# Run one test, by name
cargo test -p m01-getting-started-tests --features mine ex4_

# See the reference solution
cargo run -p m01-getting-started-solution -- 1       # Hello, Rust! (interactive)
cargo run -p m01-getting-started-solution -- 2       # Cargo basics
cargo run -p m01-getting-started-solution -- 3       # Documentation explorer
cargo run -p m01-getting-started-solution -- 4       # Guessing game (interactive)
cargo run -p m01-getting-started-solution -- 5       # Project setup
cargo run -p m01-getting-started-solution -- 6       # Debugging
cargo run -p m01-getting-started-solution -- bonus   # Build script output
cargo run -p m01-getting-started-solution -- all     # Every non-interactive part

# The tests pass against the solution
cargo test -p m01-getting-started-tests
```

The `--` separates Cargo's own arguments from your program's arguments.
Everything after it goes to `std::env::args()`.

### Interactive exercises

Exercises 1 and 4 read from the keyboard. Type your answers and press Enter;
Ctrl-D ends input. You can also pipe input in:

```bash
echo Ferris | cargo run -p m01-getting-started-solution -- 1
printf '50\n25\n12\n' | cargo run -p m01-getting-started-solution -- 4
```

## Suggested Workflow

1. Read the exercise in `exercises.md`.
2. Open the matching `exercise/src/exNN_*.rs` and replace the `todo!()`.
3. `cargo run -p m01-getting-started -- N` to see it work.
4. `cargo test -p m01-getting-started-tests --features mine exN_` to check it.
5. Compare with `solution/src/exNN_*.rs`.

When every exercise is done, delete the `#![allow(unused)]` line at the top of
`exercise/src/lib.rs` and fix whatever warnings appear.

### Exercise 2 as written

The exercise says `cargo new my_math --lib`. Do that too — somewhere outside
this repository, e.g. `~/scratch` — to see what Cargo generates. Inside this
repo the same code lives in `ex02_cargo_basics.rs` so everything builds from
one workspace.

### Exercise 5 and the Bonus

These are configuration. Add the `[lints]` tables to `exercise/Cargo.toml`,
create `exercise/rustfmt.toml` and `exercise/build.rs`, then:

```bash
cargo fmt -p m01-getting-started
cargo clippy -p m01-getting-started
```

## If You Get Stuck

1. `not yet implemented: Exercise N` — that is a `todo!()` you have not replaced yet.
2. Parsing always fails? You forgot `.trim()`.
3. The prompt appears after you type? Add `output.flush()?` after `write!`.
4. The game never ends when you pipe input? Check for `read_line` returning `Ok(0)`.
5. Run `rustc --explain E0308` (or whatever code you see) for a long explanation.
6. Compare against `solution/` — same file and function names as `exercise/`.
