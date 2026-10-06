//! Bonus: a commit message linter -- what reviewers ask you to fix.
//!
//! Problems, in this order of checks:
//!
//! - `empty message`                     (and nothing else is checked)
//! - `header longer than 72 characters`
//! - `header ends with a period`
//! - `header not in the imperative mood` -- the description (after any
//!   `type(scope): ` prefix) starts with a word ending in `ed` or `ing`, or
//!   with `adds`/`fixes`/`updates`/`removes` (case-insensitive)
//! - `no blank line after the header`    -- a second line that isn't empty
//! - `body line N longer than 72 characters` (N counts from 1 = the header;
//!   URLs-only lines are exempt)

pub fn lint_commit(message: &str) -> Vec<String> {
    let lines: Vec<&str> = message.trim_end().lines().collect();
    let Some(header) = lines.first().filter(|h| !h.trim().is_empty()) else {
        return vec!["empty message".to_string()];
    };
    let mut problems = Vec::new();
    if header.chars().count() > 72 {
        problems.push("header longer than 72 characters".to_string());
    }
    if header.ends_with('.') {
        problems.push("header ends with a period".to_string());
    }
    let description = header.split_once(": ").map_or(*header, |(_, d)| d);
    let first_word = description
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_lowercase();
    let non_imperative = first_word.ends_with("ed")
        || first_word.ends_with("ing")
        || matches!(
            first_word.as_str(),
            "adds" | "fixes" | "updates" | "removes"
        );
    if non_imperative {
        problems.push("header not in the imperative mood".to_string());
    }
    if lines.get(1).is_some_and(|l| !l.is_empty()) {
        problems.push("no blank line after the header".to_string());
    }
    for (i, line) in lines.iter().enumerate().skip(1) {
        let is_url = line.trim().starts_with("http://") || line.trim().starts_with("https://");
        if line.chars().count() > 72 && !is_url {
            problems.push(format!("body line {} longer than 72 characters", i + 1));
        }
    }
    problems
}
