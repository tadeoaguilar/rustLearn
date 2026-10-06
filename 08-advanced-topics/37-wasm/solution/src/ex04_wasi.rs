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
    FileStats {
        name: name.to_string(),
        lines: text.lines().count(),
        words: text.split_whitespace().count(),
        bytes: text.len(),
    }
}

/// Statistics for every regular file directly in `dir` whose name ends in
/// `extension` (e.g. ".txt"), sorted by name. Unreadable (non-UTF-8) files
/// are skipped.
pub fn scan_dir(dir: &Path, extension: &str) -> io::Result<Vec<FileStats>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !entry.file_type()?.is_file() || !name.ends_with(extension) {
            continue;
        }
        if let Ok(text) = fs::read_to_string(entry.path()) {
            out.push(stats_of(&name, &text));
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// Write a report like `wc`: one line per file, then a total.
pub fn write_report(stats: &[FileStats], out_path: &Path) -> io::Result<()> {
    let mut text = String::new();
    let (mut l, mut w, mut b) = (0, 0, 0);
    for s in stats {
        text.push_str(&format!(
            "{:>6} {:>6} {:>8} {}\n",
            s.lines, s.words, s.bytes, s.name
        ));
        (l, w, b) = (l + s.lines, w + s.words, b + s.bytes);
    }
    text.push_str(&format!("{l:>6} {w:>6} {b:>8} total\n"));
    fs::write(out_path, text)
}
