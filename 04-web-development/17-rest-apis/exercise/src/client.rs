//! Exercise 8: a typed client SDK.
//!
//! Callers get `Task`s and `ClientError`s, never status codes or JSON. The
//! client knows the API's conventions -- problem+json errors, the `Link`
//! header, `Retry-After` -- so its users don't have to.

use crate::model::{Task, TaskInput};
use crate::pagination::Page;
use crate::problem::Problem;
use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("API error {}: {}", .0.status, .0.title)]
    Api(Problem),
    #[error("rate limited; retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

#[derive(Debug, Clone)]
pub struct TasksClient {
    base: String,
    http: reqwest::Client,
    api_key: Option<String>,
}

impl TasksClient {
    pub fn new(base: &str) -> Self {
        todo!("Exercise 8")
    }

    pub fn with_api_key(mut self, key: &str) -> Self {
        todo!("Exercise 8")
    }

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        todo!("Exercise 8")
    }

    /// Turns any non-success response into a typed error.
    async fn check(response: reqwest::Response) -> Result<reqwest::Response, ClientError> {
        todo!("Exercise 8")
    }

    pub async fn create(&self, input: &TaskInput) -> Result<Task, ClientError> {
        todo!("Exercise 8")
    }

    pub async fn get(&self, id: u64) -> Result<Task, ClientError> {
        todo!("Exercise 8")
    }

    pub async fn replace(&self, id: u64, input: &TaskInput) -> Result<Task, ClientError> {
        todo!("Exercise 8")
    }

    pub async fn delete(&self, id: u64) -> Result<(), ClientError> {
        todo!("Exercise 8")
    }

    /// One page; `query` is the raw query string, e.g. "page=2&per_page=10".
    /// Also returns the URL path of the next page, from the `Link` header.
    pub async fn list_page(
        &self,
        path_and_query: &str,
    ) -> Result<(Page<Task>, Option<String>), ClientError> {
        todo!("Exercise 8")
    }

    /// Every task, following `rel="next"` links until there are none.
    pub async fn list_all(&self, per_page: u32) -> Result<Vec<Task>, ClientError> {
        todo!("Exercise 8")
    }
}

/// Extracts the URL of `rel="next"` from a Link header.
pub fn next_link(header: &str) -> Option<String> {
    todo!("Exercise 8")
}
