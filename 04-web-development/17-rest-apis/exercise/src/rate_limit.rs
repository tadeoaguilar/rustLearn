//! Exercise 6: rate limiting with a token bucket, per client.
//!
//! Each client has a bucket holding up to `capacity` tokens, refilled at
//! `refill_per_sec`. A request takes one token; with none left: 429 Too Many
//! Requests and `Retry-After`. Bursts up to `capacity` are allowed, the
//! long-run rate is `refill_per_sec`.
//!
//! The client is identified by its `x-api-key` header (or "anonymous"). Behind
//! a proxy you'd key on the real client IP instead; and with several server
//! instances the buckets must live in a shared store such as Redis.

use crate::problem::Problem;
use axum::extract::{Request, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    pub capacity: u32,
    pub refill_per_sec: f64,
}

struct Bucket {
    tokens: f64,
    last: Instant,
}

#[derive(Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    buckets: Arc<Mutex<HashMap<String, Bucket>>>,
}

pub enum Decision {
    Allowed { remaining: u32 },
    Limited { retry_after_secs: u64 },
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        todo!("Exercise 6")
    }

    pub fn check(&self, key: &str) -> Decision {
        todo!("Exercise 6")
    }

    /// The clock is a parameter so the logic can be tested without sleeping.
    pub fn check_at(&self, key: &str, now: Instant) -> Decision {
        todo!("Exercise 6")
    }
}

pub async fn rate_limit(
    State(limiter): State<RateLimiter>,
    request: Request,
    next: Next,
) -> Response {
    // TODO Exercise 6. Passes everything through until then, so a todo!()
    // here doesn't break the other exercises.
    next.run(request).await
}
