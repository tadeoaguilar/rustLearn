# Getting Started with 14 · Unsafe & FFI

## Quick Start

You need a C compiler:

```bash
cc --version                 # macOS: xcode-select --install   Linux: sudo apt install build-essential
```

All commands run from the **repository root**.

```bash
cargo run -p m14-unsafe-ffi-solution -- all
```

## What Is Already Here

```
14-unsafe-ffi/
├── README.md                 # The concepts
├── exercises.md              # The work: 7 exercises plus a bonus
├── GETTING_STARTED.md        # This file
│
├── exercise/                 # ← YOUR WORKSPACE (package m14-unsafe-ffi)
│   ├── csrc/shapes.{h,c}     #   the C library (provided)
│   ├── build.rs              #   compiles it with the `cc` crate
│   └── src/ex01_raw_pointers.rs … ex07_system.rs
│
├── solution/                 # ← REFERENCE IMPLEMENTATION (m14-unsafe-ffi-solution)
│   ├── csrc/, build.rs       #   same C library, symbols prefixed "sol_"
│   ├── ANSWERS.md
│   └── src/
│
└── tests/                    # ← 17 tests (m14-unsafe-ffi-tests)
```

## The Commands You Need

```bash
# Run your own work
cargo run -p m14-unsafe-ffi -- 1

# Check your work
cargo test -p m14-unsafe-ffi-tests --features mine

# One exercise's tests (names start ex1_ … ex7_)
cargo test -p m14-unsafe-ffi-tests --features mine ex5_

# The reference solution
cargo run -p m14-unsafe-ffi-solution -- 1       # raw pointers
cargo run -p m14-unsafe-ffi-solution -- 2       # libc: abs, strlen, qsort
cargo run -p m14-unsafe-ffi-solution -- 3       # the C shapes library
cargo run -p m14-unsafe-ffi-solution -- 4       # Rust called from C
cargo run -p m14-unsafe-ffi-solution -- 5       # StackVec
cargo run -p m14-unsafe-ffi-solution -- 6       # counting allocator (installed globally) + arena
cargo run -p m14-unsafe-ffi-solution -- 7       # getpid, gethostname
cargo run -p m14-unsafe-ffi-solution -- all

cargo test -p m14-unsafe-ffi-tests              # always green
```

## Exercise 6: Installing Your Allocator

`exercise/src/main.rs` has the `#[global_allocator]` attribute commented out.
Once `cargo test -p m14-unsafe-ffi-tests --features mine ex6_` passes,
uncomment it and run `cargo run -p m14-unsafe-ffi -- 6`. If your `alloc`
still panics, the program dies before `main` — that's why it starts disabled.

## The Bonus: Miri

```bash
rustup toolchain install nightly
rustup +nightly component add miri
cargo +nightly miri test -p m14-unsafe-ffi-tests -- ex1_ ex5_ ex6_arena
```

Miri can't execute C, so stick to the pure-Rust tests. Then break something on
purpose — e.g. in `StackVec::pop`, read `self.items[self.len]` *before*
decrementing `len` — and watch Miri report `using uninitialized data`.

## Looking at Symbols

```bash
cargo build -p m14-unsafe-ffi-solution
nm -g target/debug/m14-unsafe-ffi-solution | grep -E 'sol_(shapes|counter|rust)'
```

You'll see the C functions (`T`, defined) and the Rust exports side by side —
they're all just symbols to the linker.

## If You Get Stuck

1. **`linking with cc failed ... undefined symbol: _shapes_distance`** — the extern declaration's name doesn't match the C function (check spelling; the solution's are prefixed `sol_`).
2. **`error: extern blocks must be unsafe`** — Rust 2024: write `unsafe extern "C" { ... }`.
3. **`unsafe attribute used without unsafe`** — write `#[unsafe(no_mangle)]`.
4. **A test binary dies with `SIGSEGV` / `abort`** — a null pointer was dereferenced, or a panic hit an `extern "C" fn`. Run that test alone with `-- --test-threads=1 exact_name`.
5. **`` `*mut ...` cannot be sent between threads safely ``** — raw pointers aren't `Send`; decide (with a `SAFETY` argument) whether your wrapper can `unsafe impl Send`.
6. Compare against `solution/src/` — same file and function names.
