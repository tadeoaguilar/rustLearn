//! Running the real compiler on a snippet (provided). Exercises 2 and 3
//! inspect what rustc says about code; this does the plumbing.
//!
//! rustc comes with every Rust installation, so this works offline. The
//! `RUSTC` environment variable overrides which compiler is used.

use std::io;
use std::path::PathBuf;
use std::process::Command;

use tempfile::TempDir;

/// The result of one rustc invocation. `dir` holds the source (`main.rs`)
/// and anything rustc wrote; it's deleted when this is dropped.
pub struct Compiled {
    pub success: bool,
    pub stderr: String,
    pub dir: TempDir,
}

impl Compiled {
    pub fn path(&self, name: &str) -> PathBuf {
        self.dir.path().join(name)
    }
}

/// `rustc --edition 2024 <args> main.rs`, in a fresh temporary directory.
pub fn rustc(source: &str, args: &[&str]) -> io::Result<Compiled> {
    let dir = tempfile::tempdir()?;
    std::fs::write(dir.path().join("main.rs"), source)?;
    let compiler = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let output = Command::new(compiler)
        .current_dir(dir.path())
        .args(["--edition", "2024"])
        .args(args)
        .arg("main.rs")
        .output()?;
    Ok(Compiled {
        success: output.status.success(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        dir,
    })
}

/// The MIR of a library snippet, as text (`--emit=mir`; with `-O` if `optimize`).
pub fn emit_mir(source: &str, optimize: bool) -> io::Result<String> {
    let mut args = vec!["--crate-type=lib", "--emit=mir", "-o", "out.mir"];
    if optimize {
        args.push("-O");
    }
    let compiled = rustc(source, &args)?;
    if !compiled.success {
        return Err(io::Error::other(compiled.stderr));
    }
    std::fs::read_to_string(compiled.path("out.mir"))
}

/// Compile a program and run it; its stdout, or the compiler's errors.
pub fn run_program(source: &str) -> io::Result<String> {
    let compiled = rustc(source, &["-o", "program"])?;
    if !compiled.success {
        return Err(io::Error::other(compiled.stderr));
    }
    let output = Command::new(compiled.path("program")).output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "program failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
