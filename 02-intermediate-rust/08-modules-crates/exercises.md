# Exercises: Modules & Crates

All exercises in this module work on one small library: **a bookstore**. It has
a catalog of books, a shopping cart, discounts and money handling. You start
with everything in one file and end with a properly organised, documented
library plus a second crate in the workspace.

The code for the whole bookstore already exists, in one file:
`exercise/src/legacy_bookstore.rs`. It works. It's also 300 lines with no
structure, everything is public, and nothing is documented. Your job is to fix
that.

---

## Exercise 1: Refactor a Large File into Modules

**Difficulty**: Easy
**Time**: 30 minutes

**Learning Objectives**:
- Declare modules with `mod`
- Choose between `foo.rs` and `foo/mod.rs` layouts
- Refer to items with `crate::`, `super::` and `self::`

**Task**: Move the code from `legacy_bookstore.rs` into this layout:

```
src/
├── lib.rs              // mod declarations only
├── catalog.rs          // mod book; mod isbn; + Catalog
├── catalog/
│   ├── book.rs         // Book
│   └── isbn.rs         // Isbn
├── cart.rs             // Cart, CartLine, CartError
├── pricing.rs          // Discount
└── prelude.rs          // (Exercise 3)
```

```rust
// lib.rs
pub mod catalog;
pub mod cart;
pub mod pricing;
```

```rust
// catalog.rs -- a module with child modules, "modern" (2018+) layout:
// the children live in catalog/, the parent in catalog.rs (no mod.rs).
mod book;
mod isbn;

pub use book::Book;
pub use isbn::{Isbn, IsbnError};
```

**Expected Result**: these paths resolve:
```rust
use bookstore::catalog::{Book, Catalog, Isbn};
use bookstore::cart::Cart;
use bookstore::pricing::Discount;
```

**Hints**:
- A `mod foo;` declaration tells the compiler to look for `foo.rs` or `foo/mod.rs`
- Inside `cart.rs`, a `Book` is `crate::catalog::Book` (absolute) or `super::catalog::Book` (relative)
- Fix one compiler error at a time — usually a missing `use`

**Questions**:
- What is the difference between `mod foo;` and `use foo;`?
- When would you prefer `foo/mod.rs` over `foo.rs` + `foo/`?

---

## Exercise 2: Visibility and Encapsulation

**Difficulty**: Medium
**Time**: 40 minutes

**Learning Objectives**:
- Use `pub`, `pub(crate)`, `pub(super)` and private items deliberately
- Protect invariants with private fields and validating constructors

**Background**: In the legacy file, every field is `pub`. Anyone can write
`book.price = Money::from_cents(-500)` or build an `Isbn` from garbage.

**Tasks**:
1. Make `Isbn` a newtype with a **private** field. The only way to get one is
   `Isbn::parse(&str) -> Result<Isbn, IsbnError>`, which validates an ISBN-13
   (13 digits, hyphens allowed, valid checksum).
2. Make all `Book` fields private. Add `Book::new(isbn, title, author, price)`
   that rejects an empty title and a negative price, plus getters.
3. The rounding helper `round_half_up` is used by `pricing` but must not be
   part of the public API. Put it in a private module `rounding` (`mod
   rounding;` without `pub`) and make the function `pub(crate)`.
4. `Book::search_key()` (lower-cased "title author", used by
   `Catalog::search`) is an implementation detail. It lives in
   `catalog/book.rs` but is called from `catalog.rs`: make it `pub(super)` —
   visible to the parent module, invisible to users of the crate.

**ISBN-13 checksum**:
```
digits d1..d13: sum of d_i * (1 if i odd, 3 if i even) must be divisible by 10
9780306406157 -> valid
9780306406158 -> invalid
```

**Tests**:
```rust
assert!(Isbn::parse("978-0-306-40615-7").is_ok());
assert_eq!(Isbn::parse("978-0-306-40615-8"), Err(IsbnError::BadChecksum));
assert_eq!(Isbn::parse("12345"), Err(IsbnError::WrongLength(5)));
```

**Questions**:
- Why can't a `pub` struct with private fields be built with `Book { .. }` outside its module?
- What is the difference between `pub(crate)` and `pub` in a library? In a binary?

---

## Exercise 3: Re-exports and a Prelude

**Difficulty**: Easy
**Time**: 20 minutes

**Learning Objectives**:
- Shape a public API with `pub use`
- Provide a prelude module

**Tasks**:
1. Re-export the main types at the crate root, so users can write
   `bookstore::Book` as well as `bookstore::catalog::Book`.
2. Create `prelude.rs` that re-exports everything a typical user needs:
   ```rust
   use bookstore::prelude::*;
   ```
3. Keep internal module structure free to change: users who only use the
   root re-exports or the prelude must not break if you move `Book` to a
   different file.

**Questions**:
- Which standard library preludes do you use without noticing?

---

## Exercise 4: Features and Conditional Compilation

**Difficulty**: Medium
**Time**: 40 minutes

**Learning Objectives**:
- Declare Cargo features
- Use `#[cfg(feature = "...")]` and optional dependencies

**Tasks**:
1. Put all discount code behind a **default** feature called `discounts`:
   ```toml
   [features]
   default = ["discounts"]
   discounts = []
   ```
   With `--no-default-features`, `pricing::Discount` and
   `Cart::total_with_discount` must not exist, and the crate must still compile.
