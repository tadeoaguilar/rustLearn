//! The catalog of books for sale.
//!
//! This file is the parent of `catalog/book.rs` and `catalog/isbn.rs` -- the
//! Rust 2018+ layout, where `catalog.rs` sits next to a `catalog/` directory
//! instead of using `catalog/mod.rs`.

mod book;
mod isbn;

pub use book::{Book, BookError};
pub use isbn::{Isbn, IsbnError};

use std::collections::BTreeMap;

/// All books for sale, keyed by ISBN.
///
/// A `BTreeMap` rather than a `HashMap` so iteration -- and therefore JSON
/// output and reports -- comes out in a stable order.
#[derive(Debug, Default, Clone)]
pub struct Catalog {
    books: BTreeMap<Isbn, Book>,
}

impl Catalog {
    /// An empty catalog.
    pub fn new() -> Self {
        todo!("Exercise 1")
    }

    /// Adds a book, replacing any book with the same ISBN. Returns the
    /// replaced book, if there was one.
    pub fn add(&mut self, book: Book) -> Option<Book> {
        todo!("Exercise 1")
    }

    /// Looks up a book by ISBN.
    pub fn get(&self, isbn: &Isbn) -> Option<&Book> {
        todo!("Exercise 1")
    }

    /// Number of books.
    pub fn len(&self) -> usize {
        todo!("Exercise 1")
    }

    /// True if there are no books.
    pub fn is_empty(&self) -> bool {
        todo!("Exercise 1")
    }

    /// All books, in ISBN order.
    pub fn iter(&self) -> impl Iterator<Item = &Book> {
        // A bare `todo!()` doesn't compile behind `impl Iterator`; replace both lines.
        todo!("Exercise 1");
        std::iter::empty()
    }

    /// Books whose title or author contains `query`, ignoring case.
    pub fn search(&self, query: &str) -> Vec<&Book> {
        todo!("Exercise 1")
    }

    /// Exercise 4: JSON export, only compiled with `--features json`.
    #[cfg(feature = "json")]
    pub fn to_json(&self) -> String {
        todo!("Exercise 1")
    }
}
