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
        RateLimiter {
            config,
            buckets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn check(&self, key: &str) -> Decision {
        self.check_at(key, Instant::now())
    }

    /// The clock is a parameter so the logic can be tested without sleeping.
    pub fn check_at(&self, key: &str, now: Instant) -> Decision {
        let RateLimitConfig {
            capacity,
            refill_per_sec,
        } = self.config;
        let mut buckets = self.buckets.lock().unwrap();
        let bucket = buckets.entry(key.to_string()).or_insert(Bucket {
            tokens: capacity as f64,
            last: now,
        });
        let elapsed = now.saturating_duration_since(bucket.last).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * refill_per_sec).min(capacity as f64);
        bucket.last = now;
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            Decision::Allowed {
                remaining: bucket.tokens.floor() as u32,
            }
        } else {
            let wait = (1.0 - bucket.tokens) / refill_per_sec;
            Decision::Limited {
                retry_after_secs: wait.ceil().max(1.0) as u64,
            }
        }
    }
}

pub async fn rate_limit(
    State(limiter): State<RateLimiter>,
    request: Request,
    next: Next,
) -> Response {
    let key = request
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anonymous")
        .to_string();
    let limit = HeaderValue::from(limiter.config.capacity);
    match limiter.check(&key) {
        Decision::Allowed { remaining } => {
            let mut response = next.run(request).await;
            response.headers_mut().insert("x-ratelimit-limit", limit);
            response
                .headers_mut()
                .insert("x-ratelimit-remaining", HeaderValue::from(remaining));
            response
        }
        Decision::Limited { retry_after_secs } => {
            let problem = Problem::new(
                StatusCode::TOO_MANY_REQUESTS,
                "rate-limited",
                "Too many requests",
            )
            .with_detail(format!("try again in {retry_after_secs}s"));
            let mut response = problem.into_response();
            response
                .headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from(retry_after_secs));
            response.headers_mut().insert("x-ratelimit-limit", limit);
            response
                .headers_mut()
                .insert("x-ratelimit-remaining", HeaderValue::from(0));
            response
        }
    }
}
