//! Exercise 3: the same notes API in Actix-web.
//!
//! The differences from Axum, line by line:
//!
//! | Concern        | Axum                         | Actix-web                         |
//! |----------------|------------------------------|-----------------------------------|
//! | shared state   | `State<NoteStore>`           | `web::Data<NoteStore>`            |
//! | path params    | `Path<u64>`, route `/{id}`   | `web::Path<u64>`, route `/{id}`   |
//! | JSON body      | `Json<T>` (last argument)    | `web::Json<T>` (any position)     |
//! | error -> HTTP  | `impl IntoResponse`          | `impl ResponseError`              |
//! | routes         | `Router::new().route(..)`    | `App::new().configure(..)` / `web::resource` |
//! | middleware     | tower `Layer`s               | `wrap(..)`, `middleware::from_fn` |
//! | runtime        | plain Tokio                  | actix-rt: a Tokio runtime *per worker thread* |
//!
//! `NoteError` implements both actix's `ResponseError` and Axum's
//! `IntoResponse`: both traits are foreign, but the type is ours, so the
//! orphan rule allows it.

use crate::middleware::actix_request_id;
use crate::notes::{NoteError, NoteFilter, NoteInput, NoteStore};
use actix_web::http::StatusCode;
use actix_web::{App, HttpResponse, HttpServer, ResponseError, web};
use serde_json::json;

impl ResponseError for NoteError {
    fn status_code(&self) -> StatusCode {
        todo!("Exercise 3")
    }

    fn error_response(&self) -> HttpResponse {
        todo!("Exercise 3")
    }
}

async fn health() -> HttpResponse {
    todo!("Exercise 3")
}

async fn list_notes(store: web::Data<NoteStore>, filter: web::Query<NoteFilter>) -> HttpResponse {
    todo!("Exercise 3")
}

async fn create_note(
    store: web::Data<NoteStore>,
    input: web::Json<NoteInput>,
) -> Result<HttpResponse, NoteError> {
    todo!("Exercise 3")
}

async fn get_note(
    store: web::Data<NoteStore>,
    id: web::Path<u64>,
) -> Result<HttpResponse, NoteError> {
    todo!("Exercise 3")
}

async fn update_note(
    store: web::Data<NoteStore>,
    id: web::Path<u64>,
    input: web::Json<NoteInput>,
) -> Result<HttpResponse, NoteError> {
    todo!("Exercise 3")
}

async fn delete_note(
    store: web::Data<NoteStore>,
    id: web::Path<u64>,
) -> Result<HttpResponse, NoteError> {
    todo!("Exercise 3")
}

/// Route table, as a function that configures any `App` -- so the server
/// and the tests (`actix_web::test::init_service`) share it.
pub fn configure(cfg: &mut web::ServiceConfig) {
    todo!("Exercise 3")
}

/// Builds the server. `HttpServer::new` takes a *factory*: actix calls it
/// once per worker thread, which is why the store must be cheap to clone.
/// Returns the base URL and the server future to run (or `tokio::spawn`).
pub fn server(
    store: NoteStore,
    addr: &str,
    workers: usize,
) -> std::io::Result<(String, actix_web::dev::Server)> {
    todo!("Exercise 3")
}
