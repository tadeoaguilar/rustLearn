//! Exercise 6: a Dockerfile linter.
//!
//! Checking the rules yourself is the best way to learn them. Each rule is a
//! mistake that's easy to make and costs size, build time or security.
//! (`hadolint` is the real-world tool.)

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    /// 1-based line where the instruction starts.
    pub line: usize,
    /// Upper-case: FROM, RUN, COPY...
    pub keyword: String,
    /// Everything after the keyword, continuation lines joined with spaces.
    pub args: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rule {
    /// `FROM rust` or `FROM rust:latest`: the build changes under you.
    UnpinnedBaseImage,
    /// One stage: the compiler and sources end up in the image you ship.
    SingleStage,
    /// No `USER` (or `USER root`) in the final stage.
    RunsAsRoot,
    /// No `HEALTHCHECK` in the final stage.
    NoHealthcheck,
    /// `cargo build` with neither cargo-chef nor a cache mount: every source
    /// change recompiles every dependency.
    NoDependencyCaching,
    /// `ADD` of local files: `COPY` says what it does (ADD also unpacks tars
    /// and downloads URLs).
    AddForLocalFiles,
    /// `CMD app` / `ENTRYPOINT app` (shell form): `/bin/sh` becomes PID 1
    /// and SIGTERM never reaches the app; and `scratch` has no shell.
    ShellFormCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub line: usize,
    pub rule: Rule,
    pub message: String,
}

/// Splits a Dockerfile into instructions: skips blank lines and comments,
/// joins lines ending in `\`.
pub fn parse(text: &str) -> Vec<Instruction> {
    let mut out = Vec::new();
    let mut pending: Option<(usize, String)> = None;
    for (i, raw) in text.lines().enumerate() {
        let trimmed = raw.trim();
        // Comments are allowed between continuation lines too.
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let (body, continues) = match trimmed.strip_suffix('\\') {
            Some(b) => (b.trim_end(), true),
            None => (trimmed, false),
        };
        let (line, mut acc) = pending.take().unwrap_or((i + 1, String::new()));
        if !acc.is_empty() {
            acc.push(' ');
        }
        acc.push_str(body);
        if continues {
            pending = Some((line, acc));
        } else {
            out.push(split(line, &acc));
        }
    }
    if let Some((line, acc)) = pending {
        out.push(split(line, &acc));
    }
    out
}

fn split(line: usize, text: &str) -> Instruction {
    let (keyword, args) = text.split_once(char::is_whitespace).unwrap_or((text, ""));
    Instruction {
        line,
        keyword: keyword.to_ascii_uppercase(),
        args: args.trim().to_string(),
    }
}

/// The image of a FROM line, skipping flags like `--platform=...`.
fn from_image(args: &str) -> &str {
    args.split_whitespace()
        .find(|t| !t.starts_with("--"))
        .unwrap_or("")
}

/// The stage name: `FROM image AS name`.
fn from_alias(args: &str) -> Option<String> {
    let tokens: Vec<&str> = args.split_whitespace().collect();
    tokens
        .iter()
        .position(|t| t.eq_ignore_ascii_case("as"))
        .and_then(|i| tokens.get(i + 1))
        .map(|s| s.to_lowercase())
}

fn is_unpinned(image: &str) -> bool {
    if image.contains('@') {
        return false; // pinned by digest
    }
    // The tag is after the last '/': "localhost:5000/app" has no tag.
    let last = image.rsplit('/').next().unwrap_or(image);
    match last.split_once(':') {
        None => true,
        Some((_, tag)) => tag == "latest",
    }
}

pub fn lint(text: &str) -> Vec<Finding> {
    let instructions = parse(text);
    let mut findings = Vec::new();
    let mut add = |line, rule, message: &str| {
        findings.push(Finding {
            line,
            rule,
            message: message.to_string(),
        })
    };

    // Split into stages at each FROM.
    let mut stages: Vec<&[Instruction]> = Vec::new();
    let starts: Vec<usize> = instructions
        .iter()
        .enumerate()
        .filter(|(_, i)| i.keyword == "FROM")
        .map(|(n, _)| n)
        .collect();
    for (k, &start) in starts.iter().enumerate() {
        let end = starts.get(k + 1).copied().unwrap_or(instructions.len());
        stages.push(&instructions[start..end]);
    }

    let mut aliases: Vec<String> = Vec::new();
    for stage in &stages {
        let from = &stage[0];
        let image = from_image(&from.args);
        let is_stage_ref = aliases.contains(&image.to_lowercase());
        if image != "scratch" && !is_stage_ref && is_unpinned(image) {
            add(
                from.line,
                Rule::UnpinnedBaseImage,
                &format!("pin a version: `{image}` changes under you"),
            );
        }
        if let Some(alias) = from_alias(&from.args) {
            aliases.push(alias);
        }

        let uses_chef = stage
            .iter()
            .any(|i| i.keyword == "RUN" && i.args.contains("cargo chef cook"));
        for i in stage.iter() {
            match i.keyword.as_str() {
                "RUN"
                    if i.args.contains("cargo build")
                        && !uses_chef
                        && !i.args.contains("--mount=type=cache") =>
                {
                    add(
                        i.line,
                        Rule::NoDependencyCaching,
                        "every source change rebuilds all dependencies: use cargo-chef or a cache mount",
                    )
                }
                "ADD" => {
                    let remote = i
                        .args
                        .split_whitespace()
                        .any(|t| t.starts_with("http://") || t.starts_with("https://"));
                    if !remote {
                        add(i.line, Rule::AddForLocalFiles, "use COPY for local files");
                    }
                }
                "CMD" | "ENTRYPOINT" if !i.args.starts_with('[') => add(
                    i.line,
                    Rule::ShellFormCommand,
                    "use the exec form, [\"/app\", ...]: in shell form, SIGTERM goes to /bin/sh",
                ),
                _ => {}
            }
        }
    }

    if let Some(last) = stages.last() {
        let from_line = last[0].line;
        if stages.len() == 1 {
            add(
                from_line,
                Rule::SingleStage,
                "a single stage ships the compiler: build in one stage, copy the binary into a small one",
            );
        }
        // The last USER wins; none at all means root.
        let user = last.iter().rfind(|i| i.keyword == "USER");
        let runs_as_root = user.is_none_or(|u| {
            let name = u.args.split(':').next().unwrap_or("");
            name == "root" || name == "0"
        });
        if runs_as_root {
            add(
                user.map_or(from_line, |u| u.line),
                Rule::RunsAsRoot,
                "run as an unprivileged user, e.g. USER 10001:10001",
            );
        }
        if !last.iter().any(|i| i.keyword == "HEALTHCHECK") {
            add(
                from_line,
                Rule::NoHealthcheck,
                "add a HEALTHCHECK so `docker ps` and Compose know when the app is ready",
            );
        }
    }

    findings.sort_by_key(|f| (f.line, f.rule));
    findings
}
