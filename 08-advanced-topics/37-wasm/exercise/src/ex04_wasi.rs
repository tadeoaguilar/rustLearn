//! Exercise 4: WASI -- the same program on the command line and in a sandbox.
//!
//! WASI (the WebAssembly System Interface) gives a WASM module files,
//! clocks and arguments -- but only the directories the host explicitly
//! *preopens* (`wasmtime run --dir=./data app.wasm`). Code written with
//! plain `std::fs` on relative paths runs unchanged natively and under
//! WASI (`cargo build --target wasm32-wasip1`). No threads, no sockets (in
//! preview 1), no absolute paths outside the preopens.

use std::fs;
use std::io;
use std::path::Path;

/// A file's statistics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileStats {
    pub name: String,
    pub lines: usize,
    pub words: usize,
    pub bytes: usize,
}

/// `wc` for one text.
pub fn stats_of(name: &str, text: &str) -> FileStats {
    todo!("Exercise 4")
}

/// Statistics for every regular file directly in `dir` whose name ends in
/// `extension` (e.g. ".txt"), sorted by name. Unreadable (non-UTF-8) files
/// are skipped.
pub fn scan_dir(dir: &Path, extension: &str) -> io::Result<Vec<FileStats>> {
    todo!("Exercise 4")
}

/// Write a report like `wc`: one line per file, then a total.
pub fn write_report(stats: &[FileStats], out_path: &Path) -> io::Result<()> {
    todo!("Exercise 4")
}
