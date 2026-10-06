use std::time::Duration;

use crate::sut::bonus_commit_lint::lint_commit;
use crate::sut::ex01_upstream::{ParseError, format_bytes, parse_duration, truncate};
use crate::sut::ex02_semver::{
    Bump, Change, ChangeKind, diff, next_version, public_api, required_bump,
};
use crate::sut::ex03_changelog::{
    Commit, bump_for, insert_release, latest_version, parse_commit, release_section,
};
use crate::sut::ex04_ci::{Project, validate, workflow};

// ---------------------------------------------------------------- Exercise 1
// The existing behaviour (passes before and after your fixes) ...

#[test]
fn ex1_existing_behaviour() {
    assert_eq!(truncate("hello", 10), "hello");
    assert_eq!(truncate("hello world", 6), "hello…");
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(1023), "1023 B");
    assert_eq!(format_bytes(1536), "1.5 KiB");
    assert_eq!(format_bytes(5 * 1024 * 1024 * 1024), "5.0 GiB");
    assert_eq!(parse_duration("1h30m"), Ok(Duration::from_secs(5400)));
    assert_eq!(parse_duration("2h 5s"), Ok(Duration::from_secs(7205)));
    assert_eq!(parse_duration("90s"), Ok(Duration::from_secs(90)));
    assert_eq!(parse_duration("1x"), Err(ParseError::InvalidChar('x')));
}

// ... and one regression test per issue (fails until it's fixed).

#[test]
fn ex1_issue_1_truncate_non_ascii() {
    assert_eq!(
        truncate("año nuevo", 3),
        "añ…",
        "cut by characters, not bytes"
    );
    assert_eq!(
        truncate("日本語", 3),
        "日本語",
        "3 characters fit, though they're 9 bytes"
    );
    assert_eq!(truncate("🦀🦀🦀🦀", 2), "🦀…");
    assert_eq!(truncate("abc", 0), "", "max 0 must not underflow");
}

#[test]
fn ex1_issue_2_unit_boundary() {
    assert_eq!(format_bytes(1_048_575), "1.0 MiB", "not 1024.0 KiB");
    assert_eq!(
        format_bytes(1024 * 1024 - 52),
        "1023.9 KiB",
        "just below the rounding edge"
    );
    assert_eq!(
        format_bytes(1024u64.pow(4) * 2000),
        "2000.0 TiB",
        "no unit above TiB"
    );
}

#[test]
fn ex1_issue_3_malformed_durations() {
    assert_eq!(parse_duration("90"), Err(ParseError::MissingUnit));
    assert_eq!(parse_duration("1h30"), Err(ParseError::MissingUnit));
    assert_eq!(parse_duration(""), Err(ParseError::Empty));
    assert_eq!(parse_duration("   "), Err(ParseError::Empty));
    assert_eq!(
        parse_duration("h"),
        Err(ParseError::InvalidChar('h')),
        "a unit needs a number"
    );
}

#[test]
fn ex1_issue_4_overflow() {
    assert_eq!(
        parse_duration("99999999999999999999h"),
        Err(ParseError::Overflow)
    );
    assert_eq!(
        parse_duration("5124095576030432h"),
        Err(ParseError::Overflow),
        "the number fits, the seconds don't"
    );
    assert_eq!(
        parse_duration(&format!("{}s1s", u64::MAX)),
        Err(ParseError::Overflow),
        "the total doesn't"
    );
    assert_eq!(
        parse_duration(&format!("{}s", u64::MAX)),
        Ok(Duration::from_secs(u64::MAX))
    );
}

// ---------------------------------------------------------------- Exercise 2

const V1: &str = r#"
pub struct Config { pub name: String, pub retries: u32 }
pub struct Opaque { inner: u8 }
pub enum Level { Info, Warn }
#[non_exhaustive]
pub enum Kind { A }
pub trait Store { fn get(&self, key: &str) -> Option<String>; }
pub fn connect(config: &Config) -> bool { true }
pub fn removed_later() {}
fn private() {}
pub(crate) fn crate_only() {}
#[doc(hidden)]
pub fn hidden() {}
impl Config {
    pub fn new(name: &str) -> Self { todo!() }
    fn helper(&self) {}
}
impl std::fmt::Display for Config { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) } }
pub mod util { pub fn helper(x: u32) -> u32 { x } }
"#;

