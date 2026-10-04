# 01 · Getting Started

## Overview

Before ownership, traits or async, you need a working loop: **edit → build →
run → test**. This module is about getting that loop fast and getting
comfortable with the two tools you will use every day, `cargo` and the
compiler's error messages.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Hello, Rust! | Reading stdin, `trim()`, and why the prompt needs `flush()` |
| 2 | Cargo basics | Library vs binary, unit tests, doc tests, `--release` |
| 3 | Documentation explorer | `String` vs `&str`, finding things in the std docs |
| 4 | Guessing game | `rand`, `Ordering`, loops, parsing that can fail |
| 5 | Project setup | `[lints]` in `Cargo.toml`, `rustfmt.toml`, clippy |
| 6 | Debugging | Reading compiler errors, `dbg!` vs `println!` |
| 7 | Environment | rust-analyzer and editor setup (no code) |
| Bonus | Build scripts | `build.rs` and baking values in at compile time |

## Key Concepts

### Separate logic from I/O, even in a toy program

The guessing game in the solution is three layers:

```rust
fn check_guess(guess: u32, secret: u32) -> Ordering        // pure logic
fn play(secret: u32, input: impl BufRead, output: impl Write) // the loop
fn secret_number() -> u32                                   // randomness
```

`play` never calls `stdin()` itself. `main` passes `io::stdin().lock()`, and the
tests pass `"50\n75\n62\n".as_bytes()`. That one decision is the difference
between a program you can only test by typing and one with eleven automated
tests.

### `read_line` keeps the newline

```rust
let mut name = String::new();
stdin().read_line(&mut name)?;   // name == "John\n"
let name = name.trim();          // name == "John"
```

Forget `trim()` and `"50\n".parse::<u32>()` fails every time, or your greeting
prints `Hello, John` and then `! Welcome to Rust!` on the next line.

### The last expression is the return value

```rust
fn double(n: i32) -> i32 {
    n * 2      // returned
}
fn broken(n: i32) -> i32 {
    n * 2;     // a statement: the block now evaluates to (), error[E0308]
}
```

### `dbg!` vs `println!`

| | `println!` | `dbg!` |
|---|---|---|
| Writes to | stdout | stderr |
| Prints | only what you format | file, line, expression **and** value |
| Returns | `()` | the value, so you can wrap any expression |

```rust
let total: i32 = values.iter().map(|v| dbg!(v * v)).sum();
```

### Compile-time values with `build.rs`

A build script runs before your crate compiles. Anything it prints as
`cargo:rustc-env=NAME=value` is readable in the crate with `env!("NAME")`. The
solution uses it to embed the compiler version (Exercise 1) and the build time
(Bonus).

## Common Pitfalls

1. **Forgetting `trim()`** after `read_line` — every parse fails
2. **No `flush()` after `print!`** — the prompt appears *after* you type
3. **Looping forever on end of input** — `read_line` returns `Ok(0)` at EOF; check it
4. **`use Enum::*`** — hides where names come from; this module's lint denies it
5. **Reading `cargo build` output as "it works"** — it only means it compiles; run the tests

## Running This Module

```bash
cargo run  -p m01-getting-started -- 1                          # your code, exercise 1
cargo test -p m01-getting-started-tests --features mine         # test your code
cargo run  -p m01-getting-started-solution -- all               # the reference solution
cargo test -p m01-getting-started-tests                         # 15 tests against the solution
```

See [GETTING_STARTED.md](GETTING_STARTED.md) for the full walkthrough.

## Notes on `exercises.md`

- **Exercise 1** expects "Rust version: 1.75.0". Your number will be whatever
  `rustc --version` says; the solution reads it at compile time in `build.rs`.
- **Exercise 6** lists "missing semicolon in return" as a bug to introduce. In
  Rust it is the other way round: *adding* a semicolon to the final expression
  is the bug. See `solution/src/ex06_debugging.rs`.
- **Bonus** calls `chrono::Utc::now()` inside `build.rs`. That needs
  `chrono` under `[build-dependencies]`, not `[dependencies]` — a common
  surprise. The solution avoids the dependency with `std::time::SystemTime`.
- **Exercises 3 and 7** are mostly reading and setup. Written answers to every
  question in this module are in [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[02 · Basic Syntax](../02-basic-syntax/)
