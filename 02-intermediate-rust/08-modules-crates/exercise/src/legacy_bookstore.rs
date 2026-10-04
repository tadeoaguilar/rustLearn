// The whole bookstore in one file -- the "before" picture for this module.
//
// It works. It also has every problem the exercises fix:
//   * no modules: catalog, cart, pricing and money are all mixed together
//   * every field is `pub`: anyone can set a negative price, or build an
//     Isbn out of any string -- `is_valid_isbn` exists, but nothing forces
//     anyone to call it
//   * no features: discount code is always compiled in
//   * money handling is stuck here instead of in its own reusable crate
//   * no documentation
//
// Move each piece to its place in the new layout (see lib.rs), then delete
// this file and the `mod legacy_bookstore;` line in lib.rs.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Money {
    pub cents: i64,
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.cents < 0 { "-" } else { "" };
        let abs = self.cents.unsigned_abs();
        write!(f, "{sign}${}.{:02}", abs / 100, abs % 100)
    }
}

pub fn parse_money(s: &str) -> Option<Money> {
    let t = s.trim();
    let (negative, t) = match t.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, t),
    };
    let t = t.strip_prefix('$').unwrap_or(t);
    let (dollars, cents) = t.split_once('.').unwrap_or((t, "0"));
    if dollars.is_empty() || cents.is_empty() || cents.len() > 2 {
        return None;
    }
    let dollars: i64 = dollars.parse().ok()?;
    let cents: i64 = format!("{cents:0<2}").parse().ok()?;
    let total = dollars * 100 + cents;
    Some(Money {
        cents: if negative { -total } else { total },
    })
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Isbn(pub String);

pub fn is_valid_isbn(s: &str) -> bool {
    let digits: String = s.chars().filter(|c| *c != '-' && *c != ' ').collect();
    if digits.len() != 13 || !digits.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let sum: u32 = digits
        .bytes()
        .enumerate()
        .map(|(i, b)| u32::from(b - b'0') * if i % 2 == 0 { 1 } else { 3 })
        .sum();
    sum.is_multiple_of(10)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Book {
    pub isbn: Isbn,
    pub title: String,
    pub author: String,
    pub price: Money,
}

#[derive(Debug, Default)]
pub struct Store {
    pub books: BTreeMap<Isbn, Book>,
    pub cart: Vec<(Isbn, u32)>,
}

impl Store {
    pub fn add_book(&mut self, isbn: &str, title: &str, author: &str, price: Money) {
        let isbn = Isbn(isbn.replace('-', ""));
        self.books.insert(
            isbn.clone(),
            Book {
                isbn,
                title: title.to_string(),
                author: author.to_string(),
                price,
            },
        );
    }

    pub fn search(&self, query: &str) -> Vec<&Book> {
        let q = query.to_lowercase();
        self.books
            .values()
            .filter(|b| {
                format!("{} {}", b.title, b.author)
                    .to_lowercase()
                    .contains(&q)
            })
            .collect()
    }

    pub fn add_to_cart(&mut self, isbn: &str, quantity: u32) {
        let isbn = Isbn(isbn.replace('-', ""));
        if quantity == 0 {
            return;
        }
        for line in self.cart.iter_mut() {
            if line.0 == isbn {
                line.1 += quantity;
                return;
            }
        }
        self.cart.push((isbn, quantity));
    }

    pub fn cart_total(&self) -> Result<Money, String> {
        let mut total = 0;
        for (isbn, qty) in &self.cart {
            let book = self
                .books
                .get(isbn)
                .ok_or(format!("book {} is not in the catalog", isbn.0))?;
            total += book.price.cents * i64::from(*qty);
        }
        Ok(Money { cents: total })
    }

    // 'P' = percent off, 'F' = fixed amount off, anything else = no discount.
    pub fn cart_total_with_discount(&self, kind: char, value: i64) -> Result<Money, String> {
        let subtotal = self.cart_total()?.cents;
        let total = match kind {
            'P' => subtotal - round_half_up(subtotal * value.min(100), 100),
            'F' => (subtotal - value).max(0),
            _ => subtotal,
        };
        Ok(Money { cents: total })
    }
}

pub fn round_half_up(numerator: i64, denominator: i64) -> i64 {
    let half = denominator / 2;
    if numerator >= 0 {
        (numerator + half) / denominator
    } else {
        (numerator - half) / denominator
    }
}

pub fn sample_store() -> Store {
    let mut store = Store::default();
    store.add_book(
        "978-1-4920-5259-3",
        "Programming Rust",
        "Jim Blandy",
        Money { cents: 4999 },
    );
    store.add_book(
        "978-1-7185-0044-0",
        "The Rust Programming Language",
        "Steve Klabnik",
        Money { cents: 3999 },
    );
    store.add_book(
        "978-1-61729-455-6",
        "Rust in Action",
        "Tim McNamara",
        Money { cents: 4499 },
    );
    store
}
