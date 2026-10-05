//! Module 21 -- Containerization. YOUR WORKSPACE.
//!
//! | File                 | Exercise |
//! |----------------------|----------|
//! | `config.rs`          | 1  configuration from the environment |
//! | `health.rs`          | 2  liveness, readiness, startup |
//! | `server.rs`          | 3  graceful shutdown |
//! | `probe.rs`           | 4  a built-in health check |
//! | `build_info.rs`      | 5  build metadata (`build.rs` is provided) |
//! | `dockerfile_lint.rs` | 6  a Dockerfile linter |
//! | `compose.rs`         | 7  Compose start order |
//! | `dep_watch.rs`       | bonus |
//! | `../docker/`         | 6, 8  your Dockerfiles and compose.yaml |
//!
//! Each file has the signatures the tests expect, with `todo!()` bodies.
//! Replace each `todo!()` with your implementation, then:
//!
//!     cargo run  -p m21-containerization -- demo
//!     cargo test -p m21-containerization-tests --features mine
//!
//! Stuck? Compare with ../solution/src -- same file and function names.

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]

pub mod build_info;
pub mod compose;
pub mod config;
pub mod dep_watch;
pub mod dockerfile_lint;
pub mod health;
pub mod probe;
pub mod server;

/// The directory holding this crate's Dockerfiles and compose.yaml.
pub const DOCKER_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/docker");
