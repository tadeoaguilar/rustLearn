//! Exercise 3: Async File I/O.
//!
//! Tokio's file API mirrors std's, with `.await`. (Under the hood most OSes
//! have no truly async file I/O; Tokio runs these calls on a blocking thread
//! pool. Still worth it: your async tasks don't stall while the disk works.)

use futures::future::join_all;
use std::io;
use std::path::{Path, PathBuf};
use tokio::fs::{self, File};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

/// The exercise's program, with the path as a parameter.
pub async fn write_then_read(path: &Path, text: &str) -> io::Result<String> {
    todo!("Exercise 3")
}

/// Task 1: copy. Returns the number of bytes copied.
pub async fn copy_file(from: &Path, to: &Path) -> io::Result<u64> {
    todo!("Exercise 3")
}

/// Task 2: read several files *concurrently*. One failure doesn't hide the
/// others' results.
pub async fn read_many(paths: &[PathBuf]) -> Vec<io::Result<String>> {
    todo!("Exercise 3")
}

/// Task 3: line by line -- memory use stays flat however big the file is.
pub async fn count_matching_lines(path: &Path, needle: &str) -> io::Result<usize> {
    todo!("Exercise 3")
}

pub async fn run() {
    todo!("Exercise 3")
}
