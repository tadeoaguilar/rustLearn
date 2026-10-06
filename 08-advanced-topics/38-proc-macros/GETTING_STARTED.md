# Getting Started with 38 · Procedural Macros

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m38-proc-macros-solution -- all
```

shows every macro in use:
- nested structs and enums serialized to JSON, with a skipped password;
- `CREATE TABLE`/`INSERT`/`SELECT` generated from a struct;
- a door state machine refusing invalid events;
- `fib(90)` computed instantly through the memoization cache;
- an enum listing its variants.

## What Is Already Here

```
38-proc-macros/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/                  # package m38-proc-macros: runtime traits + a runner (provided)
│   ├── core/src/              # ← YOUR WORKSPACE (package m38-proc-macros-core)
│   │   ├── ex01_to_json.rs    #   Ex 1
│   │   ├── ex02_table.rs      #   Ex 2
│   │   ├── ex03_state_machine.rs  # Ex 3
│   │   ├── ex04_memoize.rs    #   Ex 4
│   │   ├── bonus_enum_iter.rs #   bonus
│   │   └── attrs.rs           #   provided: #[json(..)] / #[column(..)] parsing
│   └── derive/                # the proc-macro crate (provided: wrappers over core)
├── solution/                  # ← REFERENCE: same three crates + ANSWERS.md
└── tests/                     # ← 17 tests (m38-proc-macros-tests)
    └── src/
        ├── expansion.rs       #   calls your core functions directly (always compiled)
        └── ex1.rs .. bonus.rs #   uses the macros on real types (opt-in with --features mine)
```

## The Commands You Need

```bash
cargo run  -p m38-proc-macros -- 2                          # what does my Table derive generate?
cargo test -p m38-proc-macros-tests --features mine ex2_    # my expansion tests for Ex 2
cargo test -p m38-proc-macros-tests --features mine,ex2     # ...and the derive on real structs + SQLite
cargo test -p m38-proc-macros-tests --features mine,all
cargo test -p m38-proc-macros-tests                         # the solution: always green
```

With `--features mine,exN`, a macro that's still `todo!()` fails to
*compile* with "proc-macro derive panicked ... not yet implemented". That's
expected. Turn on one exercise at a time.

Seeing generated code: `cargo install cargo-expand`, then
`cargo expand -p m38-proc-macros-tests --features mine,ex1 --lib --tests ex1`.

## If You Get Stuck

1. **`expected one of ...` in your generated code** -- print it (`cargo run -p m38-proc-macros -- N`) and paste it into a file to see where it breaks; `quote!` output is plain Rust.
2. **`ToJson` not found in the tests** -- generated code must use `#krate::json::ToJson`, not `ToJson`.
3. **Repeating with `#(...),*`** -- the variable must be an iterator or `Vec` of `ToTokens` values; two vectors in one repetition must have the same length.
4. **A string literal from a `String`** -- `quote!` turns a `String` variable into `"..."` automatically.
5. **Identifiers from strings** -- `format_ident!("{}State", name)`.
6. **`unique` order wrong in Ex 3** -- start with the initial state, then from/to of each transition, keeping first appearances.
7. **`#[memoize]` panics "already borrowed"** -- don't hold the `RefCell` borrow while the body runs (recursion re-enters).
8. Compare with `solution/core/src/` -- same file and function names.
