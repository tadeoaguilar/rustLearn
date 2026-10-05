//! Exercise 5: what exactly is running?
//!
//! "Which version is in production?" should be one request away. The values
//! come from `build.rs` (commit, profile, target) and Cargo (name, version),
//! all fixed at compile time with `env!`.

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BuildInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub git_sha: &'static str,
    pub profile: &'static str,
    pub target: &'static str,
    pub linkage: &'static str,
}

pub fn current() -> BuildInfo {
    BuildInfo {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
        git_sha: env!("BUILD_GIT_SHA"),
        profile: env!("BUILD_PROFILE"),
        target: env!("BUILD_TARGET"),
        linkage: linkage(),
    }
}

/// "static" when the C runtime is linked in (the default on musl targets) --
/// such a binary runs in a `scratch` image with no libc at all.
pub fn linkage() -> &'static str {
    if cfg!(target_feature = "crt-static") {
        "static"
    } else {
        "dynamic"
    }
}
