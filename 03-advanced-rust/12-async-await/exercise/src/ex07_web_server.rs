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
    todo!("Exercise 7")
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
        todo!("Exercise 7")
    }
}

async fn list_todos(State(state): State<AppState>) -> Json<Vec<Todo>> {
    todo!("Exercise 7")
}

/// Task 1: POST. `Json<NewTodo>` in the arguments parses the body (and
/// rejects malformed JSON with 4xx before our code runs).
async fn create_todo(
    State(state): State<AppState>,
    Json(new): Json<NewTodo>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    todo!("Exercise 7")
}

async fn get_todo(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Todo>, AppError> {
    todo!("Exercise 7")
}

async fn complete_todo(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Todo>, AppError> {
    todo!("Exercise 7")
}

/// Task 3: middleware. Runs around every request: counts it and adds an
/// `x-request-count` header to the response.
async fn count_requests(State(state): State<AppState>, request: Request, next: Next) -> Response {
    todo!("Exercise 7")
}

pub fn app() -> Router {
    todo!("Exercise 7")
}

/// Binds and serves on a background task; returns the base URL. Port 0 means
/// "any free port", so tests never collide.
pub async fn spawn(addr: &str) -> String {
    todo!("Exercise 7")
}

/// `cargo run -p m12-async-await -- serve`: run until Ctrl-C.
pub async fn serve_forever() {
    todo!("Exercise 7")
}

/// Demo: start on a random port, exercise every route with reqwest, stop.
pub async fn run() {
    todo!("Exercise 7")
}
