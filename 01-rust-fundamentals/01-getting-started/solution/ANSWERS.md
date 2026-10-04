# Answers · 01 Getting Started

Written answers to the questions in `exercises.md`. Code answers are in `src/`.

## Exercise 2: Cargo Basics

**What's the difference between `cargo build` and `cargo build --release`?**

`cargo build` uses the `dev` profile: no optimisation (`opt-level = 0`), debug
info, overflow checks on. It compiles fast and runs slowly. `--release` uses the
`release` profile: `opt-level = 3`, overflow checks off. It compiles slowly and
can run 10–100× faster. Never benchmark a debug build.

**Where are the compiled binaries stored?**

In `target/debug/` and `target/release/`. In this repository all crates share
one workspace, so that is the **root** `target/` directory, e.g.
`target/debug/m01-getting-started-solution`.

**What files does Cargo generate?**

`cargo new my_math --lib` creates `Cargo.toml`, `src/lib.rs` (a binary gets
`src/main.rs` instead) and, unless you are already inside a git repo, a
`.git/` and `.gitignore`. The first build adds `Cargo.lock` (exact dependency
versions) and `target/` (build output — never commit it).

## Exercise 3: Documentation Explorer

**What's the difference between `String` and `&str`?**

`String` is an *owned*, growable, heap-allocated UTF-8 buffer. `&str` is a
*borrowed* view of UTF-8 bytes that live somewhere else — inside a `String`, or
in the binary itself for a literal like `"hello"`. Functions that only read
text should take `&str`; a `&String` converts to it automatically (deref
coercion), so callers can pass either.

**How do you find documentation for a specific method?**

`cargo doc --open` builds docs for your crate *and all its dependencies*;
`rustup doc --std` opens the standard library offline. Both have a search box
(press `S`). In an editor with rust-analyzer, hover the method or "Go to
definition".

**What does the `std::prelude` module contain?**

The names every Rust file imports automatically so you don't have to: `Option`,
`Some`, `None`, `Result`, `Ok`, `Err`, `String`, `Vec`, `Box`, `ToString`,
`ToOwned`, `Clone`, `Copy`, `Iterator`, `IntoIterator`, `Drop`, `Fn`/`FnMut`/
`FnOnce`, `From`/`Into`, and a few more. Edition 2021 added `TryFrom`, `TryInto`
and `FromIterator`.

## Exercise 6: Debugging Practice

**What was each error?**

1. *Unused variable* — `warning: unused variable: 'x'`. A warning, not an error;
   prefix with `_x` if it is deliberate.
2. *Wrong type conversion* — `error[E0277]: cannot add '&str' to '{integer}'`.
   Rust never converts implicitly. Fix: `let y: i32 = "10".parse()?;`
3. *Semicolon on the return expression* — `error[E0308]: mismatched types,
   expected 'i32', found '()'`. The compiler even suggests "remove this
   semicolon to return this value".

**How did the compiler help you?**

Each error has a code (`E0277`) you can look up with `rustc --explain E0277`,
points at the exact span, and often includes a `help:` line with the fix.

**What's the difference between `println!()` and `dbg!()`?**

`println!` writes your formatted text to stdout and returns `()`. `dbg!` writes
file, line, the expression's source text and its `Debug` value to stderr, and
returns the value — so you can wrap it around any expression in place. Remove
`dbg!` calls before committing; leave `println!` for real output.

## Exercise 7: Environment Setup

1. VS Code → install **rust-analyzer** (not the old "Rust" extension).
2. Format on save: `"editor.formatOnSave": true` and
   `"[rust]": { "editor.defaultFormatter": "rust-lang.rust-analyzer" }`.
3. Clippy hints: `"rust-analyzer.check.command": "clippy"`.
4. rust-analyzer adds **Run | Debug** lenses above `main` and every `#[test]`;
   bind `rust-analyzer.run` to a key for "run the thing under the cursor".

Verify by hovering a `let` with no type annotation — the inferred type should
appear.

## Bonus: Cargo Extensions

```bash
cargo install cargo-watch
cargo watch -x 'test -p m01-getting-started-tests --features mine'
```

`cargo add` and `cargo rm` used to come from `cargo-edit`; they are built into
Cargo since 1.62, so that install is no longer needed.
