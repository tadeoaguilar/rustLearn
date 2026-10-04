# Getting Started with 13 · Macros

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m13-macros-solution -- all
```

## What Is Already Here

```
13-macros/
├── README.md                   # The concepts
├── exercises.md                # The work: 7 exercises plus a bonus
├── GETTING_STARTED.md          # This file
│
├── exercise/                   # ← YOUR WORKSPACE
│   ├── Cargo.toml              #   package m13-macros
│   ├── derive/                 #   package m13-macros-derive (proc-macro = true)
│   │   └── src/lib.rs          #   Describe, Builder, timed: registered, generate nothing yet
│   └── src/
│       ├── ex01_basics.rs …    #   what to write, as comments; helper functions provided
│       └── ex06_derive.rs …    #   types already annotated with #[derive(Describe)] etc.
│
├── solution/                   # ← REFERENCE IMPLEMENTATION
│   ├── derive/src/lib.rs       #   m13-macros-derive-solution
│   └── src/                    #   m13-macros-solution
│
└── tests/                      # ← 30 tests, one file per exercise (src/ex1.rs …)
```

## The Commands You Need

```bash
# Run your own work
cargo run -p m13-macros -- 1

# Test one exercise of your own (each is opt-in, see below)
cargo test -p m13-macros-tests --features mine,ex1
cargo test -p m13-macros-tests --features mine,ex1,ex2,ex3
cargo test -p m13-macros-tests --features mine,all

# The reference solution
cargo run -p m13-macros-solution -- 1       # square!, max!, hashmap!
cargo run -p m13-macros-solution -- 2       # newtype!, string_enum!, count!
cargo run -p m13-macros-solution -- 3       # const_assert!, nonzero!, hex_color!
cargo run -p m13-macros-solution -- 4       # state_machine! DSL
cargo run -p m13-macros-solution -- 5       # hygiene, my_assert_eq!, traced!
cargo run -p m13-macros-solution -- 6       # #[derive(Describe)]
cargo run -p m13-macros-solution -- 7       # #[derive(Builder)]
cargo run -p m13-macros-solution -- bonus   # #[timed]
cargo run -p m13-macros-solution -- all

cargo test -p m13-macros-tests              # always green
cargo test -p m13-macros-solution --doc     # includes the compile_fail checks
```

## Why Are the Tests Opt-In?

`tests/src/ex6.rs` calls `Point::describe()`. Until your `Describe` derive
generates that method, the file doesn't compile — and one file that doesn't
compile stops *every* test in the crate. So in `mine` mode each exercise's
test file is compiled only when you name it: `--features mine,ex6`.

## See What a Macro Expands To

```bash
cargo install cargo-expand          # uses the nightly toolchain under the hood
rustup toolchain install nightly    # if you don't have it
cargo expand -p m13-macros-solution ex04_dsl
cargo expand -p m13-macros-solution ex07_builder
```

The second one shows the whole generated `CommandBuilder`.

## Debugging a Proc Macro

- Print what you generate: `eprintln!("{}", tokens);` inside the macro shows up during `cargo build`.
- Make errors point somewhere useful: `syn::Error::new_spanned(&field.ty, "...")`.
- `cargo build -p m13-macros -vv` shows the full compiler output.

## If You Get Stuck

1. **`cannot find macro 'square' in this scope`** — missing `#[macro_export]`, or calling it as `crate::ex01_basics::square!` (exported macros live at the crate root).
2. **`no rules expected the token`** — your matcher doesn't accept that input; check separators (`,`) and the optional trailing comma `$(,)?`.
3. **`recursion limit reached`** — a recursive rule never reaches its base case.
4. **`proc-macro derive panicked`** — an `unwrap()` in your macro failed; return a `syn::Error` instead.
5. **`no function or associated item named 'describe'`** — your derive returns an empty `TokenStream`; nothing was generated yet.
6. Compare against `solution/` — same files and names.
