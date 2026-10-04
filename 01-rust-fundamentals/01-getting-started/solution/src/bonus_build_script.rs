//! Bonus Challenge: a build script.
//!
//! `build.rs` (next to `Cargo.toml`) runs before compilation and exports two
//! environment variables that are baked into the binary with `env!`.

/// Seconds since the Unix epoch at the moment this crate was compiled.
pub fn build_time() -> u64 {
    env!("BUILD_TIME").parse().unwrap_or(0)
}

/// e.g. "built with rustc 1.98.1 at unix time 1791140000"
pub fn build_info() -> String {
    format!(
        "built with rustc {} at unix time {}",
        env!("RUSTC_VERSION"),
        build_time()
    )
}
