# Getting Started with 02 · Basic Syntax

## Quick Start

All commands run from the **repository root**.

```bash
cargo build -p m02-basic-syntax-solution
cargo run   -p m02-basic-syntax-solution -- all
```

## What Is Already Here

```
02-basic-syntax/
├── README.md               # The concepts
├── exercises.md            # The work: 9 exercises plus a bonus
├── GETTING_STARTED.md      # This file
│
├── exercise/               # ← YOUR WORKSPACE (package m02-basic-syntax)
│   └── src/
│       ├── main.rs         #   ready-made runner: `-- 1` … `-- 9`, `-- bonus`, `-- all`
│       └── ex01_variables.rs … bonus_primes.rs   (bodies are todo!())
│
├── solution/               # ← REFERENCE IMPLEMENTATION (m02-basic-syntax-solution)
│   ├── ANSWERS.md          #   written answers
│   └── src/                #   same file and function names as exercise/
│
└── tests/                  # ← 29 tests (m02-basic-syntax-tests)
```

## The Commands You Need

```bash
# Run your own work
cargo run -p m02-basic-syntax -- 1

# Check your work
cargo test -p m02-basic-syntax-tests --features mine

# Only the tests for one exercise (test names start with ex1_, ex2_, ... bonus_)
cargo test -p m02-basic-syntax-tests --features mine ex5_

# See the reference solution
cargo run -p m02-basic-syntax-solution -- 1       # Variables and mutability
cargo run -p m02-basic-syntax-solution -- 2       # Data types
cargo run -p m02-basic-syntax-solution -- 3       # Functions
cargo run -p m02-basic-syntax-solution -- 4       # If/else
cargo run -p m02-basic-syntax-solution -- 5       # Loops
cargo run -p m02-basic-syntax-solution -- 6       # Match
cargo run -p m02-basic-syntax-solution -- 7       # Calculator (interactive)
cargo run -p m02-basic-syntax-solution -- 8       # FizzBuzz
cargo run -p m02-basic-syntax-solution -- 9       # Temperature converter (interactive)
cargo run -p m02-basic-syntax-solution -- bonus   # Primes
cargo run -p m02-basic-syntax-solution -- all     # Everything non-interactive

# Tests against the solution (always green)
cargo test -p m02-basic-syntax-tests
```

Script the interactive ones by piping input:

```bash
printf '5 + 3\n10 / 0\nquit\n' | cargo run -p m02-basic-syntax-solution -- 7
printf '100\nC\n'              | cargo run -p m02-basic-syntax-solution -- 9
```

## Try the Overflow Bonus Yourself

Add this to `exercise/src/ex02_data_types.rs` `run()`:

```rust
let x: i8 = std::hint::black_box(127);
println!("{}", x + 1);
```

```bash
cargo run -p m02-basic-syntax -- 2             # panics: attempt to add with overflow
cargo run -p m02-basic-syntax --release -- 2   # prints -128
```

(`black_box` stops the compiler from noticing the overflow at compile time,
where it would refuse to build at all.)

## If You Get Stuck

1. `not yet implemented: Exercise N` — a `todo!()` you have not replaced yet.
2. `expected i32, found ()` — remove the semicolon from the last expression.
3. `non-exhaustive patterns` — add the missing arm, or a `_ =>` catch-all.
4. A loop never ends — check that the condition variable actually changes.
5. Compare against `solution/src/` — same file and function names.
