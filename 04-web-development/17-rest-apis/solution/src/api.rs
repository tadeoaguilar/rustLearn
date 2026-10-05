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
    query.validate()?;
    let format = negotiate(&headers)?;
    let page = paginate(store.all(), &query);
    let link = link_header("/v1/tasks", &query, page.total);
    let mut response = match format {
        Format::Json => Json(&page).into_response(),
        Format::Csv => (
            [(header::CONTENT_TYPE, "text/csv; charset=utf-8")],
            to_csv(&page.data),
        )
            .into_response(),
    };
    let headers = response.headers_mut();
    headers.insert("x-total-count", HeaderValue::from(page.total));
    if let Some(link) = link {
        headers.insert(header::LINK, HeaderValue::from_str(&link).expect("ascii"));
    }
    Ok(response)
}

#[utoipa::path(post, path = "/v1/tasks", request_body = TaskInput,
    responses((status = 201, body = Task), (status = 400, body = Problem), (status = 422, body = Problem)))]
pub async fn create_task(
    State(store): State<TaskStore>,
    ValidatedJson(input): ValidatedJson<TaskInput>,
) -> Response {
    let task = store.create(input);
    let mut response = (StatusCode::CREATED, Json(&task)).into_response();
    response.headers_mut().insert(
        header::LOCATION,
        HeaderValue::from_str(&format!("/v1/tasks/{}", task.id)).unwrap(),
    );
    response.headers_mut().insert(header::ETAG, etag(&task));
    response
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
    let task = store
        .get(id)
        .ok_or_else(|| Problem::not_found(&format!("task {id}")))?;
    let tag = etag(&task);
    if headers.get(header::IF_NONE_MATCH) == Some(&tag) {
        return Ok((StatusCode::NOT_MODIFIED, [(header::ETAG, tag)]).into_response());
    }
    Ok(([(header::ETAG, tag)], Json(task)).into_response())
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
    let current = store
        .get(id)
        .ok_or_else(|| Problem::not_found(&format!("task {id}")))?;
    if let Some(expected) = headers.get(header::IF_MATCH)
        && expected != etag(&current)
    {
        return Err(Problem::new(
            StatusCode::PRECONDITION_FAILED,
            "precondition-failed",
            "Precondition failed",
        )
        .with_detail(format!(
            "task {id} was modified; it is now at {}",
            etag(&current).to_str().unwrap()
        )));
    }
    // (A real database would do the check and the write atomically, e.g.
    //  UPDATE ... WHERE id = ? AND version = ?.)
    let task = store
        .replace(id, input)
        .ok_or_else(|| Problem::not_found(&format!("task {id}")))?;
    Ok(([(header::ETAG, etag(&task))], Json(task)).into_response())
}

#[utoipa::path(delete, path = "/v1/tasks/{id}", params(("id" = u64, Path)),
    responses((status = 204), (status = 404, body = Problem)))]
pub async fn delete_task(
    State(store): State<TaskStore>,
    Path(id): Path<u64>,
) -> Result<StatusCode, Problem> {
    if store.delete(id) {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(Problem::not_found(&format!("task {id}")))
    }
}

pub fn etag(task: &Task) -> HeaderValue {
    HeaderValue::from_str(&format!("\"{}-{}\"", task.id, task.version)).expect("ascii")
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
        let priority =
            ["lowest", "low", "medium", "high", "highest"][(t.priority.clamp(1, 5) - 1) as usize];
        TaskV2 {
            links: Links {
                self_: format!("/v2/tasks/{}", t.id),
            },
            id: t.id,
            title: t.title,
            description: t.description,
            status: t.status,
            priority: priority.to_string(),
        }
    }
}

