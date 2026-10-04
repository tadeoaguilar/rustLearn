# Getting Started with 03 · Ownership & Borrowing

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m03-ownership-borrowing-solution -- all
```

## What Is Already Here

```
03-ownership-borrowing/
├── README.md               # The concepts
├── exercises.md            # The work: 8 exercises plus a bonus
├── GETTING_STARTED.md      # This file
│
├── exercise/               # ← YOUR WORKSPACE (package m03-ownership-borrowing)
│   └── src/
│       ├── main.rs         #   ready-made runner: `-- 1` … `-- 8`, `-- bonus`, `-- all`
│       └── ex01_ownership.rs … bonus_simple_rc.rs   (bodies are todo!())
│
├── solution/               # ← REFERENCE IMPLEMENTATION (m03-ownership-borrowing-solution)
│   ├── ANSWERS.md          #   written answers
│   └── src/                #   broken versions kept as comments, with the real compiler error
│
└── tests/                  # ← 21 tests (m03-ownership-borrowing-tests)
```

## The Commands You Need

```bash
# Run your own work
cargo run -p m03-ownership-borrowing -- 1

# Check your work
cargo test -p m03-ownership-borrowing-tests --features mine

# One exercise's tests (names start ex1_ … ex8_, bonus_)
cargo test -p m03-ownership-borrowing-tests --features mine ex7_

# The reference solution
cargo run -p m03-ownership-borrowing-solution -- 1       # Understanding ownership
cargo run -p m03-ownership-borrowing-solution -- 2       # References and borrowing
cargo run -p m03-ownership-borrowing-solution -- 3       # String vs &str
cargo run -p m03-ownership-borrowing-solution -- 4       # Dangling references
cargo run -p m03-ownership-borrowing-solution -- 5       # Clone vs Copy
cargo run -p m03-ownership-borrowing-solution -- 6       # Mutable vs immutable borrow
cargo run -p m03-ownership-borrowing-solution -- 7       # Text buffer
cargo run -p m03-ownership-borrowing-solution -- 8       # Ownership in collections
cargo run -p m03-ownership-borrowing-solution -- bonus   # SimpleRc
cargo run -p m03-ownership-borrowing-solution -- all     # Everything

cargo test -p m03-ownership-borrowing-tests              # always green
```

## This Module Is About Compiler Errors

Half of these exercises start from code that **doesn't compile**. Don't just
read them — paste each broken snippet into your `exercise/src` file, run

```bash
cargo build -p m03-ownership-borrowing
```

and read the whole error, including the `help:` lines. Then look up the code:

```bash
rustc --explain E0382    # use of moved value
rustc --explain E0502    # cannot borrow as mutable because also borrowed as immutable
rustc --explain E0499    # cannot borrow as mutable more than once
rustc --explain E0106    # missing lifetime specifier
rustc --explain E0507    # cannot move out of index
```

The solution files keep each broken version as a comment, with the exact error
it produces, directly above the fix.

## Catching Memory Bugs in the Bonus

The bonus uses `unsafe`. The compiler no longer checks those blocks for you, but
**Miri** (an interpreter that detects undefined behaviour) can:

```bash
rustup +nightly component add miri
cargo +nightly miri test -p m03-ownership-borrowing-tests bonus_
```

Try deleting the `ref_count -= 1` line in `Drop` and running Miri again — it
reports the leak. Try freeing on every drop instead — it reports the
use-after-free.

## If You Get Stuck

1. **E0382 use of moved value** — borrow (`&x`) instead of moving, or use it before the move.
2. **E0502 / E0499** — find the *last use* of the earlier borrow and move the mutation after it.
3. **Returning a reference to a local** — return the owned value instead.
4. **`cannot move out of index`** — borrow with `&v[i]`, or `.clone()` if you really need ownership.
5. Compare against `solution/src/` — same file and function names.
