//! Exercise 5: bake build metadata into the binary.
//!
//! A container image should be able to say exactly what it is. Cargo gives a
//! build script the profile and target; the git commit comes from `git`, or
//! from a `GIT_SHA` build argument -- inside `docker build` there is usually
//! no `.git` directory.

use std::env;
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

fn main() {
    let sha = env::var("GIT_SHA")
        .ok()
        .filter(|s| !s.is_empty() && s != "unknown")
        .or_else(|| git(&["rev-parse", "--short=12", "HEAD"]))
        .unwrap_or_else(|| "unknown".into());

    println!("cargo:rustc-env=BUILD_GIT_SHA={sha}");
    println!(
        "cargo:rustc-env=BUILD_PROFILE={}",
        env::var("PROFILE").unwrap()
    );
    println!(
        "cargo:rustc-env=BUILD_TARGET={}",
        env::var("TARGET").unwrap()
    );

    // Rebuild when the commit changes, not on every build.
    println!("cargo:rerun-if-env-changed=GIT_SHA");
    if let Some(head) = git(&["rev-parse", "--path-format=absolute", "--git-path", "HEAD"]) {
        println!("cargo:rerun-if-changed={head}");
    }
    if let Some(refs) = git(&[
        "rev-parse",
        "--path-format=absolute",
        "--git-path",
        "refs/heads",
    ]) {
        println!("cargo:rerun-if-changed={refs}");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