#[utoipa::path(get, path = "/v2/tasks", params(ListQuery), responses((status = 200, body = Page<TaskV2>)))]
pub async fn list_tasks_v2(
    State(store): State<TaskStore>,
    Query(query): Query<ListQuery>,
) -> Result<Response, Problem> {
    query.validate()?;
    let page = paginate(store.all(), &query);
    let link = link_header("/v2/tasks", &query, page.total);
    let page = Page {
        data: page.data.into_iter().map(TaskV2::from).collect::<Vec<_>>(),
        page: page.page,
        per_page: page.per_page,
        total: page.total,
    };
    let mut response = Json(page).into_response();
    if let Some(link) = link {
        response
            .headers_mut()
            .insert(header::LINK, HeaderValue::from_str(&link).unwrap());
    }
    Ok(response)
}

#[utoipa::path(get, path = "/v2/tasks/{id}", params(("id" = u64, Path)), responses((status = 200, body = TaskV2), (status = 404, body = Problem)))]
pub async fn get_task_v2(
    State(store): State<TaskStore>,
    Path(id): Path<u64>,
) -> Result<Json<TaskV2>, Problem> {
    store
        .get(id)
        .map(|t| Json(t.into()))
        .ok_or_else(|| Problem::not_found(&format!("task {id}")))
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
    let Some(accept) = headers.get(header::ACCEPT).and_then(|v| v.to_str().ok()) else {
        return Ok(Format::Json);
    };
    let mut best: Option<(f32, Format)> = None;
    for part in accept.split(',') {
        let mut pieces = part.split(';').map(str::trim);
        let media = pieces.next().unwrap_or_default();
        let q = pieces
            .find_map(|p| p.strip_prefix("q="))
            .and_then(|q| q.parse::<f32>().ok())
            .unwrap_or(1.0);
        let format = match media {
            "application/json" | "application/*" | "*/*" => Format::Json,
            "text/csv" | "text/*" => Format::Csv,
            _ => continue,
        };
        if q > 0.0 && best.is_none_or(|(bq, _)| q > bq) {
            best = Some((q, format));
        }
    }
    best.map(|(_, f)| f).ok_or_else(|| {
        Problem::new(
            StatusCode::NOT_ACCEPTABLE,
            "not-acceptable",
            "Not acceptable",
        )
        .with_detail("supported types: application/json, text/csv")
    })
}

fn to_csv(tasks: &[Task]) -> String {
    let escape = |s: &str| {
        if s.contains([',', '"', '\n']) {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s.to_string()
        }
    };
    let mut out = String::from("id,title,status,priority\n");
    for t in tasks {
        let status = serde_json::to_value(t.status).unwrap();
        out.push_str(&format!(
            "{},{},{},{}\n",
            t.id,
            escape(&t.title),
            status.as_str().unwrap(),
            t.priority
        ));
    }
    out
}

// ---- Putting it together -------------------------------------------------------

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

/// Unknown routes get a problem+json 404 too, not an empty body.
async fn not_found_fallback() -> Problem {
    Problem::not_found("this route")
}

pub fn app(store: TaskStore, config: ApiConfig) -> Router {
    let mut api = Router::new()
        .route("/v1/tasks", get(list_tasks).post(create_task))
        .route(
            "/v1/tasks/{id}",
            get(get_task).put(replace_task).delete(delete_task),
        )
        .route("/v2/tasks", get(list_tasks_v2))
        .route("/v2/tasks/{id}", get(get_task_v2))
        .with_state(store);

    // Exercise 6: only the API routes are rate limited, not /health or the docs.
    if let Some(rl) = config.rate_limit {
        api = api.layer(axum::middleware::from_fn_with_state(
            RateLimiter::new(rl),
            rate_limit,
        ));
    }

    // Exercise 7: CORS. Browsers send a preflight OPTIONS request before
    // cross-origin calls with JSON bodies; this layer answers it.
    let origins: Vec<HeaderValue> = config
        .allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::IF_MATCH,
            header::IF_NONE_MATCH,
        ])
        .expose_headers([
            header::LINK,
            header::ETAG,
            header::HeaderName::from_static("x-total-count"),
        ]);

    Router::new()
        .route("/health", get(health))
        .route("/openapi.json", get(crate::openapi::openapi_json))
        .merge(api)
        .fallback(not_found_fallback)
        .layer(cors)
}
