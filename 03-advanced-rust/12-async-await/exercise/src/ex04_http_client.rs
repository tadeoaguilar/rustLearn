//! Exercise 4: Async HTTP Client.
//!
//! Everything here takes a `base` URL, so it works against the in-process
//! `mock_api` (tests, demos) or a real service.

use futures::future::join_all;
use serde::Deserialize;
use std::time::Duration;

/// `#[serde(rename_all = "camelCase")]` maps JSON's `userId` to Rust's
/// snake_case `user_id`. The exercise's `userId` field works too, but trips the
/// `non_snake_case` lint.
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    pub user_id: i32,
    pub id: i32,
    pub title: String,
    pub body: String,
}

/// Task 3: build one client and reuse it -- it pools connections. The timeout
/// covers the whole request, including reading the body.
pub fn client(timeout: Duration) -> reqwest::Client {
    todo!("Exercise 4")
}

/// `error_for_status` turns 4xx/5xx into an Err; without it, a 404 page would
/// reach `.json()` and fail with a confusing decode error instead.
pub async fn fetch_post(
    client: &reqwest::Client,
    base: &str,
    id: u32,
) -> Result<Post, reqwest::Error> {
    todo!("Exercise 4")
}

/// Task 1: many URLs at once. All requests are in flight together.
pub async fn fetch_many(
    client: &reqwest::Client,
    base: &str,
    ids: &[u32],
) -> Vec<Result<Post, reqwest::Error>> {
    todo!("Exercise 4")
}

/// Task 2: retry with exponential backoff, only for errors worth retrying:
/// timeouts, connection failures and 5xx. A 404 won't fix itself.
pub async fn get_text_with_retry(
    client: &reqwest::Client,
    url: &str,
    max_attempts: u32,
    base_delay: Duration,
) -> Result<(String, u32), reqwest::Error> {
    todo!("Exercise 4")
}

fn is_retryable(e: &reqwest::Error) -> bool {
    todo!("Exercise 4")
}

pub async fn run() {
    todo!("Exercise 4")
}
