//! Exercise 2: the notes API in Axum.
//!
//! Axum's model: a handler is any async fn whose arguments are *extractors*
//! (`State`, `Path`, `Query`, `Json`, ...) and whose return type implements
//! `IntoResponse`. Routing, state and middleware are all tower `Service`s and
//! `Layer`s underneath.

use crate::middleware::{request_id, timing};
use crate::notes::{Note, NoteError, NoteFilter, NoteInput, NoteStore};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

/// Exercise 2's error mapping. Implementing `IntoResponse` for our error type
/// lets handlers return `Result<_, NoteError>` and use `?`. (Allowed by the
/// orphan rule because NoteError is local to this crate.)
impl IntoResponse for NoteError {
    fn into_response(self) -> Response {
        todo!("Exercise 2")
    }
}

async fn health() -> Json<serde_json::Value> {
    todo!("Exercise 2")
}

async fn list_notes(
    State(store): State<NoteStore>,
    Query(filter): Query<NoteFilter>,
) -> Json<Vec<Note>> {
    todo!("Exercise 2")
}

/// 201 Created + Location header, the REST convention for "made a new thing".
async fn create_note(
    State(store): State<NoteStore>,
    Json(input): Json<NoteInput>,
) -> Result<impl IntoResponse, NoteError> {
    // `impl IntoResponse` needs a concrete type behind it; replace both lines.
    todo!("Exercise 2: 201 Created, a Location header, and the note as JSON");
    Ok(StatusCode::CREATED)
}

async fn get_note(
    State(store): State<NoteStore>,
    Path(id): Path<u64>,
) -> Result<Json<Note>, NoteError> {
    todo!("Exercise 2")
}

async fn update_note(
    State(store): State<NoteStore>,
    Path(id): Path<u64>,
    Json(input): Json<NoteInput>,
) -> Result<Json<Note>, NoteError> {
    todo!("Exercise 2")
}

async fn delete_note(
    State(store): State<NoteStore>,
    Path(id): Path<u64>,
) -> Result<StatusCode, NoteError> {
    todo!("Exercise 2")
}

/// The notes API on its own. `blog::router` adds the HTML pages.
pub fn notes_router(store: NoteStore) -> Router {
    todo!("Exercise 2")
}

/// Notes API + middleware (Exercise 4). Layers wrap everything added
/// *before* them; the last `.layer` is the outermost.
pub fn app(store: NoteStore) -> Router {
    todo!("Exercise 2")
}

/// Starts the app on a background task; returns its base URL.
/// Bind to "127.0.0.1:0" to get any free port (tests do this).
pub async fn spawn(router: Router, addr: &str) -> String {
    todo!("Exercise 2")
}
