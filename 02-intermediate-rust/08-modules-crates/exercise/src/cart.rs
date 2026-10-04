//! A shopping cart.

use crate::catalog::{Catalog, Isbn};
use m08_money::Money;
use std::fmt;

/// One line of a cart: a book and how many copies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CartLine {
    /// Which book.
    pub isbn: Isbn,
    /// How many copies (always at least 1).
    pub quantity: u32,
}

/// Why a cart couldn't be totalled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CartError {
    /// The cart refers to a book that isn't in the catalog.
    UnknownBook(Isbn),
}

impl fmt::Display for CartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Exercise 1")
    }
}

impl std::error::Error for CartError {}

/// A shopping cart. Holds ISBNs and quantities only -- prices come from the
/// catalog at checkout, so a price change is picked up automatically.
#[derive(Debug, Default, Clone)]
pub struct Cart {
    lines: Vec<CartLine>,
}

impl Cart {
    /// An empty cart.
    pub fn new() -> Self {
        todo!("Exercise 1")
    }

    /// Adds copies of a book, merging with an existing line. Adding 0 copies
    /// does nothing.
    pub fn add(&mut self, isbn: Isbn, quantity: u32) {
        todo!("Exercise 1")
    }

    /// Removes a book entirely. Returns how many copies were removed.
    pub fn remove(&mut self, isbn: &Isbn) -> u32 {
        todo!("Exercise 1")
    }

    /// The lines, in the order books were first added.
    pub fn lines(&self) -> &[CartLine] {
        todo!("Exercise 1")
    }

    /// Total number of copies.
    pub fn item_count(&self) -> u32 {
        todo!("Exercise 1")
    }

    /// Sum of price × quantity for every line.
    ///
    /// ```
    /// use m08_modules_crates::{sample_catalog, Cart, Isbn};
    /// let catalog = sample_catalog();
    /// let mut cart = Cart::new();
    /// cart.add(Isbn::parse("978-1-4920-5259-3")?, 2);
    /// assert_eq!(cart.total(&catalog)?.to_string(), "$99.98");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn total(&self, catalog: &Catalog) -> Result<Money, CartError> {
        todo!("Exercise 1")
    }

    /// The total after applying a discount. Only with feature `discounts`.
    #[cfg(feature = "discounts")]
    pub fn total_with_discount(
        &self,
        catalog: &Catalog,
        discount: &crate::pricing::Discount,
    ) -> Result<Money, CartError> {
        todo!("Exercise 1")
    }
}
