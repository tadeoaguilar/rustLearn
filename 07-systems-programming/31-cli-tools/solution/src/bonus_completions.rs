//! Bonus: shell completions and colour detection.
//!
//! clap already knows every subcommand and flag, so it can write the
//! completion script for each shell; users install it once
//! (`minigrep --completions zsh > _minigrep`). And colour should follow the
//! user's choice -- or, on `auto`, whether output is a terminal (and the
//! `NO_COLOR` convention).

use clap::CommandFactory;
use clap_complete::{Shell, generate};

use crate::ex01_grep::{ColorChoice, GrepArgs};

/// The completion script for `minigrep` in `shell`.
pub fn completions(shell: Shell) -> String {
    let mut command = GrepArgs::command();
    let mut out = Vec::new();
    generate(shell, &mut command, "minigrep", &mut out);
    String::from_utf8(out).expect("completion scripts are UTF-8")
}

/// Whether to colour output: `always`/`never` as asked; `auto` only on a
/// terminal, and never if `NO_COLOR` is set (to anything non-empty).
pub fn should_color(choice: ColorChoice, is_terminal: bool, no_color_env: Option<&str>) -> bool {
    match choice {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => is_terminal && no_color_env.is_none_or(str::is_empty),
    }
}
