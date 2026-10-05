//! Exercises 1 and 2: structured logging.
//!
//! `println!("order {id} failed")` is a string a human can read and a
//! machine can't. Structured logs are *events with fields*: the log
//! pipeline (Loki, Elasticsearch, CloudWatch) can then filter on
//! `order_id = "o-42"` or count `level = "ERROR"` by `route`. In Rust that's
//! `tracing`: events (`info!`) carry fields, and **spans** (`#[instrument]`)
//! give every event inside them context -- which request, which order.

use std::fmt;
use tracing::{Subscriber, info, instrument, warn};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::MakeWriter;

/// A subscriber writing one JSON object per line, filtered by `directives`
/// (the `RUST_LOG` syntax: `"info,my_crate::db=debug"`).
pub fn json_subscriber<W>(writer: W, directives: &str) -> impl Subscriber + Send + Sync
where
    W: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    tracing_subscriber::fmt()
        .json()
        .with_writer(writer)
        .with_env_filter(EnvFilter::new(directives))
        .flatten_event(true) // fields at the top level: {"level":..,"message":..,"order_id":..}
        .with_current_span(true) // plus the enclosing span and its fields
        .with_span_list(false)
        .with_target(true)
        .finish()
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub email: String,
    pub items: Vec<(String, u32, u64)>, // sku, quantity, unit price in cents
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckoutError {
    Empty,
}

/// Every event inside gets the span's fields. Fields are recorded with `%`
/// (Display) or `?` (Debug); the email is masked before it's recorded.
#[instrument(name = "checkout", skip(order), fields(order_id = %order.id, customer = %mask_email(&order.email), items = order.items.len()))]
pub fn checkout(order: &Order) -> Result<u64, CheckoutError> {
    if order.items.is_empty() {
        warn!(reason = "empty", "order rejected");
        return Err(CheckoutError::Empty);
    }
    let total: u64 = order
        .items
        .iter()
        .map(|(_, qty, price)| u64::from(*qty) * price)
        .sum();
    info!(total_cents = total, "order accepted");
    Ok(total)
}

/// Exercise 2: wraps a value so it can't leak into logs by accident --
/// `Debug` and `Display` print `***`.
pub struct Redacted<T>(pub T);

impl<T> fmt::Debug for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

impl<T> fmt::Display for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

/// "alice@example.com" -> "a***@example.com": enough to correlate, not
/// enough to identify. Not an email: fully masked.
pub fn mask_email(email: &str) -> String {
    match email.split_once('@') {
        Some((user, domain)) if !user.is_empty() && !domain.is_empty() => {
            let first = user.chars().next().unwrap();
            format!("{first}***@{domain}")
        }
        _ => "***".to_string(),
    }
}

/// A chatty module, for practising per-module filters.
pub mod noisy {
    /// This module's tracing target -- what a filter directive names.
    pub const TARGET: &str = module_path!();

    pub fn chatter() {
        tracing::debug!(cache_hits = 1042, "cache statistics");
        tracing::warn!("cache almost full");
    }
}
