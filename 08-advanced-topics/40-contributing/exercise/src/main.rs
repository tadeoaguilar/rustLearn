// 40-contributing -- YOUR WORKSPACE.
//
// This runner is ready to use: it calls the functions in src/ that you
// fill in. Until you implement a part it stops with "not yet implemented".
//
//     cargo run -p m40-contributing -- <1-4|bonus|all>
//     cargo run -p m40-contributing -- semver <old.rs> <new.rs> <current-version>

use m40_contributing::{
    bonus_commit_lint, ex01_upstream as upstream, ex02_semver as semver,
    ex03_changelog as changelog, ex04_ci as ci,
};

const V1: &str = r#"
pub struct Config { pub name: String, pub retries: u32 }
pub enum Level { Info, Warn }
pub trait Store { fn get(&self, key: &str) -> Option<String>; }
pub fn connect(config: &Config) -> bool { true }
impl Config { pub fn new(name: &str) -> Self { todo!() } }
"#;

const V2: &str = r#"
pub struct Config { pub name: String, pub retries: u32, pub timeout_ms: u64 }
#[non_exhaustive]
pub enum Level { Info, Warn, Error }
pub trait Store {
    fn get(&self, key: &str) -> Option<String>;
    fn contains(&self, key: &str) -> bool { self.get(key).is_some() }
}
pub fn connect(config: &Config) -> bool { true }
pub fn disconnect() {}
impl Config { pub fn new(name: &str) -> Self { todo!() } }
"#;

fn ex1() {
    println!("--- 1: the four issues, fixed");
    println!(
        "#1 truncate(\"año nuevo\", 3) = {:?}",
        upstream::truncate("año nuevo", 3)
    );
    println!(
        "#2 format_bytes(1_048_575) = {:?}",
        upstream::format_bytes(1_048_575)
    );
    println!(
        "#3 parse_duration(\"90\") = {:?}, (\"\") = {:?}",
        upstream::parse_duration("90"),
        upstream::parse_duration("")
    );
    println!(
        "#4 parse_duration(\"99999999999999999999h\") = {:?}",
        upstream::parse_duration("99999999999999999999h")
    );
    println!(
        "    and still: parse_duration(\"1h30m\") = {:?}",
        upstream::parse_duration("1h30m")
    );
}

fn ex2() {
    println!("--- 2: semver check, 1.4.2 -> ?");
    let changes = semver::diff(
        &semver::public_api(V1).unwrap(),
        &semver::public_api(V2).unwrap(),
    );
    for change in &changes {
        println!("  {change}");
    }
    let bump = semver::required_bump(&changes);
    println!(
        "required: {bump:?} -> {}  (and from 0.3.1: {})",
        semver::next_version("1.4.2", bump).unwrap(),
        semver::next_version("0.3.1", bump).unwrap()
    );
}

fn ex3() {
    println!("--- 3: changelog");
    let messages = [
        "feat(parser): support comments",
        "fix: handle empty input",
        "docs: typo in README",
        "feat(api)!: rename `run` to `execute`",
        "perf: avoid a copy in the lexer\n\nCloses #12.",
    ];
    let commits: Vec<_> = messages
        .iter()
        .map(|m| changelog::parse_commit(m).unwrap())
        .collect();
    let bump = changelog::bump_for(&commits).unwrap();
    let existing = "# Changelog\n\n## [Unreleased]\n\n## [1.1.0] - 2026-09-01\n\n### Added\n\n- first release\n\n[Unreleased]: https://github.com/o/r/compare/v1.1.0...HEAD\n[1.1.0]: https://github.com/o/r/releases/tag/v1.1.0\n";
    let version =
        semver::next_version(&changelog::latest_version(existing).unwrap(), bump).unwrap();
    println!(
        "{}",
        changelog::insert_release(existing, &version, "2026-10-05", &commits).unwrap()
    );
}

fn ex4() {
    println!("--- 4: CI");
    let project = ci::Project {
        name: "demo".into(),
        msrv: Some("1.87".into()),
        os: vec!["ubuntu-latest".into(), "macos-latest".into()],
        workspace: true,
    };
    let yaml = ci::workflow(&project);
    println!("{yaml}");
    println!(
        "review of the generated workflow: {:?}",
        ci::validate(&yaml)
    );
    println!(
        "review of a sloppy one: {:?}",
        ci::validate(
            "on: push\njobs:\n  ci:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: dtolnay/rust-toolchain\n      - run: cargo clippy\n"
        )
    );
}

fn bonus() {
    println!("--- bonus: commit lint");
    for message in ["fix: handle empty input", "Fixed the bug.\nIt was bad", ""] {
        println!(
            "{message:?} -> {:?}",
            bonus_commit_lint::lint_commit(message)
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("1") => ex1(),
        Some("2") => ex2(),
        Some("3") => ex3(),
        Some("4") => ex4(),
        Some("bonus") => bonus(),
        Some("all") => {
            ex1();
            ex2();
            ex3();
            ex4();
            bonus();
        }
        Some("semver") if args.len() == 4 => {
            let read = |p: &str| std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{p}: {e}"));
            let old = semver::public_api(&read(&args[1])).expect("old version parses");
            let new = semver::public_api(&read(&args[2])).expect("new version parses");
            let changes = semver::diff(&old, &new);
            changes.iter().for_each(|c| println!("{c}"));
            let bump = semver::required_bump(&changes);
            println!(
                "{bump:?}: {} -> {}",
                args[3],
                semver::next_version(&args[3], bump).unwrap_or_else(|e| e)
            );
        }
        _ => println!(
            "40-contributing -- your workspace\n\n  cargo run -p m40-contributing -- <1-4|bonus|all>\n  ... -- semver <old.rs> <new.rs> <current-version>"
        ),
    }
}
