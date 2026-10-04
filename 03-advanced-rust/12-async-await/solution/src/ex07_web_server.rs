//! Exercise 7: Async Web Server with axum.
//!
//! Note: the exercise targets axum 0.7. In 0.8 (used here) path parameters
//! are written `/{id}` instead of `/:id`; everything else shown is the same.

use axum::extract::{Path, Request, State};
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;

#[derive(Serialize)]
pub struct Message {
    pub content: String,
}

pub async fn handler() -> Json<Message> {
    Json(Message {
        content: "Hello, async!".to_string(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub done: bool,
}

#[derive(Debug, Deserialize)]
pub struct NewTodo {
    pub title: String,
}

/// Task 2: the "database mock" -- shared state handed to every handler.
/// `tokio::sync::RwLock` because a handler might hold it across an `.await`.
#[derive(Clone, Default)]
pub struct AppState {
    todos: Arc<RwLock<BTreeMap<u64, Todo>>>,
    next_id: Arc<AtomicU64>,
    requests: Arc<AtomicU64>,
}

/// Task 4: one error type for the API. `IntoResponse` decides the status
/// code and body, so handlers can just use `?`.
#[derive(Debug)]
pub enum AppError {
    NotFound(u64),
    Invalid(&'static str),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(id) => (StatusCode::NOT_FOUND, format!("todo {id} not found")),
            AppError::Invalid(why) => (StatusCode::UNPROCESSABLE_ENTITY, why.to_string()),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

async fn list_todos(State(state): State<AppState>) -> Json<Vec<Todo>> {
    Json(state.todos.read().await.values().cloned().collect())
}

/// Task 1: POST. `Json<NewTodo>` in the arguments parses the body (and
/// rejects malformed JSON with 4xx before our code runs).
async fn create_todo(
    State(state): State<AppState>,
    Json(new): Json<NewTodo>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    let title = new.title.trim();
    if title.is_empty() {
        return Err(AppError::Invalid("title must not be empty"));
    }
    let id = state.next_id.fetch_add(1, Ordering::SeqCst) + 1;
    let todo = Todo {
        id,
        title: title.to_string(),
        done: false,
    };
    state.todos.write().await.insert(id, todo.clone());
    Ok((StatusCode::CREATED, Json(todo)))
}

async fn get_todo(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Todo>, AppError> {
    state
        .todos
        .read()
        .await
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(AppError::NotFound(id))
}

async fn complete_todo(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Todo>, AppError> {
    let mut todos = state.todos.write().await;
    let todo = todos.get_mut(&id).ok_or(AppError::NotFound(id))?;
    todo.done = true;
    Ok(Json(todo.clone()))
}

/// Task 3: middleware. Runs around every request: counts it and adds an
/// `x-request-count` header to the response.
async fn count_requests(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let n = state.requests.fetch_add(1, Ordering::SeqCst) + 1;
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("x-request-count", HeaderValue::from(n));
    response
}

pub fn app() -> Router {
    let state = AppState::default();
    Router::new()
        .route("/", get(handler))
        .route("/todos", get(list_todos).post(create_todo))
        .route("/todos/{id}", get(get_todo).patch(complete_todo))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            count_requests,
        ))
        .with_state(state)
}

/// Binds and serves on a background task; returns the base URL. Port 0 means
/// "any free port", so tests never collide.
pub async fn spawn(addr: &str) -> String {
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    let local = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, app()).await.expect("server") });
    format!("http://{local}")
}

/// `cargo run -p m12-async-await-solution -- serve`: run until Ctrl-C.
pub async fn serve_forever() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("port 3000 is free");
    println!("Server running on http://127.0.0.1:3000  (Ctrl-C to stop)");
    println!("  curl localhost:3000/");
    println!(
        "  curl -X POST localhost:3000/todos -H 'content-type: application/json' -d '{{\"title\":\"learn async\"}}'"
    );
    println!("  curl -i localhost:3000/todos/1");
    axum::serve(listener, app())
        .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.expect("ctrl-c handler") })
        .await
        .expect("server");
}

/// Demo: start on a random port, exercise every route with reqwest, stop.
pub async fn run() {
    let base = spawn("127.0.0.1:0").await;
    let http = reqwest::Client::new();
    println!(
        "GET /            -> {}",
        http.get(&base).send().await.unwrap().text().await.unwrap()
    );
    let created = http
        .post(format!("{base}/todos"))
        .json(&serde_json::json!({"title": "learn async"}))
        .send()
        .await
        .unwrap();
    println!(
        "POST /todos      -> {} {}",
        created.status(),
        created.text().await.unwrap()
    );
    let bad = http
        .post(format!("{base}/todos"))
        .json(&serde_json::json!({"title": "  "}))
        .send()
        .await
        .unwrap();
    println!(
        "POST empty title -> {} {}",
        bad.status(),
        bad.text().await.unwrap()
    );
    let done = http.patch(format!("{base}/todos/1")).send().await.unwrap();
    println!("PATCH /todos/1   -> {}", done.text().await.unwrap());
    let missing = http.get(format!("{base}/todos/99")).send().await.unwrap();
    let count = missing.headers().get("x-request-count").cloned();
    println!(
        "GET /todos/99    -> {} {} (x-request-count: {count:?})",
        missing.status(),
        missing.text().await.unwrap()
    );
    println!("Run it for real: cargo run -p m12-async-await-solution -- serve");
}
