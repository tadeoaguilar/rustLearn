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
        let status = match self {
            NoteError::NotFound(_) => StatusCode::NOT_FOUND,
            NoteError::Invalid(_) => StatusCode::UNPROCESSABLE_ENTITY,
        };
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok", "framework": "axum" }))
}

async fn list_notes(
    State(store): State<NoteStore>,
    Query(filter): Query<NoteFilter>,
) -> Json<Vec<Note>> {
    Json(store.list(&filter))
}

/// 201 Created + Location header, the REST convention for "made a new thing".
async fn create_note(
    State(store): State<NoteStore>,
    Json(input): Json<NoteInput>,
) -> Result<impl IntoResponse, NoteError> {
    let note = store.create(input)?;
    let location = format!("/notes/{}", note.id);
    Ok((
        StatusCode::CREATED,
        [(axum::http::header::LOCATION, location)],
        Json(note),
    ))
}

async fn get_note(
    State(store): State<NoteStore>,
    Path(id): Path<u64>,
) -> Result<Json<Note>, NoteError> {
    store.get(id).map(Json)
}

async fn update_note(
    State(store): State<NoteStore>,
    Path(id): Path<u64>,
    Json(input): Json<NoteInput>,
) -> Result<Json<Note>, NoteError> {
    store.update(id, input).map(Json)
}

async fn delete_note(
    State(store): State<NoteStore>,
    Path(id): Path<u64>,
) -> Result<StatusCode, NoteError> {
    store.delete(id)?;
    Ok(StatusCode::NO_CONTENT)
}

/// The notes API on its own. `blog::router` adds the HTML pages.
pub fn notes_router(store: NoteStore) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/notes", get(list_notes).post(create_note))
        .route(
            "/notes/{id}",
            get(get_note).put(update_note).delete(delete_note),
        )
        .with_state(store)
}

/// Notes API + middleware (Exercise 4). Layers wrap everything added
/// *before* them; the last `.layer` is the outermost.
pub fn app(store: NoteStore) -> Router {
    notes_router(store)
        .layer(axum::middleware::from_fn(timing))
        .layer(axum::middleware::from_fn(request_id))
}

/// Starts the app on a background task; returns its base URL.
/// Bind to "127.0.0.1:0" to get any free port (tests do this).
pub async fn spawn(router: Router, addr: &str) -> String {
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    let local = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, router).await.expect("axum server") });
    format!("http://{local}")
}
