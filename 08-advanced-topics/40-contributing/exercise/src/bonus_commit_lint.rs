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
    todo!("Bonus")
}
