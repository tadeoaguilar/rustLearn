//! Bonus Challenge: Data Pipeline.

use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, PartialEq)]
pub struct Sale {
    pub product: String,
    pub amount: f64,
    pub quantity: i32,
}

impl Sale {
    pub fn new(product: &str, amount: f64, quantity: i32) -> Self {
        todo!("Bonus")
    }

    pub fn revenue(&self) -> f64 {
        todo!("Bonus")
    }
}

pub fn sample_sales() -> Vec<Sale> {
    todo!("Bonus")
}

/// From the exercise. The fold version works; the `entry` loop below is the
/// more common spelling.
pub fn revenue_by_product(sales: &[Sale]) -> HashMap<String, f64> {
    todo!("Bonus")
}

/// Top n by revenue. Two fixes over the exercise's version:
/// * `f64::total_cmp` instead of `partial_cmp(..).unwrap()`, which panics if
///   any revenue is NaN.
/// * ties broken by product name, so the order doesn't depend on HashMap's
///   random iteration order.
pub fn top_products(sales: &[Sale], n: usize) -> Vec<(String, f64)> {
    todo!("Bonus")
}

/// Task 1: sales whose revenue is above a threshold. Returns references --
/// no need to clone the sales just to look at them.
pub fn sales_above(sales: &[Sale], threshold: f64) -> Vec<&Sale> {
    todo!("Bonus")
}

/// Task 2: average unit price per product, weighted by quantity
/// (total revenue / total units). BTreeMap for sorted, testable output.
pub fn average_price_per_product(sales: &[Sale]) -> BTreeMap<String, f64> {
    todo!("Bonus")
}

/// Task 3: products whose total quantity exceeds `min`, sorted by name.
pub fn products_with_quantity_over(sales: &[Sale], min: i32) -> Vec<String> {
    todo!("Bonus")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PriceRange {
    Budget,  // < 5
    Mid,     // 5 ..< 50
    Premium, // >= 50
}

impl PriceRange {
    pub fn of(amount: f64) -> Self {
        todo!("Bonus")
    }
}

/// Task 4: group sales by unit price range.
pub fn group_by_price_range(sales: &[Sale]) -> BTreeMap<PriceRange, Vec<&Sale>> {
    todo!("Bonus")
}

pub fn run() {
    todo!("Bonus")
}
