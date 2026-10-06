//! Exercise 4: a CI workflow for a Rust project -- generate it, and check
//! that a workflow does what reviewers expect.
//!
//! Most Rust projects gate pull requests on the same checks this repository
//! uses: formatting, clippy with warnings as errors, and tests on every
//! supported OS and on the minimum supported Rust version (MSRV).
//! `workflow` generates a GitHub Actions workflow from a `Project`;
//! `validate` reviews any workflow text.

use std::collections::BTreeMap;

use serde::Serialize;
use serde_yaml_ng::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub name: String,
    /// `rust-version` from Cargo.toml, if set: adds an `msrv` job.
    pub msrv: Option<String>,
    /// e.g. `ubuntu-latest`, `macos-latest`, `windows-latest`
    pub os: Vec<String>,
    /// A workspace adds `--workspace` to cargo commands.
    pub workspace: bool,
}

#[derive(Serialize)]
struct Workflow {
    name: String,
    on: BTreeMap<String, Value>,
    env: BTreeMap<String, String>,
    jobs: BTreeMap<String, Job>,
}

#[derive(Serialize)]
struct Job {
    name: String,
    #[serde(rename = "runs-on")]
    runs_on: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    strategy: Option<Strategy>,
    steps: Vec<Step>,
}

#[derive(Serialize)]
struct Strategy {
    #[serde(rename = "fail-fast")]
    fail_fast: bool,
    matrix: BTreeMap<String, Vec<String>>,
}

#[derive(Serialize, Default)]
struct Step {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uses: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    with: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    run: Option<String>,
}

const CHECKOUT: &str = "actions/checkout@v4";
const CACHE: &str = "Swatinem/rust-cache@v2";

fn uses(action: &str) -> Step {
    Step {
        uses: Some(action.into()),
        ..Step::default()
    }
}

fn toolchain(version: &str, components: Option<&str>) -> Step {
    let mut with = BTreeMap::from([("toolchain".to_string(), version.to_string())]);
    if let Some(c) = components {
        with.insert("components".into(), c.into());
    }
    Step {
        uses: Some("dtolnay/rust-toolchain@master".into()),
        with: Some(with),
        ..Step::default()
    }
}

fn run(name: &str, command: String) -> Step {
    Step {
        name: Some(name.into()),
        run: Some(command),
        ..Step::default()
    }
}

/// A workflow running on pushes to `main` and on pull requests:
///
/// - `check` (ubuntu): `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`
/// - `test`: a matrix over `project.os`, `cargo test`
/// - `msrv` (if set): `cargo check` with that toolchain
///
/// every job checks out the code first, installs the toolchain, and uses
/// a cache; cargo commands get `--workspace` for a workspace.
pub fn workflow(project: &Project) -> String {
    todo!("Exercise 4")
}

/// Review a workflow; every problem found, sorted. Checks:
///
/// - it parses as YAML, and has `on` and a non-empty `jobs` mapping
///   (`invalid YAML: ...`, `missing \`on\``, `no jobs`)
/// - every job has `runs-on` and non-empty `steps`
///   (`job X: missing runs-on`, `job X: no steps`)
/// - every job's first step uses `actions/checkout`
///   (`job X: first step must be actions/checkout`)
/// - every `uses:` is pinned to a version with `@`
///   (`job X: unpinned action Y`)
/// - every `cargo clippy` command denies warnings (`-D warnings`)
///   (`job X: clippy without -D warnings`)
/// - some step runs `cargo fmt` with `--check` (`no cargo fmt --check`),
///   and some step runs `cargo test` (`no cargo test`)
pub fn validate(yaml: &str) -> Vec<String> {
    todo!("Exercise 4")
}
