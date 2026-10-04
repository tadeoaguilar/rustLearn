# Getting Started with 05 · Error Handling

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m05-error-handling-solution -- all
```

## What Is Already Here

```
05-error-handling/
├── README.md               # The concepts
├── exercises.md            # The work: 9 exercises plus a bonus
├── GETTING_STARTED.md      # This file
│
├── exercise/               # ← YOUR WORKSPACE (package m05-error-handling)
│   ├── Cargo.toml          #   rand, thiserror and anyhow already added
│   └── src/ex01_panic.rs … bonus_combinators.rs   (bodies are todo!())
│
├── solution/               # ← REFERENCE IMPLEMENTATION (m05-error-handling-solution)
│   ├── ANSWERS.md
│   └── src/
│
└── tests/                  # ← 30 tests (m05-error-handling-tests), using `tempfile`
```

## The Commands You Need

```bash
# Run your own work
cargo run -p m05-error-handling -- 3

# Check your work
cargo test -p m05-error-handling-tests --features mine

# One exercise's tests (names start ex1_ … ex9_, bonus_)
cargo test -p m05-error-handling-tests --features mine ex8_

# The reference solution
cargo run -p m05-error-handling-solution -- 1       # Panic basics
cargo run -p m05-error-handling-solution -- 2       # Result basics
cargo run -p m05-error-handling-solution -- 3       # The ? operator
cargo run -p m05-error-handling-solution -- 4       # Custom error types
cargo run -p m05-error-handling-solution -- 5       # Converting between error types
cargo run -p m05-error-handling-solution -- 6       # Option <-> Result
cargo run -p m05-error-handling-solution -- 7       # Retry and fallbacks
cargo run -p m05-error-handling-solution -- 8       # Config file parser
cargo run -p m05-error-handling-solution -- 9       # thiserror and anyhow
cargo run -p m05-error-handling-solution -- bonus   # Result combinators
cargo run -p m05-error-handling-solution -- all     # Everything

cargo test -p m05-error-handling-tests              # always green
```

## Trying Exercise 8 With a Real File

```bash
printf 'port = 8080\nhost = localhost\ndebug = true\n# comment\n' > /tmp/config.txt
```

Then call `start_server("/tmp/config.txt")` from your `run()`. Break the file
(`port = eighty`, a line without `=`) and check each error message names the
problem and the line.

## Seeing Panics With Backtraces

```bash
RUST_BACKTRACE=1 cargo test -p m05-error-handling-tests --features mine ex1_
```

## If You Get Stuck

1. **`the ? operator can only be used in a function that returns Result`** — change the return type (even `main` can return `Result<(), Box<dyn Error>>`).
2. **`the trait From<X> is not implemented for Y`** — `?` needs a `From<X> for Y` impl; write it, or use `#[from]` with thiserror.
3. **`` `MyError` doesn't implement `std::fmt::Display` ``** — `Error` requires `Display` and `Debug`.
4. **Retry closure won't compile** — it mutates a counter, so it must be `FnMut`, and the variable `mut`.
5. Compare against `solution/src/` — same file and function names.
