# 13 · Macros

## Overview

Macros are code that writes code, run by the compiler. They're the right tool
when a function or a generic can't express what you need: a variable number of
arguments (`vec!`, `println!`), generating new items (`#[derive(Debug)]`), a
custom syntax, or rejecting bad input at compile time. They're the wrong tool
for everything else — a macro is harder to read, debug and document than the
function it replaces.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | `macro_rules!` | Fragments, repetition, recursion, `$crate`, evaluate-once |
| 2 | Generating items | Newtypes and string enums; `stringify!`; counting tokens |
| 3 | Compile-time validation | `const fn` + `const { }` + `panic!` = a compile error |
| 4 | A DSL | A state-machine syntax parsed by `macro_rules!` |
| 5 | Hygiene & debugging | Why a macro's `let x` is invisible; `file!`/`line!`; `cargo expand` |
| 6 | Derive macro | `#[derive(Describe)]` with `syn` and `quote`, generics included |
| 7 | Builder derive | Generating a whole new type; `Option<T>` detection; helper attributes |
| Bonus | Attribute macro | `#[timed]` wrapping a body without breaking `return` and `?` |

## Key Concepts

### Two kinds of macro

| | `macro_rules!` | Procedural |
|---|---|---|
| Written as | pattern → template | a Rust function `TokenStream → TokenStream` |
| Lives in | any crate | its own crate with `proc-macro = true` |
| Forms | `name!(...)` | `#[derive(X)]`, `#[attr]`, `name!(...)` |
| Good for | small syntax sugar, repetition | inspecting types and fields, generating impls |
| Tools | — | `syn` (parse), `quote` (generate), `proc-macro2` |

### Evaluate arguments once

```rust
macro_rules! square_naive { ($x:expr) => { $x * $x }; }
square_naive!(next())          // calls next() TWICE
macro_rules! square { ($x:expr) => {{ let v = $x; v * v }}; }
```

A macro pastes tokens; every `$x` in the template is another copy of the
argument expression.

### Hygiene

Variables a macro *invents* (`let v = ...` above) live in the macro's own
syntax context: they can't clash with the caller's variables, and the caller
can't see them. To create a name the caller can use, take it as an `$name:ident`
argument. Note: hygiene covers local variables, not items or paths — that's
why exported macros write `$crate::path` and `::std::...` in full.

### Compile-time checks

```rust
macro_rules! hex_color {
    ($s:literal) => { const { match parse_hex_color($s) { Ok(c) => c, Err(_) => panic!("invalid hex color") } } };
}
```

A panic during constant evaluation is a compile error (E0080), so
`hex_color!("#ff88")` doesn't build. The parsing itself is a normal
`const fn`.

### Procedural macro errors

Never `panic!` in a proc macro — the user gets a useless "proc macro panicked".
Return `syn::Error::new_spanned(tokens, "message").to_compile_error()` and the
compiler underlines the right spot in the user's code.

## Common Pitfalls

1. **`$x * $x`** — double evaluation
2. **Calling helpers without `$crate::`** — breaks when used from another crate
3. **Referring to `HashMap`, `Option`, `Ok` unqualified in generated code** — the user may have shadowed them; write `::std::...`
4. **Generating references to *your* crate from a proc macro** — a proc-macro crate can't know its users' paths; generate inherent impls, or take the path as input
5. **Panicking in a proc macro** — return a `syn::Error` instead
6. **A macro where a function would do** — prefer the function

## Running This Module

```bash
cargo run  -p m13-macros -- 1                                  # your code
cargo test -p m13-macros-tests --features mine,ex1             # test your Exercise 1
cargo test -p m13-macros-tests --features mine,all             # test everything you've done
cargo run  -p m13-macros-solution -- all                       # the reference solution
cargo test -p m13-macros-tests                                 # 30 tests against the solution
cargo test -p m13-macros-solution --doc                        # 12 doc tests, 4 of them compile_fail
cargo expand -p m13-macros-solution ex02_items                 # see what the macros expand to
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of the others and the exercise list in
[the phase README](../README.md) (compile-time validation, a custom derive, a
DSL, a builder derive). Three things differ from other modules:

- **Two crates per side.** Procedural macros must live in a `proc-macro`
  crate: `exercise/derive` (`m13-macros-derive`) and `solution/derive`
  (`m13-macros-derive-solution`). The main crates re-export them.
- **The skeleton has no macros.** A `macro_rules!` with a `todo!()` body
  wouldn't tell you anything, so `exercise/src/exNN_*.rs` describe what to
  write instead. The proc-macro entry points exist but generate nothing.
- **Tests are opt-in per exercise** in `mine` mode, because tests that use
  `Point::describe()` can't compile until your derive generates it. Use
  `--features mine,ex1`, `mine,ex1,ex2`, … or `mine,all`.

## Next

[14 · Unsafe & FFI](../14-unsafe-ffi/)
