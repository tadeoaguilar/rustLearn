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
    let ws = if project.workspace {
        " --workspace"
    } else {
        ""
    };
    let mut jobs = BTreeMap::new();
    jobs.insert(
        "check".to_string(),
        Job {
            name: "fmt + clippy".into(),
            runs_on: "ubuntu-latest".into(),
            strategy: None,
            steps: vec![
                uses(CHECKOUT),
                toolchain("stable", Some("rustfmt, clippy")),
                uses(CACHE),
                run("fmt", "cargo fmt --all --check".into()),
                run(
                    "clippy",
                    format!("cargo clippy{ws} --all-targets -- -D warnings"),
                ),
            ],
        },
    );
    jobs.insert(
        "test".to_string(),
        Job {
            name: "test (${{ matrix.os }})".into(),
            runs_on: "${{ matrix.os }}".into(),
            strategy: Some(Strategy {
                fail_fast: false,
                matrix: BTreeMap::from([("os".to_string(), project.os.clone())]),
            }),
            steps: vec![
                uses(CHECKOUT),
                toolchain("stable", None),
                uses(CACHE),
                run("test", format!("cargo test{ws}")),
            ],
        },
    );
    if let Some(msrv) = &project.msrv {
        jobs.insert(
            "msrv".to_string(),
            Job {
                name: format!("MSRV {msrv}"),
                runs_on: "ubuntu-latest".into(),
                strategy: None,
                steps: vec![
                    uses(CHECKOUT),
                    toolchain(msrv, None),
                    uses(CACHE),
                    run("check", format!("cargo check{ws} --all-targets")),
                ],
            },
        );
    }
    let branches = serde_yaml_ng::to_value(BTreeMap::from([("branches", vec!["main"])]))
        .expect("serializable");
    let workflow = Workflow {
        name: format!("CI ({})", project.name),
        on: BTreeMap::from([
            ("push".to_string(), branches),
            (
                "pull_request".to_string(),
                Value::Mapping(Default::default()),
            ),
        ]),
        env: BTreeMap::from([("CARGO_TERM_COLOR".to_string(), "always".to_string())]),
        jobs,
    };
    serde_yaml_ng::to_string(&workflow).expect("serializable")
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
    let doc: Value = match serde_yaml_ng::from_str(yaml) {
        Ok(doc) => doc,
        Err(e) => return vec![format!("invalid YAML: {e}")],
    };
    let mut problems = Vec::new();
    if doc.get("on").is_none() {
        problems.push("missing `on`".to_string());
    }
    let Some(jobs) = doc
        .get("jobs")
        .and_then(Value::as_mapping)
        .filter(|j| !j.is_empty())
    else {
        problems.push("no jobs".to_string());
        problems.sort();
        return problems;
    };
    let (mut fmt_check, mut test) = (false, false);
    for (name, job) in jobs {
        let name = name.as_str().unwrap_or("?");
        if job.get("runs-on").is_none() {
            problems.push(format!("job {name}: missing runs-on"));
        }
        let steps = job
            .get("steps")
            .and_then(Value::as_sequence)
            .cloned()
            .unwrap_or_default();
        if steps.is_empty() {
            problems.push(format!("job {name}: no steps"));
            continue;
        }
        let first_uses = steps[0].get("uses").and_then(Value::as_str).unwrap_or("");
        if !first_uses.starts_with("actions/checkout") {
            problems.push(format!("job {name}: first step must be actions/checkout"));
        }
        for step in &steps {
            if let Some(action) = step.get("uses").and_then(Value::as_str)
                && !action.contains('@')
            {
                problems.push(format!("job {name}: unpinned action {action}"));
            }
            let command = step.get("run").and_then(Value::as_str).unwrap_or("");
            if command.contains("cargo clippy") && !command.contains("-D warnings") {
                problems.push(format!("job {name}: clippy without -D warnings"));
            }
            fmt_check |= command.contains("cargo fmt") && command.contains("--check");
            test |= command.contains("cargo test");
        }
    }
    if !fmt_check {
        problems.push("no cargo fmt --check".to_string());
    }
    if !test {
        problems.push("no cargo test".to_string());
    }
    problems.sort();
    problems
}
