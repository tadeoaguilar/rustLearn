# Answers · 08 Modules & Crates

## Exercise 1

**What is the difference between `mod foo;` and `use foo;`?**

`mod foo;` *creates* a module: it tells the compiler to read `foo.rs` (or
`foo/mod.rs`) and place its contents in the module tree at this point. Each
module is declared exactly once. `use` creates a local alias for a path that
already exists, so you can write `Book` instead of `crate::catalog::Book`. It
doesn't load any file.

**When would you prefer `foo/mod.rs` over `foo.rs` + `foo/`?**

Mostly for consistency with an existing codebase, or when a directory should be
self-contained (move or delete one folder and everything goes with it). The
`foo.rs` + `foo/` layout is the default since Rust 2018 because editors show
meaningful tab names instead of a row of `mod.rs`.

## Exercise 2

**Why can't a `pub` struct with private fields be built with `Book { .. }`
outside its module?**

A struct literal must name every field, and you can't name a field you can't
see. So the only way to obtain a `Book` from outside is whatever the module
offers — `Book::new`, which validates. (The same applies to pattern-matching
`Book { title, .. }`: you can only destructure visible fields.)

**What is the difference between `pub(crate)` and `pub` in a library? In a binary?**

In a library, `pub` items reachable from the crate root become part of your
public API — other crates can use them, and changing them is a breaking change.
`pub(crate)` items are usable anywhere inside your crate but invisible outside.
In a binary there is no "outside", so the two behave the same; `pub(crate)` is
still useful to document intent, and it stops the `unreachable_pub` lint from
complaining.

## Exercise 3

**Which standard library preludes do you use without noticing?**

`std::prelude::rust_2024` — `Option`, `Result`, `Vec`, `String`, `Box`, the
`Clone`/`Copy`/`Iterator`/`From`/`Into` traits and more — is imported into
every module automatically. Many crates copy the pattern: `std::io::prelude`,
`rayon::prelude`, `tokio::prelude` (historically), `bevy::prelude`.

## Exercise 5

**What does a workspace share between its members?**

One `Cargo.lock` (so all members use the same dependency versions), one
`target/` directory (so shared dependencies are compiled once), and optionally
`[workspace.package]` metadata, `[workspace.dependencies]` versions and
`[workspace.lints]` that members inherit with `.workspace = true`.

**Why can't two workspace members depend on each other in a cycle?**

A crate is compiled after its dependencies, as a unit. If A depends on B and B
on A, neither can be compiled first. (Dev-dependencies can form a cycle in
limited cases, because tests are compiled after the library.) The fix is to
move the shared part into a third crate both depend on — exactly what the
money crate is.

## Exercise 7

**Which of these is a breaking change?**

| Change | Breaking? | Why |
|---|---|---|
| Adding a public function | No | Minor version bump. (Technically it can clash with a glob import, which the SemVer guide accepts as "minor".) |
| Adding a field to a struct with all-public fields | **Yes** | Users' struct literals and exhaustive patterns stop compiling |
| Adding a variant to a public enum | **Yes** | Users' exhaustive `match`es stop compiling |
| Making a private field public | No | It only allows more |

**What does `#[non_exhaustive]` change?**

On an enum, users outside your crate must include a `_ =>` arm, so you can add
variants in a minor release. On a struct, users outside your crate can't build
it with a literal or destructure it exhaustively, so you can add fields.
`pricing::Discount` in the solution is `#[non_exhaustive]` for this reason.
