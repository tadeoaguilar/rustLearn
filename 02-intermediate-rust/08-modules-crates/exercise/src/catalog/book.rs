//! A book in the catalog.

use super::Isbn;
use m08_money::Money;
use std::fmt;

/// A book for sale. All fields are private; use [`Book::new`] and the getters.
///
/// Outside this module a struct literal doesn't compile, so nobody can create
/// a book with an empty title or a negative price:
///
/// ```compile_fail
/// use m08_modules_crates::{Book, Isbn, Money};
/// let book = Book {
///     isbn: Isbn::parse("9780306406157").unwrap(),
///     title: String::new(),
///     author: String::new(),
///     price: Money::from_cents(-500),
/// }; // error[E0451]: fields `isbn`, `title`, `author` and `price` of struct `Book` are private
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(serde::Serialize))]
pub struct Book {
    isbn: Isbn,
    title: String,
    author: String,
    price: Money,
}

/// Why a book couldn't be created.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BookError {
    /// The title is empty or only whitespace.
    EmptyTitle,
    /// The price is below zero.
    NegativePrice(Money),
}

impl fmt::Display for BookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Exercise 2")
    }
}

impl std::error::Error for BookError {}

impl Book {
    /// Creates a book, checking the title and price.
    pub fn new(isbn: Isbn, title: &str, author: &str, price: Money) -> Result<Book, BookError> {
        todo!("Exercise 2")
    }

    /// The ISBN.
    pub fn isbn(&self) -> &Isbn {
        todo!("Exercise 2")
    }

    /// The title.
    pub fn title(&self) -> &str {
        todo!("Exercise 2")
    }

    /// The author.
    pub fn author(&self) -> &str {
        todo!("Exercise 2")
    }

    /// The list price.
    pub fn price(&self) -> Money {
        todo!("Exercise 2")
    }

    /// Lower-cased "title author", used by `Catalog::search`.
    ///
    /// `pub(super)`: visible in the parent module (`catalog`), where the
    /// search lives, but not to anyone using the crate.
    pub(super) fn search_key(&self) -> String {
        todo!("Exercise 2")
    }
}
