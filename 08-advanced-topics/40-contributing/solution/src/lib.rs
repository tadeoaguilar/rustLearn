//! Module 40 -- Contributing. Reference solution.
//!
//! The craft around the code: fixing a reported bug the way maintainers
//! like, picking the right version number, writing the changelog, and the
//! CI that guards a project.
//!
//! | File                   | Exercise |
//! |------------------------|----------|
//! | `ex01_upstream.rs`     | 1  Fix four reported bugs (with regression tests) |
//! | `ex02_semver.rs`       | 2  A semver checker: API diff -> required version bump |
//! | `ex03_changelog.rs`    | 3  Conventional Commits -> Keep a Changelog release |
//! | `ex04_ci.rs`           | 4  Generate and review a CI workflow |
//! | `bonus_commit_lint.rs` | bonus: a commit message linter |

pub mod bonus_commit_lint;
pub mod ex01_upstream;
pub mod ex02_semver;
pub mod ex03_changelog;
pub mod ex04_ci;
