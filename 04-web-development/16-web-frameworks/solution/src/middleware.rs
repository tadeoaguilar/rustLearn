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
    uuid::Uuid::new_v4().to_string()
}

/// Axum: `axum::middleware::from_fn` turns an async fn `(Request, Next) ->
/// Response` into a tower Layer. Code before `next.run` sees the request,
/// code after it sees the response.
pub async fn request_id(mut request: Request, next: Next) -> Response {
    let id = request
        .headers()
        .get(REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .filter(|v| !v.is_empty() && v.len() <= 128)
        .map(str::to_string)
        .unwrap_or_else(new_request_id);
    let value = HeaderValue::from_str(&id).expect("ids are ASCII");
    request.headers_mut().insert(REQUEST_ID, value.clone()); // handlers can read it too
    let mut response = next.run(request).await;
    response.headers_mut().insert(REQUEST_ID, value);
    response
}

pub async fn timing(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let mut response = next.run(request).await;
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    response.headers_mut().insert(
        RESPONSE_TIME,
        HeaderValue::from_str(&format!("{ms:.3}")).unwrap(),
    );
    response
}

/// Actix-web: `actix_web::middleware::from_fn` (actix-web 4.9+) has the same
/// shape -- request in, `next.call(req)`, response out.
pub async fn actix_request_id(
    req: actix_web::dev::ServiceRequest,
    next: actix_web::middleware::Next<impl actix_web::body::MessageBody>,
) -> Result<actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>, actix_web::Error> {
    use actix_web::http::header::{HeaderName, HeaderValue as ActixHeaderValue};
    let id = req
        .headers()
        .get(REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .filter(|v| !v.is_empty() && v.len() <= 128)
        .map(str::to_string)
        .unwrap_or_else(new_request_id);
    let start = Instant::now();
    let mut res = next.call(req).await?;
    let headers = res.headers_mut();
    headers.insert(
        HeaderName::from_static(REQUEST_ID),
        ActixHeaderValue::from_str(&id).unwrap(),
    );
    let ms = format!("{:.3}", start.elapsed().as_secs_f64() * 1000.0);
    headers.insert(
        HeaderName::from_static(RESPONSE_TIME),
        ActixHeaderValue::from_str(&ms).unwrap(),
    );
    Ok(res)
}
