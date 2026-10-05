//! Module 21 -- Containerization. Reference solution.
//!
//! A container runs one process with no terminal, configured by environment
//! variables, stopped with SIGTERM, and watched by probes. This crate is a
//! small HTTP service that behaves well in that world, plus tools that check
//! the container files themselves.
//!
//! | File                 | Exercise |
//! |----------------------|----------|
//! | `config.rs`          | 1  configuration from the environment, secrets from files |
//! | `health.rs`          | 2  liveness, readiness and startup probes |
//! | `server.rs`          | 3  the service and graceful shutdown |
//! | `probe.rs`           | 4  a built-in health check for images without curl |
//! | `build_info.rs`      | 5  version and commit baked in at build time (see `build.rs`) |
//! | `dockerfile_lint.rs` | 6  a Dockerfile linter, run against `docker/*.Dockerfile` |
//! | `compose.rs`         | 7  Compose `depends_on`: start order and validation |
//! | `dep_watch.rs`       | bonus: readiness that follows a dependency |
//!
//! Exercise 8 -- the Dockerfiles and Compose file -- is in `docker/`.

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
