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
    let mut file = File::create(path).await?;
    file.write_all(text.as_bytes()).await?;
    file.flush().await?; // make sure it's on disk before we reopen it

    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(contents)
}

/// Task 1: copy. Returns the number of bytes copied.
pub async fn copy_file(from: &Path, to: &Path) -> io::Result<u64> {
    fs::copy(from, to).await
    // By hand it's: tokio::io::copy(&mut File::open(from).await?, &mut File::create(to).await?).await
}

/// Task 2: read several files *concurrently*. One failure doesn't hide the
/// others' results.
pub async fn read_many(paths: &[PathBuf]) -> Vec<io::Result<String>> {
    join_all(paths.iter().map(fs::read_to_string)).await
}

/// Task 3: line by line -- memory use stays flat however big the file is.
pub async fn count_matching_lines(path: &Path, needle: &str) -> io::Result<usize> {
    let file = File::open(path).await?;
    let mut lines = BufReader::new(file).lines();
    let mut count = 0;
    while let Some(line) = lines.next_line().await? {
        if line.contains(needle) {
            count += 1;
        }
    }
    Ok(count)
}

pub async fn run() {
    let dir = std::env::temp_dir().join(format!("rustlearn-m12-{}", std::process::id()));
    fs::create_dir_all(&dir).await.expect("temp dir");
    let a = dir.join("test.txt");
    println!(
        "Contents: {:?}",
        write_then_read(&a, "Hello, async world!").await
    );

    let b = dir.join("copy.txt");
    println!("copied {:?} bytes", copy_file(&a, &b).await);

    let results = read_many(&[a.clone(), b.clone(), dir.join("missing.txt")]).await;
    for r in &results {
        println!("  read_many -> {:?}", r.as_ref().map_err(|e| e.kind()));
    }

    let big = dir.join("big.txt");
    let text: String = (0..10_000)
        .map(|i| format!("line {i} {}\n", if i % 7 == 0 { "ERROR" } else { "ok" }))
        .collect();
    fs::write(&big, text).await.expect("write big file");
    println!(
        "lines containing ERROR: {:?}",
        count_matching_lines(&big, "ERROR").await
    );

    fs::remove_dir_all(&dir).await.ok();
}