2. Add a feature `json` that enables an **optional** dependency on
   `serde`/`serde_json`, derives `Serialize` for `Book`, and adds
   `Catalog::to_json(&self) -> String`.
   ```toml
   serde = { workspace = true, optional = true }
   serde_json = { workspace = true, optional = true }
   json = ["dep:serde", "dep:serde_json"]
   ```
3. Use `#[cfg_attr(feature = "json", derive(serde::Serialize))]` on `Book`.

**Test all combinations**:
```bash
cargo build -p m08-modules-crates --no-default-features
cargo build -p m08-modules-crates --no-default-features --features json
cargo build -p m08-modules-crates --all-features
```

**Hints**:
- Features are **additive**: enabling one must never remove functionality
- `cfg!(feature = "json")` gives a `bool` at runtime-looking places; `#[cfg]` removes code entirely

---

## Exercise 5: A Workspace with Multiple Crates

**Difficulty**: Medium
**Time**: 45 minutes

**Learning Objectives**:
- Split code into separate crates inside one workspace
- Use path dependencies and workspace-inherited settings

**Background**: Money handling (`Money`, formatting, parsing `"$12.34"`) is
useful outside the bookstore. Move it into its own crate.

**Tasks**:
1. Create the crate `exercise/crates/money` (package `m08-money`) with
   `Money { cents: i64 }`, `Display` as `$12.34`, `FromStr`, `Add`, `Sub`,
   `Mul<u32>`, `Sum`.
2. In `exercise/Cargo.toml`, depend on it by path:
   ```toml
   m08-money = { path = "crates/money" }
   ```
3. Re-export it so bookstore users don't need a second dependency:
   `pub use m08_money::Money;`
4. Use `version.workspace = true` and friends so the new crate inherits from
   the root `Cargo.toml`.

**Tests**:
```rust
assert_eq!(Money::from_cents(1234).to_string(), "$12.34");
assert_eq!(Money::from_cents(-5).to_string(), "-$0.05");
assert_eq!("$12.34".parse::<Money>(), Ok(Money::from_cents(1234)));
assert_eq!(Money::from_cents(250) * 3, Money::from_cents(750));
```

**Questions**:
- What does a workspace share between its members? (Look at `Cargo.lock` and `target/`.)
- Why can't two workspace members depend on each other in a cycle?

---

## Exercise 6: Documentation

**Difficulty**: Easy
**Time**: 30 minutes

**Learning Objectives**:
- Write `//!` and `///` doc comments
- Make examples that are compiled and run as tests
- Use `compile_fail` examples to document what is *not* allowed

**Tasks**:
1. Add `#![warn(missing_docs)]` to `lib.rs` and document every public item
2. Add a runnable example to `Isbn::parse`, `Cart::total` and `Money`'s `Display`
3. Add a `compile_fail` example showing that `Book` can't be built with a
   struct literal outside its module
4. Generate and browse the docs:
   ```bash
   cargo doc -p m08-modules-crates --open
   ```

**Example**:
```rust
/// Parses an ISBN-13, with or without hyphens.
///
/// ```
/// use m08_modules_crates::catalog::Isbn;
/// let isbn = Isbn::parse("978-0-306-40615-7")?;
/// assert_eq!(isbn.to_string(), "9780306406157");
/// # Ok::<(), m08_modules_crates::catalog::IsbnError>(())
/// ```
```

**Note**: Doc tests are turned off in the exercise crate (`doctest = false` in
its `Cargo.toml`) because the skeleton's `todo!()` bodies would fail them.
Remove that line once your implementation is complete.

---

## Exercise 7: Publishing (Optional)

**Difficulty**: Easy
**Time**: 30 minutes

**Tasks**:
1. Outside this repository, `cargo new --lib` a tiny crate
2. Fill in `description`, `license`, `repository` in `Cargo.toml`
3. Run `cargo package --list` and `cargo publish --dry-run`
4. Read the [SemVer compatibility rules](https://doc.rust-lang.org/cargo/reference/semver.html)

**Questions**:
- Which of these is a breaking change: adding a public function; adding a
  field to a struct with all-public fields; adding a variant to a public enum;
  making a private field public?
- What does `#[non_exhaustive]` change?

(The crates in this repository have `publish = false`, so `cargo publish`
refuses to run on them — on purpose.)

---

## Bonus Challenge: Feature-Gated Inventory Report

**Difficulty**: Hard
**Time**: 45 minutes

Add a `report` module, compiled only with feature `report`, that produces an
inventory summary:

```rust
let report = catalog.report();
println!("{report}");
// 3 books by 3 authors, total list value $134.97
// Most expensive: Programming Rust ($49.99)
```

Requirements:
- `report` depends on `discounts` (enabling it enables `discounts`)
- Report formatting uses `Money`'s `Display`, not hand-rolled arithmetic

---

## Check Your Understanding

- [ ] Split code into modules and files
- [ ] Choose `pub`, `pub(crate)`, `pub(super)` deliberately
- [ ] Protect invariants with private fields and constructors
- [ ] Re-export items to shape a public API
- [ ] Declare and test Cargo features
- [ ] Add a crate to a workspace and depend on it by path
- [ ] Write doc comments with runnable examples

---

## Additional Resources

- [Rust Book Chapter 7](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)
- [Cargo Book: Features](https://doc.rust-lang.org/cargo/reference/features.html)
- [Cargo Book: Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