#[test]
fn ex2_public_api() {
    let api = public_api(V1).unwrap();
    let keys: Vec<&str> = api.keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        [
            "enum Kind",
            "enum Level",
            "field Config.name",
            "field Config.retries",
            "fn Config::new",
            "fn connect",
            "fn removed_later",
            "fn util::helper",
            "impl std :: fmt :: Display for Config",
            "struct Config",
            "struct Opaque",
            "trait Store",
            "trait_fn Store::get",
            "variant Kind::A",
            "variant Level::Info",
            "variant Level::Warn",
        ]
    );
    assert_eq!(api["struct Config"], "exhaustive constructible");
    assert_eq!(api["struct Opaque"], "exhaustive");
    assert_eq!(api["enum Kind"], "non_exhaustive");
    assert_eq!(api["field Config.retries"], "u32");
    assert!(api["trait_fn Store::get"].ends_with("[required]"));
    assert!(
        api["fn connect"].contains("config : & Config"),
        "{}",
        api["fn connect"]
    );
    assert!(public_api("pub fn broken(").is_err());
}

fn changes(old: &str, new: &str) -> Vec<(String, ChangeKind, Bump)> {
    diff(&public_api(old).unwrap(), &public_api(new).unwrap())
        .into_iter()
        .map(|c| (c.key, c.kind, c.bump))
        .collect()
}

fn one(key: &str, kind: ChangeKind, bump: Bump) -> Vec<(String, ChangeKind, Bump)> {
    vec![(key.to_string(), kind, bump)]
}

#[test]
fn ex2_classification() {
    use Bump::*;
    use ChangeKind::*;
    // removals and signature changes break
    assert_eq!(changes("pub fn a() {}", ""), one("fn a", Removed, Major));
    assert_eq!(
        changes("pub fn a(x: u32) {}", "pub fn a(x: u64) {}"),
        one("fn a", Changed, Major)
    );
    assert_eq!(
        changes("pub struct S { pub x: u8 }", "pub struct S { pub x: u16 }"),
        one("field S.x", Changed, Major)
    );
    // additions don't...
    assert_eq!(changes("", "pub fn b() {}"), one("fn b", Added, Minor));
    assert_eq!(
        changes(
            "pub struct S;",
            "pub struct S; impl S { pub fn new() -> S { S } }"
        ),
        one("fn S::new", Added, Minor)
    );
    // ...unless they break exhaustive matches or struct literals
    assert_eq!(
        changes("pub enum E { A }", "pub enum E { A, B }"),
        one("variant E::B", Added, Major)
    );
    assert_eq!(
        changes(
            "#[non_exhaustive] pub enum E { A }",
            "#[non_exhaustive] pub enum E { A, B }"
        ),
        one("variant E::B", Added, Minor)
    );
    assert_eq!(
        changes(
            "pub struct S { pub a: u8 }",
            "pub struct S { pub a: u8, pub b: u8 }"
        ),
        one("field S.b", Added, Major)
    );
    assert_eq!(
        changes(
            "pub struct S { pub a: u8, c: u8 }",
            "pub struct S { pub a: u8, pub b: u8, c: u8 }"
        ),
        one("field S.b", Added, Minor)
    );
    // trait methods: required breaks implementors, provided doesn't
    assert_eq!(
        changes("pub trait T {}", "pub trait T { fn m(&self); }"),
        one("trait_fn T::m", Added, Major)
    );
    assert_eq!(
        changes("pub trait T {}", "pub trait T { fn m(&self) {} }"),
        one("trait_fn T::m", Added, Minor)
    );
    assert_eq!(
        changes(
            "pub trait T { fn m(&self); }",
            "pub trait T { fn m(&self) {} }"
        ),
        one("trait_fn T::m", Changed, Minor)
    );
    assert_eq!(
        changes("", "pub trait T { fn m(&self); }").len(),
        2,
        "a new trait with a required method is just new"
    );
    assert!(
        changes("", "pub trait T { fn m(&self); }")
            .iter()
            .all(|c| c.2 == Minor)
    );
    // exhaustiveness
    assert_eq!(
        changes("pub enum E { A }", "#[non_exhaustive] pub enum E { A }"),
        one("enum E", Changed, Major)
    );
    assert_eq!(
        changes("#[non_exhaustive] pub enum E { A }", "pub enum E { A }"),
        one("enum E", Changed, Minor)
    );
    // trait impls
    assert_eq!(
        changes(
            "pub struct S; impl Clone for S { fn clone(&self) -> S { S } }",
            "pub struct S;"
        ),
        one("impl Clone for S", Removed, Major)
    );
    // private changes are invisible
    assert!(
        changes(
            "fn a() {} pub struct S { x: u8 }",
            "fn b() {} pub struct S { x: u16, y: u8 }"
        )
        .is_empty()
    );
}

