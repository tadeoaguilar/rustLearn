//! Build script -- Exercise 1 and the Bonus Challenge.
//!
//! A build script runs on the *host* before your crate compiles. Whatever it
//! prints as `cargo:rustc-env=NAME=value` becomes available to the crate as a
//! compile-time constant through `env!("NAME")`.
//!
//! The exercise suggests `chrono::Utc::now()` here. That works, but it means a
//! `[build-dependencies]` entry just to format one timestamp; the standard
//! library's `SystemTime` gives us seconds since the Unix epoch for free.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Only re-run this script when it changes, not on every build.
    println!("cargo:rerun-if-changed=build.rs");

    // Cargo tells build scripts which compiler it is using via $RUSTC.
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let version = Command::new(rustc)
        .arg("--version")
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        // "rustc 1.98.1 (48a229cea 2026-09-01)" -> "1.98.1"
        .and_then(|s| s.split_whitespace().nth(1).map(str::to_string))
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=RUSTC_VERSION={version}");

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    println!("cargo:rustc-env=BUILD_TIME={secs}");
}
