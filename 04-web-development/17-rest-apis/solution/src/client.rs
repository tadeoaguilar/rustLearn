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
        TasksClient {
            base: base.trim_end_matches('/').to_string(),
            http: reqwest::Client::new(),
            api_key: None,
        }
    }

    pub fn with_api_key(mut self, key: &str) -> Self {
        self.api_key = Some(key.to_string());
        self
    }

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let req = self.http.request(method, format!("{}{path}", self.base));
        match &self.api_key {
            Some(k) => req.header("x-api-key", k),
            None => req,
        }
    }

    /// Turns any non-success response into a typed error.
    async fn check(response: reqwest::Response) -> Result<reqwest::Response, ClientError> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        if status == StatusCode::TOO_MANY_REQUESTS {
            let retry_after_secs = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok())
                .unwrap_or(1);
            return Err(ClientError::RateLimited { retry_after_secs });
        }
        let problem = response.json::<Problem>().await?;
        Err(ClientError::Api(problem))
    }

    pub async fn create(&self, input: &TaskInput) -> Result<Task, ClientError> {
        let r = self
            .request(reqwest::Method::POST, "/v1/tasks")
            .json(input)
            .send()
            .await?;
        Ok(Self::check(r).await?.json().await?)
    }

    pub async fn get(&self, id: u64) -> Result<Task, ClientError> {
        let r = self
            .request(reqwest::Method::GET, &format!("/v1/tasks/{id}"))
            .send()
            .await?;
        Ok(Self::check(r).await?.json().await?)
    }

    pub async fn replace(&self, id: u64, input: &TaskInput) -> Result<Task, ClientError> {
        let r = self
            .request(reqwest::Method::PUT, &format!("/v1/tasks/{id}"))
            .json(input)
            .send()
            .await?;
        Ok(Self::check(r).await?.json().await?)
    }

    pub async fn delete(&self, id: u64) -> Result<(), ClientError> {
        let r = self
            .request(reqwest::Method::DELETE, &format!("/v1/tasks/{id}"))
            .send()
            .await?;
        Self::check(r).await?;
        Ok(())
    }

    /// One page; `query` is the raw query string, e.g. "page=2&per_page=10".
    /// Also returns the URL path of the next page, from the `Link` header.
    pub async fn list_page(
        &self,
        path_and_query: &str,
    ) -> Result<(Page<Task>, Option<String>), ClientError> {
        let r = self
            .request(reqwest::Method::GET, path_and_query)
            .send()
            .await?;
        let r = Self::check(r).await?;
        let next = r
            .headers()
            .get("link")
            .and_then(|v| v.to_str().ok())
            .and_then(next_link);
        Ok((r.json().await?, next))
    }

    /// Every task, following `rel="next"` links until there are none.
    pub async fn list_all(&self, per_page: u32) -> Result<Vec<Task>, ClientError> {
        let mut all = Vec::new();
        let mut next = Some(format!("/v1/tasks?page=1&per_page={per_page}"));
        while let Some(path) = next {
            let (page, following) = self.list_page(&path).await?;
            all.extend(page.data);
            next = following;
        }
        Ok(all)
    }
}

/// Extracts the URL of `rel="next"` from a Link header.
pub fn next_link(header: &str) -> Option<String> {
    header.split(',').find_map(|part| {
        let (url, params) = part.split_once(';')?;
        params.contains("rel=\"next\"").then(|| {
            url.trim()
                .trim_start_matches('<')
                .trim_end_matches('>')
                .to_string()
        })
    })
}