#[test]
fn ex2_bumps_and_versions() {
    assert_eq!(required_bump(&[]), Bump::Patch);
    let c = |bump| Change {
        key: "k".into(),
        kind: ChangeKind::Added,
        bump,
    };
    assert_eq!(
        required_bump(&[c(Bump::Minor), c(Bump::Major), c(Bump::Minor)]),
        Bump::Major
    );
    assert_eq!(c(Bump::Minor).to_string(), "Added k (Minor)");
    assert_eq!(next_version("1.4.2", Bump::Patch).unwrap(), "1.4.3");
    assert_eq!(next_version("1.4.2", Bump::Minor).unwrap(), "1.5.0");
    assert_eq!(next_version("1.4.2", Bump::Major).unwrap(), "2.0.0");
    assert_eq!(
        next_version("0.3.1", Bump::Major).unwrap(),
        "0.4.0",
        "0.y: y is the major"
    );
    assert_eq!(next_version("0.3.1", Bump::Minor).unwrap(), "0.3.2");
    assert_eq!(
        next_version("0.0.7", Bump::Patch).unwrap(),
        "0.0.8",
        "0.0.z: always breaking"
    );
    assert_eq!(next_version("0.0.7", Bump::Major).unwrap(), "0.0.8");
    assert!(next_version("1.2", Bump::Patch).is_err());
    assert!(next_version("1.2.x", Bump::Patch).is_err());
}

// ---------------------------------------------------------------- Exercise 3

fn commit(kind: &str, scope: Option<&str>, breaking: bool, description: &str) -> Commit {
    Commit {
        kind: kind.into(),
        scope: scope.map(Into::into),
        breaking,
        description: description.into(),
    }
}

#[test]
fn ex3_parse_commit() {
    assert_eq!(
        parse_commit("feat(parser): support comments").unwrap(),
        commit("feat", Some("parser"), false, "support comments")
    );
    assert_eq!(
        parse_commit("fix!: drop old API").unwrap(),
        commit("fix", None, true, "drop old API")
    );
    assert_eq!(
        parse_commit("refactor(core)!: split module").unwrap(),
        commit("refactor", Some("core"), true, "split module")
    );
    assert!(
        parse_commit("chore: bump deps\n\nBREAKING CHANGE: MSRV is now 1.87")
            .unwrap()
            .breaking
    );
    assert!(
        parse_commit("chore: x\n\nBREAKING-CHANGE: y")
            .unwrap()
            .breaking
    );
    assert!(
        !parse_commit("docs: mention BREAKING CHANGE: in the guide")
            .unwrap()
            .breaking,
        "only in a footer"
    );
    for bad in [
        "no colon here",
        "Feat: capital type",
        "feat(): empty scope",
        "feat(x: unclosed",
        "feat: ",
        ": no type",
        "feat:no space",
    ] {
        assert!(parse_commit(bad).is_err(), "{bad:?}");
    }
}

