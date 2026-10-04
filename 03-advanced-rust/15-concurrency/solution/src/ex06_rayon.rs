//! Exercise 6: Data Parallelism with Rayon.
//!
//! `par_iter()` splits the work across a global thread pool (one thread per
//! core) using work-stealing. Same adaptors as normal iterators; the closure
//! must be `Fn + Sync + Send` because it runs on many threads at once.

use rayon::prelude::*;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub fn sum_of_squares_seq(v: &[u64]) -> u64 {
    v.iter().map(|x| x * x).sum()
}

pub fn sum_of_squares_par(v: &[u64]) -> u64 {
    v.par_iter().map(|x| x * x).sum()
}

fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    (2..)
        .take_while(|d| d * d <= n)
        .all(|d| !n.is_multiple_of(d))
}

pub fn count_primes_seq(limit: u64) -> usize {
    (0..limit).filter(|&n| is_prime(n)).count()
}

pub fn count_primes_par(limit: u64) -> usize {
    (0..limit).into_par_iter().filter(|&n| is_prime(n)).count()
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileStats {
    pub name: String,
    pub lines: usize,
    pub words: usize,
    pub bytes: usize,
}

fn stats_for(path: &Path) -> io::Result<FileStats> {
    let text = std::fs::read_to_string(path)?;
    Ok(FileStats {
        name: path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        lines: text.lines().count(),
        words: text.split_whitespace().count(),
        bytes: text.len(),
    })
}

/// Every `.txt` file in `dir`, processed in parallel. Per-file stats sorted by
/// name, plus the total. One unreadable file fails the whole call.
pub fn process_dir(dir: &Path) -> io::Result<(Vec<FileStats>, FileStats)> {
    let paths: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "txt"))
        .collect();

    // Result<Vec<_>, _> collects from a parallel iterator too: the first
    // error wins.
    let mut stats: Vec<FileStats> = paths
        .par_iter()
        .map(|p| stats_for(p))
        .collect::<io::Result<_>>()?;
    stats.sort_by(|a, b| a.name.cmp(&b.name));

    let total = stats.iter().fold(
        FileStats {
            name: "total".into(),
            ..Default::default()
        },
        |acc, s| FileStats {
            lines: acc.lines + s.lines,
            words: acc.words + s.words,
            bytes: acc.bytes + s.bytes,
            ..acc
        },
    );
    Ok((stats, total))
}

pub fn time<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let start = Instant::now();
    let v = f();
    (v, start.elapsed())
}

pub fn run() {
    let data: Vec<u64> = (0..5_000_000).collect();
    let (a, ta) = time(|| sum_of_squares_seq(&data));
    let (b, tb) = time(|| sum_of_squares_par(&data));
    println!("sum of squares: seq {ta:?}, par {tb:?}, equal: {}", a == b);
    let (a, ta) = time(|| count_primes_seq(300_000));
    let (b, tb) = time(|| count_primes_par(300_000));
    println!(
        "primes below 300k: {a} -- seq {ta:?}, par {tb:?} ({} threads), equal: {}",
        rayon::current_num_threads(),
        a == b
    );
    println!("(timings mean little in a debug build: try --release)");

    let dir = std::env::temp_dir().join(format!("rustlearn-m15-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for i in 0..20 {
        std::fs::write(
            dir.join(format!("doc{i:02}.txt")),
            "lorem ipsum dolor\n".repeat(i + 1),
        )
        .unwrap();
    }
    std::fs::write(dir.join("skip.md"), "not a .txt file").unwrap();
    let (files, total) = process_dir(&dir).unwrap();
    println!("process_dir: {} files, total {:?}", files.len(), total);
    std::fs::remove_dir_all(&dir).ok();
}
