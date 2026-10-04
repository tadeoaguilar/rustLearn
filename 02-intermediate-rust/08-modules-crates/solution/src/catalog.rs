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
        Self::default()
    }

    /// Adds a book, replacing any book with the same ISBN. Returns the
    /// replaced book, if there was one.
    pub fn add(&mut self, book: Book) -> Option<Book> {
        self.books.insert(book.isbn().clone(), book)
    }

    /// Looks up a book by ISBN.
    pub fn get(&self, isbn: &Isbn) -> Option<&Book> {
        self.books.get(isbn)
    }

    /// Number of books.
    pub fn len(&self) -> usize {
        self.books.len()
    }

    /// True if there are no books.
    pub fn is_empty(&self) -> bool {
        self.books.is_empty()
    }

    /// All books, in ISBN order.
    pub fn iter(&self) -> impl Iterator<Item = &Book> {
        self.books.values()
    }

    /// Books whose title or author contains `query`, ignoring case.
    pub fn search(&self, query: &str) -> Vec<&Book> {
        let query = query.to_lowercase();
        // search_key is pub(super) in book.rs: visible here, not to users.
        self.books
            .values()
            .filter(|b| b.search_key().contains(&query))
            .collect()
    }

    /// Exercise 4: JSON export, only compiled with `--features json`.
    #[cfg(feature = "json")]
    pub fn to_json(&self) -> String {
        let books: Vec<&Book> = self.iter().collect();
        serde_json::to_string_pretty(&books).expect("books always serialise")
    }
}
