# 08 · Modules & Crates

## Overview

Up to now every exercise fit in one file. Real projects don't. This module is
about the tools Rust gives you to organise code and control what other code can
see: **modules** (namespaces inside a crate), **visibility** (`pub` and its
variants), **crates** (units of compilation and publishing), **features**
(optional parts of a crate) and **workspaces** (many crates, one build).

All exercises work on one library, a bookstore, that starts life as a single
300-line file and ends as a documented, feature-gated library with its money
handling split into a separate crate.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Modules | `mod`, file layout, `crate::` / `super::` paths |
| 2 | Visibility | Private fields + validating constructors; `pub(crate)`, `pub(super)` |
| 3 | Re-exports | `pub use` to decouple the public API from the file layout |
| 4 | Features | `#[cfg(feature)]`, optional dependencies, feature forwarding |
| 5 | Workspaces | A second crate, path dependencies, inherited `[workspace.package]` |
| 6 | Documentation | `///`, `//!`, runnable and `compile_fail` doc tests |
| 7 | Publishing | `cargo package`, SemVer, `#[non_exhaustive]` |
| Bonus | Feature dependencies | A `report` feature that turns on `discounts` |

## Key Concepts

### `mod` declares, `use` imports

```rust
mod catalog;              // "there is a module called catalog -- compile catalog.rs"
use catalog::Book;        // "let me write Book instead of catalog::Book here"
pub use catalog::Book;    // ...and let users of *this* module do the same
```

`mod` builds the tree; it appears exactly once per module. `use` only creates
shortcuts and can appear anywhere.

### File layout

```
src/lib.rs            crate root:   pub mod catalog;
src/catalog.rs        parent:       mod book; mod isbn;
src/catalog/book.rs   child
src/catalog/isbn.rs   child
```

`catalog/mod.rs` instead of `catalog.rs` also works (the pre-2018 style) — but
then every parent file in your editor is called `mod.rs`.

### Visibility levels

| Keyword | Visible to |
|---|---|
| (nothing) | the current module and its children |
| `pub(super)` | the parent module |
| `pub(crate)` | the whole crate, never outside it |
| `pub` | everyone who can reach the module |

A `pub struct` with **private fields** can't be built with a struct literal
outside its module. Combine that with a constructor that validates — `Isbn::parse`,
`Book::new` — and every value of that type in the program is known to be valid.
This is "parse, don't validate".

### Features are additive

```toml
[features]
default = ["discounts"]
discounts = []
json = ["dep:serde", "dep:serde_json", "m08-money-solution/serde"]
report = ["discounts"]
```

Cargo unifies features: if any crate in the build enables `json`, everyone gets
it. So a feature must only ever **add** API — never remove or change it. And
test every combination you support; `#[cfg]` code that's never compiled is code
that's silently broken.

### Re-exports make the layout private

Users who write `bookstore::Book` or `use bookstore::prelude::*` never mention
`catalog::book`. You can move `Book` to another file without a breaking change.

## Common Pitfalls

1. **`mod foo;` in two places** — a module is declared once; elsewhere, `use` it
2. **Making fields `pub` "for now"** — every user can now break your invariants
3. **Subtractive features** (`no_std`-style "disable X") — Cargo unification breaks them
4. **Untested feature combinations** — build with `--no-default-features` in CI
5. **Path dependencies in a published crate** — they need a `version` too

## Running This Module

```bash
cargo run  -p m08-modules-crates -- all                          # your code
cargo test -p m08-modules-crates-tests --features mine           # test your code
cargo run  -p m08-modules-crates-solution --all-features -- all  # the reference solution
cargo test -p m08-modules-crates-tests                           # 13 tests against the solution
cargo test -p m08-modules-crates-solution --all-features --doc   # 6 doc tests, 2 of them compile_fail
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of the others and the topics listed in
[the phase README](../README.md). Its shape is slightly different from the
other modules:

- The **exercise crate contains a working monolith**,
  `exercise/src/legacy_bookstore.rs`, next to the skeleton files of the target
  layout. You move code from one to the other.
- The **money crate** is a separate workspace member in `crates/money`, both in
  `exercise/` and `solution/`.
- The **tests crate enables the `json` and `report` features** on both
  bookstore crates.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[09 · Testing](../09-testing/)
