//! Exercise 2: one error format for the whole API -- RFC 7807 / RFC 9457
//! "Problem Details", served as `application/problem+json`.
//!
//! ```json
//! { "type": "https://example.com/problems/validation", "title": "Validation failed",
//!   "status": 422, "detail": "The request body has invalid fields",
//!   "errors": { "priority": ["must be between 1 and 5"] } }
//! ```
//!
//! Clients can branch on `type` (stable) and show `detail` (human-readable).

use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Problem {
    #[serde(rename = "type")]
    pub kind: String,
    pub title: String,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Field name -> messages, for validation problems.
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub errors: BTreeMap<String, Vec<String>>,
}

pub const PROBLEM_JSON: &str = "application/problem+json";

impl Problem {
    pub fn new(status: StatusCode, kind: &str, title: &str) -> Self {
        Problem {
            kind: format!("https://rustlearn.dev/problems/{kind}"),
            title: title.to_string(),
            status: status.as_u16(),
            detail: None,
            errors: BTreeMap::new(),
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn not_found(what: &str) -> Self {
        Problem::new(StatusCode::NOT_FOUND, "not-found", "Resource not found")
            .with_detail(format!("{what} does not exist"))
    }

    pub fn bad_request(detail: impl Into<String>) -> Self {
        Problem::new(StatusCode::BAD_REQUEST, "bad-request", "Bad request").with_detail(detail)
    }
}

impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let mut response = (status, axum::Json(&self)).into_response();
        // axum::Json sets application/json; problem details have their own type.
        response
            .headers_mut()
            .insert(header::CONTENT_TYPE, HeaderValue::from_static(PROBLEM_JSON));
        response
    }
}
