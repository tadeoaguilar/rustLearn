//! Exercises 1-4 and 7, plus the bonus: the routes.
//!
//! /health
//! /openapi.json                     Exercise 5
//! /v1/tasks          GET POST       list (JSON or CSV), create
//! /v1/tasks/{id}     GET PUT DELETE with ETags (bonus)
//! /v2/tasks          GET            the v2 representation
//! /v2/tasks/{id}     GET

use crate::model::{Status, Task, TaskInput, TaskStore};
use crate::pagination::{ListQuery, Page, link_header, paginate};
use crate::problem::Problem;
use crate::rate_limit::{RateLimitConfig, RateLimiter, rate_limit};
use crate::validation::ValidatedJson;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tower_http::cors::{AllowOrigin, CorsLayer};
use utoipa::ToSchema;

#[derive(Debug, Clone, Default)]
pub struct ApiConfig {
    pub rate_limit: Option<RateLimitConfig>,
    /// Origins allowed to call the API from a browser (CORS).
    pub allowed_origins: Vec<String>,
}

// ---- Exercise 1: CRUD -------------------------------------------------------

#[utoipa::path(get, path = "/v1/tasks", params(ListQuery),
    responses((status = 200, description = "A page of tasks (JSON, or CSV with Accept: text/csv)", body = Page<Task>),
              (status = 400, body = Problem), (status = 406, body = Problem)))]
pub async fn list_tasks(
    State(store): State<TaskStore>,
    Query(query): Query<ListQuery>,
    headers: HeaderMap,
) -> Result<Response, Problem> {
    todo!("Exercises 1-4, 7")
}

#[utoipa::path(post, path = "/v1/tasks", request_body = TaskInput,
    responses((status = 201, body = Task), (status = 400, body = Problem), (status = 422, body = Problem)))]
pub async fn create_task(
    State(store): State<TaskStore>,
    ValidatedJson(input): ValidatedJson<TaskInput>,
) -> Response {
    todo!("Exercises 1-4, 7")
}

/// Bonus: conditional GET. A client that already has version N sends
/// `If-None-Match: "id-N"`; if nothing changed, 304 with no body saves the
/// transfer.
#[utoipa::path(get, path = "/v1/tasks/{id}", params(("id" = u64, Path)),
    responses((status = 200, body = Task), (status = 304), (status = 404, body = Problem)))]
pub async fn get_task(
    State(store): State<TaskStore>,
    Path(id): Path<u64>,
    headers: HeaderMap,
) -> Result<Response, Problem> {
    todo!("Exercises 1-4, 7")
}

/// Bonus: optimistic concurrency. With `If-Match: "id-N"`, the update only
/// happens if the task is still at version N; otherwise 412 -- someone else
/// changed it since you read it, and your PUT would silently overwrite them.
#[utoipa::path(put, path = "/v1/tasks/{id}", params(("id" = u64, Path)), request_body = TaskInput,
    responses((status = 200, body = Task), (status = 404, body = Problem), (status = 412, body = Problem), (status = 422, body = Problem)))]
pub async fn replace_task(
    State(store): State<TaskStore>,
    Path(id): Path<u64>,
    headers: HeaderMap,
    ValidatedJson(input): ValidatedJson<TaskInput>,
) -> Result<Response, Problem> {
    todo!("Exercises 1-4, 7")
}

#[utoipa::path(delete, path = "/v1/tasks/{id}", params(("id" = u64, Path)),
    responses((status = 204), (status = 404, body = Problem)))]
pub async fn delete_task(
    State(store): State<TaskStore>,
    Path(id): Path<u64>,
) -> Result<StatusCode, Problem> {
    todo!("Exercises 1-4, 7")
}

pub fn etag(task: &Task) -> HeaderValue {
    todo!("Exercises 1-4, 7")
}

// ---- Exercise 4: versioning --------------------------------------------------

/// v2 changes the representation, which would break v1 clients -- hence a
/// new version rather than an edit: priority becomes a label, and each task
/// links to itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct TaskV2 {
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub status: Status,
    pub priority: String,
    pub links: Links,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Links {
    #[serde(rename = "self")]
    pub self_: String,
}

impl From<Task> for TaskV2 {
    fn from(t: Task) -> Self {
        todo!("Exercises 1-4, 7")
    }
}

#[utoipa::path(get, path = "/v2/tasks", params(ListQuery), responses((status = 200, body = Page<TaskV2>)))]
pub async fn list_tasks_v2(
    State(store): State<TaskStore>,
    Query(query): Query<ListQuery>,
) -> Result<Response, Problem> {
    todo!("Exercises 1-4, 7")
}

#[utoipa::path(get, path = "/v2/tasks/{id}", params(("id" = u64, Path)), responses((status = 200, body = TaskV2), (status = 404, body = Problem)))]
pub async fn get_task_v2(
    State(store): State<TaskStore>,
    Path(id): Path<u64>,
) -> Result<Json<TaskV2>, Problem> {
    todo!("Exercises 1-4, 7")
}

// ---- Exercise 7: content negotiation -----------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Csv,
}

/// Picks the best supported type from `Accept`, honouring q-values:
/// `text/csv;q=0.9, application/json` -> JSON. No header, or `*/*` -> JSON.
/// Only unsupported types -> 406 Not Acceptable.
pub fn negotiate(headers: &HeaderMap) -> Result<Format, Problem> {
    todo!("Exercises 1-4, 7")
}

fn to_csv(tasks: &[Task]) -> String {
    todo!("Exercises 1-4, 7")
}

// ---- Putting it together -------------------------------------------------------

async fn health() -> Json<serde_json::Value> {
    todo!("Exercises 1-4, 7")
}

/// Unknown routes get a problem+json 404 too, not an empty body.
async fn not_found_fallback() -> Problem {
    todo!("Exercises 1-4, 7")
}

pub fn app(store: TaskStore, config: ApiConfig) -> Router {
    todo!("Exercises 1-4, 7")
}
