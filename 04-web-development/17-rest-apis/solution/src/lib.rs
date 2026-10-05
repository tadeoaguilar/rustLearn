//! Module 17 -- REST APIs. Reference solution: a tasks API.
//!
//! | File            | Exercise                                           |
//! |-----------------|----------------------------------------------------|
//! | `model.rs`      | 1  the domain and the store                        |
//! | `problem.rs`    | 2  RFC 7807 problem+json errors                    |
//! | `validation.rs` | 2  a `ValidatedJson<T>` extractor using `validator` |
//! | `pagination.rs` | 3  page/per_page, filtering, sorting, Link headers |
//! | `api.rs`        | 1-4, 7  the routes, v1 and v2, content negotiation |
//! | `openapi.rs`    | 5  OpenAPI document generated with utoipa          |
//! | `rate_limit.rs` | 6  a token-bucket rate limiter middleware           |
//! | `client.rs`     | 8  a typed client SDK                              |

pub mod api;
pub mod client;
pub mod model;
pub mod openapi;
pub mod pagination;
pub mod problem;
pub mod rate_limit;
pub mod validation;

/// Starts `router` on a background task; returns its base URL.
pub async fn spawn(router: axum::Router, addr: &str) -> String {
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    let local = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, router).await.expect("server") });
    format!("http://{local}")
}
