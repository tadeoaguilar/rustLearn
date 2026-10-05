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
#[allow(unreachable_code)]
pub fn json_subscriber<W>(writer: W, directives: &str) -> impl Subscriber + Send + Sync
where
    W: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    todo!("Exercise 1");
    tracing_subscriber::registry() // placeholder so the signature compiles: replace it
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
// TODO (Exercise 1): add #[instrument(...)] with a span named "checkout" and the
// fields order_id, customer (masked) and items.
pub fn checkout(order: &Order) -> Result<u64, CheckoutError> {
    todo!("Exercise 1")
}

/// Exercise 2: wraps a value so it can't leak into logs by accident --
/// `Debug` and `Display` print `***`.
pub struct Redacted<T>(pub T);

impl<T> fmt::Debug for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Exercise 1")
    }
}

impl<T> fmt::Display for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Exercise 1")
    }
}

/// "alice@example.com" -> "a***@example.com": enough to correlate, not
/// enough to identify. Not an email: fully masked.
pub fn mask_email(email: &str) -> String {
    todo!("Exercise 1")
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
