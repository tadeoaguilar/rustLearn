//! Bonus: inventory report. Compiled only with feature `report`.

use crate::catalog::Catalog;
use m08_money::Money;
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
        todo!("Bonus")
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Bonus")
    }
}
