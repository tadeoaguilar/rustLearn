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
    reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .expect("valid client config")
}

/// `error_for_status` turns 4xx/5xx into an Err; without it, a 404 page would
/// reach `.json()` and fail with a confusing decode error instead.
pub async fn fetch_post(
    client: &reqwest::Client,
    base: &str,
    id: u32,
) -> Result<Post, reqwest::Error> {
    client
        .get(format!("{base}/posts/{id}"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}

/// Task 1: many URLs at once. All requests are in flight together.
pub async fn fetch_many(
    client: &reqwest::Client,
    base: &str,
    ids: &[u32],
) -> Vec<Result<Post, reqwest::Error>> {
    join_all(ids.iter().map(|&id| fetch_post(client, base, id))).await
}

/// Task 2: retry with exponential backoff, only for errors worth retrying:
/// timeouts, connection failures and 5xx. A 404 won't fix itself.
pub async fn get_text_with_retry(
    client: &reqwest::Client,
    url: &str,
    max_attempts: u32,
    base_delay: Duration,
) -> Result<(String, u32), reqwest::Error> {
    let mut attempt = 1;
    loop {
        let result = async {
            client
                .get(url)
                .send()
                .await?
                .error_for_status()?
                .text()
                .await
        }
        .await;
        match result {
            Ok(text) => return Ok((text, attempt)),
            Err(e) if attempt < max_attempts && is_retryable(&e) => {
                tokio::time::sleep(base_delay * 2u32.pow(attempt - 1)).await;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

fn is_retryable(e: &reqwest::Error) -> bool {
    e.is_timeout() || e.is_connect() || e.status().is_some_and(|s| s.is_server_error())
}

pub async fn run() {
    let base = crate::mock_api::spawn().await;
    let http = client(Duration::from_millis(500));

    println!("{:#?}", fetch_post(&http, &base, 1).await);
    for r in fetch_many(&http, &base, &[2, 3, 0]).await {
        println!(
            "fetch_many -> {:?}",
            r.map(|p| p.title).map_err(|e| e.status())
        );
    }
    let flaky = format!("{base}/flaky");
    println!(
        "flaky with retry: {:?}",
        get_text_with_retry(&http, &flaky, 5, Duration::from_millis(10)).await
    );
    let slow = format!("{base}/slow");
    match get_text_with_retry(&http, &slow, 2, Duration::from_millis(10)).await {
        Ok(r) => println!("slow: {r:?}"),
        Err(e) => println!("slow: gave up -- timeout? {}", e.is_timeout()),
    }
    println!("(against the real service: base = \"https://jsonplaceholder.typicode.com\")");
}
