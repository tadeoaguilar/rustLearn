# 38 · Procedural Macros

## Overview

`macro_rules!` (module 13) matches patterns. Procedural macros are
*programs*: Rust functions that the compiler runs on your code's tokens. They
power `#[derive(Serialize)]`, `#[tokio::main]`, `sqlx::query!` and clap's
derive. This module writes all three kinds: a serialization derive, an
ORM derive generating SQL, a state-machine DSL with its own grammar, and an
attribute macro that memoizes a function. Each one reports good compile
errors.

## What You'll Learn

| Exercise | Macro | Kind | The point |
|---|---|---|---|
| 1 | `#[derive(ToJson)]` | derive | `syn::DeriveInput`, `quote!`, generics, helper attributes |
| 2 | `#[derive(Table)]` | derive | Code the type checker validates; span-accurate errors |
| 3 | `state_machine!` | function-like | Your own grammar with `syn::parse::Parse` |
| 4 | `#[memoize]` | attribute | Rewriting a function; hygiene |
| Bonus | `#[derive(EnumIter)]` | derive | Generating constants and lookups |

## Key Concepts

### Three crates

```
solution/
├── core/      m38-proc-macros-core-solution    the expansions: TokenStream -> syn::Result<TokenStream>
├── derive/    m38-proc-macros-derive-solution  proc-macro = true: one-line wrappers over core
└── src/       m38-proc-macros-solution         runtime traits (json::ToJson, orm::Table) + re-exports
```

A `proc-macro` crate can only export macros, and its functions take the
compiler's `proc_macro::TokenStream`, which exists only inside the
compiler. Keeping the logic in a normal crate on `proc_macro2` types makes it
unit-testable. The runtime crate re-exports the macros, so users depend on
one crate (as with `serde` + `serde_derive`).

### Parse, generate, report

```rust
pub fn derive(input: TokenStream, krate: &Path) -> syn::Result<TokenStream> {
    let input: DeriveInput = syn::parse2(input)?;
    let name = &input.ident;
    let (impl_g, ty_g, where_c) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_g #krate::json::ToJson for #name #ty_g #where_c { ... }
    })
}
```

The wrapper turns errors into `compile_error!` at the right span:
`result.unwrap_or_else(|e| e.to_compile_error()).into()`. Use
`syn::Error::new_spanned(&field, "...")` to point the error at the code
that caused it.

### Seeing the output

`cargo install cargo-expand`, then
`cargo expand -p m38-proc-macros-solution --bin m38-proc-macros-solution`.
In the exercise, `cargo run -p m38-proc-macros -- 1` prints what your
expansion produces for a sample input.

## Common Pitfalls

1. **Relative paths in generated code** -- `Vec`, `ToJson` may not be in scope in the user's code; write `::std::vec::Vec`, `#krate::json::ToJson`
2. **`panic!` instead of `syn::Error`** -- a panic gives "proc-macro derive panicked" with no location
3. **Forgetting generics** -- use `split_for_impl`, and add the bounds the impl needs
4. **Silently ignoring unknown attribute options** -- a typo should fail to compile
5. **Checking types by name** -- a macro sees tokens, not types (`Option` may be a type alias); let generated trait bounds make the compiler check
6. **Collisions with the user's names** -- prefix generated items (`__FIB_MEMO`); local variables are hygienic, items are not
7. **Slow builds** -- `syn` with `full` is heavy; proc macros run on every build

## Running This Module

The exercises are in `exercise/core/src/`. The tests that *use* a macro
can't compile until it works, so they are opt-in per exercise in `mine`
mode (as in module 13):

```bash
cargo run  -p m38-proc-macros -- 1                              # print your Exercise 1 expansion
cargo test -p m38-proc-macros-tests --features mine             # your expansion functions (6 tests)
cargo test -p m38-proc-macros-tests --features mine,ex1         # ...plus the derive on real types
cargo test -p m38-proc-macros-tests --features mine,all         # everything
cargo run  -p m38-proc-macros-solution -- all                   # every macro in use
cargo test -p m38-proc-macros-tests                             # 17 tests against the solution
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository. It was written
to match the format of earlier modules and the exercises in
[the phase README](../README.md):
- a custom derive for serialization;
- an attribute macro;
- a DSL;
- an ORM with derive macros.

- The ORM's SQL is run by an in-memory SQLite in the tests (SQLx, as in
  Phase 4). No database server is involved.
- The phase README suggests a logging attribute macro. `#[memoize]` was
  chosen because it exercises the same rewriting, plus the
  early-`return` and re-entrancy problems.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[39 · Compiler Internals](../39-compiler-internals/)
