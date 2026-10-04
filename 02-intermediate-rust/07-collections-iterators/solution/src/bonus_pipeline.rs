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
        Sale {
            product: product.to_string(),
            amount,
            quantity,
        }
    }

    pub fn revenue(&self) -> f64 {
        self.amount * self.quantity as f64
    }
}

pub fn sample_sales() -> Vec<Sale> {
    vec![
        Sale::new("Widget", 10.0, 5),
        Sale::new("Gadget", 15.0, 3),
        Sale::new("Widget", 10.0, 2),
        Sale::new("Gizmo", 120.0, 1),
        Sale::new("Doohickey", 2.5, 12),
        Sale::new("Gadget", 14.0, 9),
    ]
}

/// From the exercise. The fold version works; the `entry` loop below is the
/// more common spelling.
pub fn revenue_by_product(sales: &[Sale]) -> HashMap<String, f64> {
    let mut totals = HashMap::new();
    for sale in sales {
        *totals.entry(sale.product.clone()).or_insert(0.0) += sale.revenue();
    }
    totals
}

/// Top n by revenue. Two fixes over the exercise's version:
/// * `f64::total_cmp` instead of `partial_cmp(..).unwrap()`, which panics if
///   any revenue is NaN.
/// * ties broken by product name, so the order doesn't depend on HashMap's
///   random iteration order.
pub fn top_products(sales: &[Sale], n: usize) -> Vec<(String, f64)> {
    let mut products: Vec<(String, f64)> = revenue_by_product(sales).into_iter().collect();
    products.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    products.truncate(n);
    products
}

/// Task 1: sales whose revenue is above a threshold. Returns references --
/// no need to clone the sales just to look at them.
pub fn sales_above(sales: &[Sale], threshold: f64) -> Vec<&Sale> {
    sales.iter().filter(|s| s.revenue() > threshold).collect()
}

/// Task 2: average unit price per product, weighted by quantity
/// (total revenue / total units). BTreeMap for sorted, testable output.
pub fn average_price_per_product(sales: &[Sale]) -> BTreeMap<String, f64> {
    let mut acc: BTreeMap<String, (f64, i32)> = BTreeMap::new();
    for s in sales {
        let e = acc.entry(s.product.clone()).or_default();
        e.0 += s.revenue();
        e.1 += s.quantity;
    }
    acc.into_iter()
        .filter(|(_, (_, qty))| *qty > 0)
        .map(|(product, (revenue, qty))| (product, revenue / qty as f64))
        .collect()
}

/// Task 3: products whose total quantity exceeds `min`, sorted by name.
pub fn products_with_quantity_over(sales: &[Sale], min: i32) -> Vec<String> {
    let mut qty: BTreeMap<&str, i32> = BTreeMap::new();
    for s in sales {
        *qty.entry(&s.product).or_default() += s.quantity;
    }
    qty.into_iter()
        .filter(|(_, q)| *q > min)
        .map(|(p, _)| p.to_string())
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PriceRange {
    Budget,  // < 5
    Mid,     // 5 ..< 50
    Premium, // >= 50
}

impl PriceRange {
    pub fn of(amount: f64) -> Self {
        match amount {
            a if a < 5.0 => PriceRange::Budget,
            a if a < 50.0 => PriceRange::Mid,
            _ => PriceRange::Premium,
        }
    }
}

/// Task 4: group sales by unit price range.
pub fn group_by_price_range(sales: &[Sale]) -> BTreeMap<PriceRange, Vec<&Sale>> {
    let mut groups: BTreeMap<PriceRange, Vec<&Sale>> = BTreeMap::new();
    for s in sales {
        groups.entry(PriceRange::of(s.amount)).or_default().push(s);
    }
    groups
}

pub fn run() {
    let sales = sample_sales();
    let mut revenue: Vec<_> = revenue_by_product(&sales).into_iter().collect();
    revenue.sort_by(|a, b| a.0.cmp(&b.0));
    println!("revenue by product: {revenue:?}");
    println!("top 3: {:?}", top_products(&sales, 3));
    let big: Vec<&str> = sales_above(&sales, 60.0)
        .iter()
        .map(|s| s.product.as_str())
        .collect();
    println!("sales above 60: {big:?}");
    println!("average price: {:?}", average_price_per_product(&sales));
    println!(
        "quantity > 10: {:?}",
        products_with_quantity_over(&sales, 10)
    );
    for (range, group) in group_by_price_range(&sales) {
        let names: Vec<&str> = group.iter().map(|s| s.product.as_str()).collect();
        println!("{range:?}: {names:?}");
    }
}
