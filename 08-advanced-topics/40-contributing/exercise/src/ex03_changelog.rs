//! Exercise 3: Conventional Commits and a Keep a Changelog file.
//!
//! Many projects write commit messages as
//! `type(scope)!: description` (Conventional Commits) so that releases can
//! be prepared by tools: the version bump and the changelog section follow
//! from the commits since the last release.
//!
//! The changelog follows keepachangelog.com:
//!
//! ```text
//! # Changelog
//!
//! ## [Unreleased]
//!
//! ## [1.1.0] - 2026-09-01
//!
//! ### Added
//!
//! - **parser:** support comments
//!
//! [Unreleased]: https://github.com/o/r/compare/v1.1.0...HEAD
//! [1.1.0]: https://github.com/o/r/compare/v1.0.0...v1.1.0
//! ```

use crate::ex02_semver::Bump;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub kind: String,
    pub scope: Option<String>,
    pub breaking: bool,
    pub description: String,
}

/// Parse a commit message: the header `type(scope)!: description`, where
/// `type` is lowercase ASCII letters, `(scope)` and `!` are optional, and
/// a footer line `BREAKING CHANGE: ...` (or `BREAKING-CHANGE:`) also marks
/// it breaking.
pub fn parse_commit(message: &str) -> Result<Commit, String> {
    todo!("Exercise 3")
}

/// The bump the commits call for: any breaking -> Major, any `feat` ->
/// Minor, any `fix`/`perf` -> Patch; `None` if nothing user-visible changed
/// (only `docs`, `chore`, `ci`, `test`, ...).
pub fn bump_for(commits: &[Commit]) -> Option<Bump> {
    todo!("Exercise 3")
}

fn entry(commit: &Commit) -> String {
    todo!("Exercise 3")
}

/// A release section. Breaking commits go under `### Changed` (first, marked
/// `**BREAKING:**`), then `feat` -> Added, `perf`/`refactor` -> Changed,
/// `fix` -> Fixed; others are left out. Sections in the order Added,
/// Changed, Fixed; empty ones omitted; commits in the given order.
pub fn release_section(version: &str, date: &str, commits: &[Commit]) -> String {
    todo!("Exercise 3")
}

/// The most recent released version in a changelog (the first `## [x.y.z]`
/// heading that isn't Unreleased).
pub fn latest_version(changelog: &str) -> Option<String> {
    todo!("Exercise 3")
}

/// Insert a release right after the `## [Unreleased]` section (before the
/// previous release), and update the compare links at the bottom:
/// `[Unreleased]` now compares from the new version, and a link for the new
/// version is added below it (when there was an `[Unreleased]:` link).
pub fn insert_release(
    changelog: &str,
    version: &str,
    date: &str,
    commits: &[Commit],
) -> Result<String, String> {
    todo!("Exercise 3")
}
