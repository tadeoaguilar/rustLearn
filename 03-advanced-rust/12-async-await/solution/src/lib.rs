//! Module 12 -- Async/Await. Reference solution.
//!
//! An `async fn` returns a *future*: a value describing work that hasn't
//! started. Nothing happens until something `.await`s it (or a runtime like
//! Tokio polls it). Awaiting yields control to the runtime instead of
//! blocking the thread, so one thread can juggle thousands of tasks that are
//! mostly waiting -- on timers, sockets, files.
//!
//! Nothing in this crate touches the public internet: the HTTP client
//! (Exercise 4) talks to `mock_api`, a small axum server started in-process
//! on a random local port.

pub mod bonus_task_queue;
pub mod ex01_basics;
pub mod ex02_concurrency;
pub mod ex03_file_io;
pub mod ex04_http_client;
pub mod ex05_channels;
pub mod ex06_tasks;
pub mod ex07_web_server;
pub mod ex08_streams;
pub mod mock_api;