#[test]
fn ex3_bump_for() {
    assert_eq!(
        bump_for(&[
            commit("docs", None, false, "x"),
            commit("chore", None, false, "y")
        ]),
        None
    );
    assert_eq!(
        bump_for(&[commit("fix", None, false, "x")]),
        Some(Bump::Patch)
    );
    assert_eq!(
        bump_for(&[commit("perf", None, false, "x")]),
        Some(Bump::Patch)
    );
    assert_eq!(
        bump_for(&[
            commit("fix", None, false, "x"),
            commit("feat", None, false, "y")
        ]),
        Some(Bump::Minor)
    );
    assert_eq!(
        bump_for(&[
            commit("docs", None, true, "x"),
            commit("feat", None, false, "y")
        ]),
        Some(Bump::Major)
    );
}

#[test]
fn ex3_release_section() {
    let commits = [
        commit("fix", None, false, "handle empty input"),
        commit("feat", Some("parser"), false, "support comments"),
        commit("docs", None, false, "typo"),
        commit("perf", None, false, "faster lexer"),
        commit("feat", Some("api"), true, "rename run"),
    ];
    assert_eq!(
        release_section("2.0.0", "2026-10-05", &commits),
        "## [2.0.0] - 2026-10-05\n\n### Added\n\n- **parser:** support comments\n\n### Changed\n\n- **BREAKING:** **api:** rename run\n- faster lexer\n\n### Fixed\n\n- handle empty input\n"
    );
    assert_eq!(
        release_section("1.0.1", "2026-10-05", &commits[..1]),
        "## [1.0.1] - 2026-10-05\n\n### Fixed\n\n- handle empty input\n"
    );
}

const CHANGELOG: &str = "# Changelog\n\nAll notable changes to this project.\n\n## [Unreleased]\n\n## [1.1.0] - 2026-09-01\n\n### Added\n\n- comments\n\n## [1.0.0] - 2026-08-01\n\n- first release\n\n[Unreleased]: https://github.com/o/r/compare/v1.1.0...HEAD\n[1.1.0]: https://github.com/o/r/compare/v1.0.0...v1.1.0\n[1.0.0]: https://github.com/o/r/releases/tag/v1.0.0\n";

#[test]
fn ex3_insert_release() {
    assert_eq!(latest_version(CHANGELOG).as_deref(), Some("1.1.0"));
    assert_eq!(latest_version("# Changelog\n\n## [Unreleased]\n"), None);
    let updated = insert_release(
        CHANGELOG,
        "1.2.0",
        "2026-10-05",
        &[commit("feat", None, false, "new thing")],
    )
    .unwrap();
    assert_eq!(
        updated,
        "# Changelog\n\nAll notable changes to this project.\n\n## [Unreleased]\n\n## [1.2.0] - 2026-10-05\n\n### Added\n\n- new thing\n\n## [1.1.0] - 2026-09-01\n\n### Added\n\n- comments\n\n## [1.0.0] - 2026-08-01\n\n- first release\n\n[Unreleased]: https://github.com/o/r/compare/v1.2.0...HEAD\n[1.2.0]: https://github.com/o/r/compare/v1.1.0...v1.2.0\n[1.1.0]: https://github.com/o/r/compare/v1.0.0...v1.1.0\n[1.0.0]: https://github.com/o/r/releases/tag/v1.0.0\n"
    );
    assert_eq!(latest_version(&updated).as_deref(), Some("1.2.0"));
    assert!(
        insert_release(CHANGELOG, "1.1.0", "2026-10-05", &[]).is_err(),
        "already released"
    );
    assert!(
        insert_release("# Changelog\n\n## [1.0.0] - x\n", "1.1.0", "d", &[]).is_err(),
        "no Unreleased section"
    );
    // a first release, without links
    let first = insert_release(
        "# Changelog\n\n## [Unreleased]\n",
        "0.1.0",
        "2026-10-05",
        &[commit("feat", None, false, "start")],
    )
    .unwrap();
    assert_eq!(
        first,
        "# Changelog\n\n## [Unreleased]\n\n## [0.1.0] - 2026-10-05\n\n### Added\n\n- start\n\n"
    );
}

// ---------------------------------------------------------------- Exercise 4

