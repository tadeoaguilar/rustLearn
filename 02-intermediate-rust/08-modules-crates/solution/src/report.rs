//! Bonus: inventory report. Compiled only with feature `report`.

use crate::catalog::Catalog;
use m08_money_solution::Money;
use std::collections::BTreeSet;
use std::fmt;

/// A summary of the catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// Number of books.
    pub books: usize,
    /// Number of distinct authors.
    pub authors: usize,
    /// Sum of list prices.
    pub total_value: Money,
    /// Title and price of the most expensive book, if any.
    pub most_expensive: Option<(String, Money)>,
}

impl Catalog {
    /// Builds an inventory report.
    ///
    /// An `impl Catalog` block in a *different module* from the type: allowed
    /// anywhere in the same crate.
    pub fn report(&self) -> Report {
        let authors: BTreeSet<&str> = self.iter().map(|b| b.author()).collect();
        Report {
            books: self.len(),
            authors: authors.len(),
            total_value: self.iter().map(|b| b.price()).sum(),
            most_expensive: self
                .iter()
                .max_by_key(|b| b.price())
                .map(|b| (b.title().to_string(), b.price())),
        }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} books by {} authors, total list value {}",
            self.books, self.authors, self.total_value
        )?;
        match &self.most_expensive {
            Some((title, price)) => write!(f, "Most expensive: {title} ({price})"),
            None => write!(f, "The catalog is empty"),
        }
    }
}
