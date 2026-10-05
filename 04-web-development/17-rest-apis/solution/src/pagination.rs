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
        self.page.unwrap_or(1)
    }

    pub fn per_page(&self) -> u32 {
        self.per_page.unwrap_or(20)
    }

    /// Bad parameters are the client's fault: 400, with a reason.
    pub fn validate(&self) -> Result<(), Problem> {
        if self.page() == 0 {
            return Err(Problem::bad_request("page starts at 1"));
        }
        if !(1..=MAX_PER_PAGE).contains(&self.per_page()) {
            return Err(Problem::bad_request(format!(
                "per_page must be between 1 and {MAX_PER_PAGE}"
            )));
        }
        if let Some(sort) = &self.sort
            && !matches!(sort.trim_start_matches('-'), "id" | "priority" | "title")
        {
            return Err(Problem::bad_request(format!("cannot sort by {sort:?}")));
        }
        Ok(())
    }
}

/// Filter, sort, then cut out one page. Returns the page and the total
/// number of matching tasks (before paging).
pub fn paginate(mut tasks: Vec<Task>, query: &ListQuery) -> Page<Task> {
    if let Some(status) = query.status {
        tasks.retain(|t| t.status == status);
    }
    let sort = query.sort.as_deref().unwrap_or("id");
    let (descending, key) = match sort.strip_prefix('-') {
        Some(k) => (true, k),
        None => (false, sort),
    };
    tasks.sort_by(|a, b| {
        let ord = match key {
            "priority" => a.priority.cmp(&b.priority),
            "title" => a.title.cmp(&b.title),
            _ => a.id.cmp(&b.id),
        }
        .then(a.id.cmp(&b.id)); // stable tie-break: same order on every request
        if descending { ord.reverse() } else { ord }
    });
    let total = tasks.len();
    let (page, per_page) = (query.page(), query.per_page());
    let start = ((page - 1) * per_page) as usize;
    let data = tasks
        .into_iter()
        .skip(start)
        .take(per_page as usize)
        .collect();
    Page {
        data,
        page,
        per_page,
        total,
    }
}

/// The `Link` header value for a page, keeping the other query parameters.
pub fn link_header(path: &str, query: &ListQuery, total: usize) -> Option<String> {
    let (page, per_page) = (query.page(), query.per_page());
    let last = (total as u32).div_ceil(per_page).max(1);
    let mut extra = String::new();
    if let Some(status) = query.status {
        extra.push_str(&format!(
            "&status={}",
            serde_json::to_value(status).unwrap().as_str().unwrap()
        ));
    }
    if let Some(sort) = &query.sort {
        extra.push_str(&format!("&sort={sort}"));
    }
    let url = |p: u32| format!("<{path}?page={p}&per_page={per_page}{extra}>");
    let mut links = Vec::new();
    if page < last {
        links.push(format!("{}; rel=\"next\"", url(page + 1)));
    }
    if page > 1 {
        links.push(format!("{}; rel=\"prev\"", url(page - 1)));
    }
    links.push(format!("{}; rel=\"first\"", url(1)));
    links.push(format!("{}; rel=\"last\"", url(last)));
    Some(links.join(", "))
}
