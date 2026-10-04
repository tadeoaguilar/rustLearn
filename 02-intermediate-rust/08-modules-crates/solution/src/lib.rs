//! # Bookstore
//!
//! Reference solution for module 08 (Modules & Crates): a small bookstore
//! library -- a catalog of books, a shopping cart, discounts, and money
//! handling in a separate crate.
//!
//! ```
//! use m08_modules_crates_solution::prelude::*;
//!
//! let mut catalog = Catalog::new();
//! let isbn = Isbn::parse("978-1-7185-0044-0")?;
//! catalog.add(Book::new(isbn.clone(), "The Rust Programming Language", "Steve Klabnik", Money::new(39, 99))?);
//!
//! let mut cart = Cart::new();
//! cart.add(isbn, 2);
//! assert_eq!(cart.total(&catalog)?.to_string(), "$79.98");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Features
//!
//! | Feature     | Default | Adds                                          |
//! |-------------|---------|-----------------------------------------------|
//! | `discounts` | yes     | [`pricing::Discount`], `Cart::total_with_discount` |
//! | `json`      | no      | `Serialize` for books, `Catalog::to_json`     |
//! | `report`    | no      | `Catalog::report` (implies `discounts`)       |

// Exercise 6: every public item must be documented.
#![warn(missing_docs)]

// Exercise 1: lib.rs only declares the module tree.
pub mod cart;
pub mod catalog;
pub mod prelude;
#[cfg(feature = "discounts")]
pub mod pricing;
#[cfg(feature = "report")]
pub mod report;

// Not `pub`: a private module is an implementation detail of the crate.
// Only `pricing` uses it, so it is compiled only when `pricing` is.
#[cfg(feature = "discounts")]
mod rounding;

// Exercise 3: re-exports at the root, so `bookstore::Book` works without
// knowing the module layout -- which we can then change freely.
pub use cart::{Cart, CartError};
pub use catalog::{Book, BookError, Catalog, Isbn, IsbnError};
// Exercise 5: re-export the money crate's type so users need one dependency.
pub use m08_money_solution::{Money, ParseMoneyError};
#[cfg(feature = "discounts")]
pub use pricing::Discount;

/// Sample data used by the demos and the tests.
pub fn sample_catalog() -> Catalog {
    let mut catalog = Catalog::new();
    for (isbn, title, author, price) in [
        (
            "978-1-4920-5259-3",
            "Programming Rust",
            "Jim Blandy",
            Money::new(49, 99),
        ),
        (
            "978-1-7185-0044-0",
            "The Rust Programming Language",
            "Steve Klabnik",
            Money::new(39, 99),
        ),
        (
            "978-1-61729-455-6",
            "Rust in Action",
            "Tim McNamara",
            Money::new(44, 99),
        ),
    ] {
        let isbn = Isbn::parse(isbn).expect("sample ISBNs are valid");
        catalog.add(Book::new(isbn, title, author, price).expect("sample books are valid"));
    }
    catalog
}
