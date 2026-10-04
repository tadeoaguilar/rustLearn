//! Bonus Challenge: a build script.  -- see exercises.md
//!
//! 1. Create ../build.rs that prints `cargo:rustc-env=BUILD_TIME=<unix secs>`
//! 2. Add `build = "build.rs"` under [package] in ../Cargo.toml (optional --
//!    Cargo finds build.rs automatically)
//! 3. Read it here with `env!("BUILD_TIME")`

/// Seconds since the Unix epoch when this crate was compiled.
pub fn build_time() -> u64 {
    todo!("Bonus: read env!(\"BUILD_TIME\")")
}

/// Any human-readable description of the build.
pub fn build_info() -> String {
    todo!("Bonus")
}