fn project(msrv: Option<&str>, workspace: bool) -> Project {
    Project {
        name: "demo".into(),
        msrv: msrv.map(Into::into),
        os: vec!["ubuntu-latest".into(), "windows-latest".into()],
        workspace,
    }
}

#[test]
fn ex4_generated_workflow() {
    let yaml = workflow(&project(Some("1.80"), true));
    assert_eq!(validate(&yaml), Vec::<String>::new(), "{yaml}");
    let doc: serde_yaml_ng::Value = serde_yaml_ng::from_str(&yaml).unwrap();
    let jobs = doc["jobs"].as_mapping().unwrap();
    let names: Vec<&str> = jobs.keys().map(|k| k.as_str().unwrap()).collect();
    assert_eq!(names, ["check", "msrv", "test"]);
    assert!(doc["on"]["pull_request"].is_mapping() && doc["on"]["push"]["branches"][0] == "main");
    assert_eq!(
        doc["jobs"]["test"]["strategy"]["matrix"]["os"][1],
        "windows-latest"
    );
    assert_eq!(
        doc["jobs"]["msrv"]["steps"][1]["with"]["toolchain"], "1.80",
        "a string, not the float 1.8"
    );
    assert!(yaml.contains("cargo clippy --workspace --all-targets -- -D warnings"));
    assert!(yaml.contains("cargo test --workspace"));

    let single = workflow(&project(None, false));
    assert!(!single.contains("msrv:") && !single.contains("--workspace"));
    assert!(single.contains("cargo test\n") || single.contains("run: cargo test"));
    assert_eq!(validate(&single), Vec::<String>::new());
}

#[test]
fn ex4_review() {
    assert!(validate("jobs: [unclosed")[0].starts_with("invalid YAML"));
    assert_eq!(validate("name: x\n"), ["missing `on`", "no jobs"]);
    let sloppy = "on: push\njobs:\n  lint:\n    steps:\n      - uses: actions/checkout@v4\n      - uses: dtolnay/rust-toolchain\n      - run: cargo clippy --all-targets\n  empty:\n    runs-on: ubuntu-latest\n    steps: []\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - run: cargo test\n";
    assert_eq!(
        validate(sloppy),
        [
            "job empty: no steps",
            "job lint: clippy without -D warnings",
            "job lint: missing runs-on",
            "job lint: unpinned action dtolnay/rust-toolchain",
            "job test: first step must be actions/checkout",
            "no cargo fmt --check",
        ]
    );
    let fmt_without_check = "on: push\njobs:\n  a:\n    runs-on: x\n    steps:\n      - uses: actions/checkout@v4\n      - run: cargo fmt --all && cargo test\n";
    assert_eq!(validate(fmt_without_check), ["no cargo fmt --check"]);
}

// --------------------------------------------------------------------- Bonus

#[test]
fn bonus_commit_lint() {
    assert!(
        lint_commit("fix(parser): handle empty input\n\nThe lexer assumed one token.\n").is_empty()
    );
    assert_eq!(lint_commit(""), ["empty message"]);
    assert_eq!(lint_commit("  \n\nbody"), ["empty message"]);
    assert_eq!(
        lint_commit("Fixed the bug."),
        [
            "header ends with a period",
            "header not in the imperative mood"
        ]
    );
    assert_eq!(
        lint_commit("feat: adding things"),
        ["header not in the imperative mood"]
    );
    assert_eq!(
        lint_commit("Adds a flag"),
        ["header not in the imperative mood"]
    );
    assert!(
        lint_commit("Add a feed parser").is_empty(),
        "only the first word counts (`feed` would trip the heuristic)"
    );
    let long = format!("fix: {}", "x".repeat(70));
    assert_eq!(lint_commit(&long), ["header longer than 72 characters"]);
    let body = format!(
        "fix: stuff\nno blank line\n\n{}\nhttps://example.com/{}",
        "y".repeat(73),
        "z".repeat(80)
    );
    assert_eq!(
        lint_commit(&body),
        [
            "no blank line after the header",
            "body line 4 longer than 72 characters"
        ]
    );
}
