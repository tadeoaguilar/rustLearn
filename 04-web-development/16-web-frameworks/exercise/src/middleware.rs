//! Exercise 4: middleware -- the same two concerns in both frameworks.
//!
//! * `x-request-id`: echo the client's, or generate a UUID. Put it in logs
//!   and error reports and you can follow one request through every service.
//! * `x-response-time-ms`: how long the handler took.

use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use std::time::Instant;

pub const REQUEST_ID: &str = "x-request-id";
pub const RESPONSE_TIME: &str = "x-response-time-ms";

fn new_request_id() -> String {
    todo!("Exercise 4")
}

/// Axum: `axum::middleware::from_fn` turns an async fn `(Request, Next) ->
/// Response` into a tower Layer. Code before `next.run` sees the request,
/// code after it sees the response.
pub async fn request_id(mut request: Request, next: Next) -> Response {
    // TODO Exercise 4. Until then this passes requests straight through --
    // a todo!() here would break every route, not just this exercise.
    next.run(request).await
}

pub async fn timing(request: Request, next: Next) -> Response {
    // TODO Exercise 4 (pass-through for now)
    next.run(request).await
}

/// Actix-web: `actix_web::middleware::from_fn` (actix-web 4.9+) has the same
/// shape -- request in, `next.call(req)`, response out.
pub async fn actix_request_id(
    req: actix_web::dev::ServiceRequest,
    next: actix_web::middleware::Next<impl actix_web::body::MessageBody>,
) -> Result<actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>, actix_web::Error> {
    // TODO Exercise 4 (pass-through for now)
    next.call(req).await
}
