//! Module 16 -- Web Frameworks. Reference solution.
//!
//! One small notes API, implemented twice -- in Axum and in Actix-web -- on
//! top of one framework-agnostic core. Comparing `axum_app.rs` with
//! `actix_app.rs` side by side *is* the framework comparison: the business
//! logic is identical, only the HTTP glue differs.
//!
//! | File          | What                                          |
//! |---------------|-----------------------------------------------|
//! | `notes.rs`    | the core: data, validation, storage. No HTTP. |
//! | `axum_app.rs` | the API in Axum (tower-based)                 |
//! | `actix_app.rs`| the same API in Actix-web                     |
//! | `middleware.rs` | request IDs and timing, for both            |
//! | `blog.rs`     | server-rendered HTML: askama templates, forms, static files |
//! | `loadtest.rs` | a tiny load generator to compare the two      |

pub mod actix_app;
pub mod axum_app;
pub mod blog;
pub mod loadtest;
pub mod middleware;
pub mod notes;
