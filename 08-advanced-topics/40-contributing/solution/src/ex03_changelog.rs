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
    let header = message.lines().next().unwrap_or("").trim_end();
    let (head, description) = header
        .split_once(": ")
        .ok_or_else(|| format!("no `type: description` in {header:?}"))?;
    let description = description.trim();
    if description.is_empty() {
        return Err("empty description".into());
    }
    let (head, bang) = match head.strip_suffix('!') {
        Some(h) => (h, true),
        None => (head, false),
    };
    let (kind, scope) = match head.split_once('(') {
        Some((kind, rest)) => {
            let scope = rest
                .strip_suffix(')')
                .ok_or_else(|| format!("unclosed scope in {header:?}"))?;
            if scope.is_empty() {
                return Err("empty scope".into());
            }
            (kind, Some(scope.to_string()))
        }
        None => (head, None),
    };
    if kind.is_empty() || !kind.chars().all(|c| c.is_ascii_lowercase()) {
        return Err(format!("invalid type {kind:?}"));
    }
    let footer_breaking = message
        .lines()
        .skip(1)
        .any(|l| l.starts_with("BREAKING CHANGE: ") || l.starts_with("BREAKING-CHANGE: "));
    Ok(Commit {
        kind: kind.to_string(),
        scope,
        breaking: bang || footer_breaking,
        description: description.to_string(),
    })
}

/// The bump the commits call for: any breaking -> Major, any `feat` ->
/// Minor, any `fix`/`perf` -> Patch; `None` if nothing user-visible changed
/// (only `docs`, `chore`, `ci`, `test`, ...).
pub fn bump_for(commits: &[Commit]) -> Option<Bump> {
    commits
        .iter()
        .filter_map(|c| match (c.breaking, c.kind.as_str()) {
            (true, _) => Some(Bump::Major),
            (_, "feat") => Some(Bump::Minor),
            (_, "fix" | "perf") => Some(Bump::Patch),
            _ => None,
        })
        .max()
}

fn entry(commit: &Commit) -> String {
    let scope = commit
        .scope
        .as_ref()
        .map(|s| format!("**{s}:** "))
        .unwrap_or_default();
    let breaking = if commit.breaking {
        "**BREAKING:** "
    } else {
        ""
    };
    format!("- {breaking}{scope}{}", commit.description)
}

/// A release section. Breaking commits go under `### Changed` (first, marked
/// `**BREAKING:**`), then `feat` -> Added, `perf`/`refactor` -> Changed,
/// `fix` -> Fixed; others are left out. Sections in the order Added,
/// Changed, Fixed; empty ones omitted; commits in the given order.
pub fn release_section(version: &str, date: &str, commits: &[Commit]) -> String {
    let added: Vec<String> = commits
        .iter()
        .filter(|c| !c.breaking && c.kind == "feat")
        .map(entry)
        .collect();
    let changed: Vec<String> = commits
        .iter()
        .filter(|c| c.breaking)
        .chain(
            commits
                .iter()
                .filter(|c| !c.breaking && matches!(c.kind.as_str(), "perf" | "refactor")),
        )
        .map(entry)
        .collect();
    let fixed: Vec<String> = commits
        .iter()
        .filter(|c| !c.breaking && c.kind == "fix")
        .map(entry)
        .collect();
    let mut out = format!("## [{version}] - {date}\n");
    for (title, entries) in [("Added", added), ("Changed", changed), ("Fixed", fixed)] {
        if !entries.is_empty() {
            out.push_str(&format!("\n### {title}\n\n{}\n", entries.join("\n")));
        }
    }
    out
}

/// The most recent released version in a changelog (the first `## [x.y.z]`
/// heading that isn't Unreleased).
pub fn latest_version(changelog: &str) -> Option<String> {
    changelog.lines().find_map(|l| {
        let version = l.strip_prefix("## [")?.split(']').next()?;
        (version != "Unreleased").then(|| version.to_string())
    })
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
    if changelog
        .lines()
        .any(|l| l.starts_with(&format!("## [{version}]")))
    {
        return Err(format!("version {version} is already in the changelog"));
    }
    let previous = latest_version(changelog);
    let lines: Vec<&str> = changelog.lines().collect();
    let unreleased = lines
        .iter()
        .position(|l| l.starts_with("## [Unreleased]"))
        .ok_or("no `## [Unreleased]` section")?;
    // the section ends at the next heading of level 2 or a link definition
    let insert_at = lines[unreleased + 1..]
        .iter()
        .position(|l| l.starts_with("## ") || l.starts_with('['))
        .map_or(lines.len(), |i| unreleased + 1 + i);
    let section = release_section(version, date, commits);
    let mut out: Vec<String> = lines[..insert_at].iter().map(|l| l.to_string()).collect();
    if out.last().is_some_and(|l| !l.is_empty()) {
        out.push(String::new());
    }
    out.extend(section.lines().map(str::to_string));
    out.push(String::new());
    for line in &lines[insert_at..] {
        match line.strip_prefix("[Unreleased]: ") {
            Some(url) => {
                let base = url.split("/compare/").next().unwrap_or(url);
                out.push(format!("[Unreleased]: {base}/compare/v{version}...HEAD"));
                match &previous {
                    Some(prev) => {
                        out.push(format!("[{version}]: {base}/compare/v{prev}...v{version}"))
                    }
                    None => out.push(format!("[{version}]: {base}/releases/tag/v{version}")),
                }
            }
            None => out.push(line.to_string()),
        }
    }
    let mut text = out.join("\n");
    text.push('\n');
    Ok(text)
}
