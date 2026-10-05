//! Exercise 3: pagination, filtering, sorting.
//!
//! `GET /v1/tasks?page=2&per_page=10&status=done&sort=-priority`
//!
//! The response carries the page in the body and navigation in headers:
//!   * `X-Total-Count: 57`
//!   * `Link: </v1/tasks?page=3&per_page=10&...>; rel="next", <...>; rel="prev", ...`
//!     (RFC 8288 -- GitHub's API does exactly this)

use crate::model::{Status, Task};
use crate::problem::Problem;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

pub const MAX_PER_PAGE: u32 = 100;

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct ListQuery {
    /// 1-based page number (default 1)
    pub page: Option<u32>,
    /// items per page, 1..=100 (default 20)
    pub per_page: Option<u32>,
    /// only tasks with this status
    pub status: Option<Status>,
    /// `id`, `priority`, `title`; prefix `-` for descending (default `id`)
    pub sort: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Page<T> {
    pub data: Vec<T>,
    pub page: u32,
    pub per_page: u32,
    pub total: usize,
}

impl ListQuery {
    pub fn page(&self) -> u32 {
        todo!("Exercise 3")
    }

    pub fn per_page(&self) -> u32 {
        todo!("Exercise 3")
    }

    /// Bad parameters are the client's fault: 400, with a reason.
    pub fn validate(&self) -> Result<(), Problem> {
        todo!("Exercise 3")
    }
}

/// Filter, sort, then cut out one page. Returns the page and the total
/// number of matching tasks (before paging).
pub fn paginate(mut tasks: Vec<Task>, query: &ListQuery) -> Page<Task> {
    todo!("Exercise 3")
}

/// The `Link` header value for a page, keeping the other query parameters.
pub fn link_header(path: &str, query: &ListQuery, total: usize) -> Option<String> {
    todo!("Exercise 3")
}
