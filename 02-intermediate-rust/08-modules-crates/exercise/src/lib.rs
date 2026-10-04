//! Module 08 -- Modules & Crates. YOUR WORKSPACE: the bookstore library.
//!
//! `legacy_bookstore.rs` holds the whole bookstore in one file. It works, but
//! everything is public, nothing is validated and nothing is documented.
//!
//! The target layout already exists -- catalog.rs, catalog/book.rs,
//! catalog/isbn.rs, cart.rs, pricing.rs, prelude.rs, report.rs, rounding.rs
//! and crates/money -- with every signature in place and a `todo!()` body.
//! Move the code from legacy_bookstore.rs into it (Exercise 1), then tighten
//! visibility (2), shape the API (3), add features (4), finish the money
//! crate (5) and document everything (6).
//!
//!     cargo run  -p m08-modules-crates -- all
//!     cargo test -p m08-modules-crates-tests --features mine
//!
//! Stuck? Compare with ../solution -- same file and function names.

// Remove once you have started: silences "unused" warnings while todo!().
#![allow(unused)]
// Exercise 6: uncomment, then document everything it complains about.
// #![warn(missing_docs)]

// The old code. Delete this line and the file when you are done.
mod legacy_bookstore;

pub mod cart;
pub mod catalog;
pub mod prelude;
#[cfg(feature = "discounts")]
pub mod pricing;
#[cfg(feature = "report")]
pub mod report;

#[cfg(feature = "discounts")]
mod rounding;

// Exercise 3: re-exports at the crate root.
pub use cart::{Cart, CartError};
pub use catalog::{Book, BookError, Catalog, Isbn, IsbnError};
// Exercise 5: re-export the money crate's types.
pub use m08_money::{Money, ParseMoneyError};
#[cfg(feature = "discounts")]
pub use pricing::Discount;

/// Sample data used by the demos and the tests: these three books, with
/// these prices --
///
/// | ISBN              | Title                         | Author        | Price  |
/// |-------------------|-------------------------------|---------------|--------|
/// | 978-1-4920-5259-3 | Programming Rust              | Jim Blandy    | $49.99 |
/// | 978-1-7185-0044-0 | The Rust Programming Language | Steve Klabnik | $39.99 |
/// | 978-1-61729-455-6 | Rust in Action                | Tim McNamara  | $44.99 |
pub fn sample_catalog() -> Catalog {
    todo!("Exercise 1: legacy_bookstore::sample_store() shows how")
}
