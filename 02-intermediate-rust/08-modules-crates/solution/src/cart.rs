//! A shopping cart.

use crate::catalog::{Catalog, Isbn};
use m08_money_solution::Money;
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
        match self {
            CartError::UnknownBook(isbn) => write!(f, "book {isbn} is not in the catalog"),
        }
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
        Self::default()
    }

    /// Adds copies of a book, merging with an existing line. Adding 0 copies
    /// does nothing.
    pub fn add(&mut self, isbn: Isbn, quantity: u32) {
        if quantity == 0 {
            return;
        }
        match self.lines.iter_mut().find(|l| l.isbn == isbn) {
            Some(line) => line.quantity += quantity,
            None => self.lines.push(CartLine { isbn, quantity }),
        }
    }

    /// Removes a book entirely. Returns how many copies were removed.
    pub fn remove(&mut self, isbn: &Isbn) -> u32 {
        let before: u32 = self.item_count();
        self.lines.retain(|l| &l.isbn != isbn);
        before - self.item_count()
    }

    /// The lines, in the order books were first added.
    pub fn lines(&self) -> &[CartLine] {
        &self.lines
    }

    /// Total number of copies.
    pub fn item_count(&self) -> u32 {
        self.lines.iter().map(|l| l.quantity).sum()
    }

    /// Sum of price × quantity for every line.
    ///
    /// ```
    /// use m08_modules_crates_solution::{sample_catalog, Cart, Isbn};
    /// let catalog = sample_catalog();
    /// let mut cart = Cart::new();
    /// cart.add(Isbn::parse("978-1-4920-5259-3")?, 2);
    /// assert_eq!(cart.total(&catalog)?.to_string(), "$99.98");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn total(&self, catalog: &Catalog) -> Result<Money, CartError> {
        self.lines
            .iter()
            .map(|line| {
                let book = catalog
                    .get(&line.isbn)
                    .ok_or_else(|| CartError::UnknownBook(line.isbn.clone()))?;
                Ok(book.price() * line.quantity)
            })
            .sum()
    }

    /// The total after applying a discount. Only with feature `discounts`.
    #[cfg(feature = "discounts")]
    pub fn total_with_discount(
        &self,
        catalog: &Catalog,
        discount: &crate::pricing::Discount,
    ) -> Result<Money, CartError> {
        let subtotal = self.total(catalog)?;
        Ok(discount.apply(subtotal, self.item_count()))
    }
}
