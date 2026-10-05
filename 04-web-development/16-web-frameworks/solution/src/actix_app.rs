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
        match self {
            NoteError::NotFound(_) => StatusCode::NOT_FOUND,
            NoteError::Invalid(_) => StatusCode::UNPROCESSABLE_ENTITY,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(json!({ "error": self.to_string() }))
    }
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(json!({ "status": "ok", "framework": "actix-web" }))
}

async fn list_notes(store: web::Data<NoteStore>, filter: web::Query<NoteFilter>) -> HttpResponse {
    HttpResponse::Ok().json(store.list(&filter))
}

async fn create_note(
    store: web::Data<NoteStore>,
    input: web::Json<NoteInput>,
) -> Result<HttpResponse, NoteError> {
    let note = store.create(input.into_inner())?;
    Ok(HttpResponse::Created()
        .insert_header(("Location", format!("/notes/{}", note.id)))
        .json(note))
}

async fn get_note(
    store: web::Data<NoteStore>,
    id: web::Path<u64>,
) -> Result<HttpResponse, NoteError> {
    Ok(HttpResponse::Ok().json(store.get(*id)?))
}

async fn update_note(
    store: web::Data<NoteStore>,
    id: web::Path<u64>,
    input: web::Json<NoteInput>,
) -> Result<HttpResponse, NoteError> {
    Ok(HttpResponse::Ok().json(store.update(*id, input.into_inner())?))
}

async fn delete_note(
    store: web::Data<NoteStore>,
    id: web::Path<u64>,
) -> Result<HttpResponse, NoteError> {
    store.delete(*id)?;
    Ok(HttpResponse::NoContent().finish())
}

/// Route table, as a function that configures any `App` -- so the server
/// and the tests (`actix_web::test::init_service`) share it.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health))
        .service(
            web::resource("/notes")
                .route(web::get().to(list_notes))
                .route(web::post().to(create_note)),
        )
        .service(
            web::resource("/notes/{id}")
                .route(web::get().to(get_note))
                .route(web::put().to(update_note))
                .route(web::delete().to(delete_note)),
        );
}

/// Builds the server. `HttpServer::new` takes a *factory*: actix calls it
/// once per worker thread, which is why the store must be cheap to clone.
/// Returns the base URL and the server future to run (or `tokio::spawn`).
pub fn server(
    store: NoteStore,
    addr: &str,
    workers: usize,
) -> std::io::Result<(String, actix_web::dev::Server)> {
    let data = web::Data::new(store);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(actix_web::middleware::from_fn(actix_request_id))
            .configure(configure)
    })
    .workers(workers)
    .bind(addr)?;
    let local = server.addrs()[0];
    Ok((format!("http://{local}"), server.run()))
}
