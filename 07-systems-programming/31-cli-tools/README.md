# 31 · CLI Tools

## Overview

Rust is a favourite for command-line tools (ripgrep, fd, bat, exa, starship)
because it produces a single fast binary with no runtime, and because clap
makes a professional interface -- help, validation, subcommands, shell
completions -- a matter of annotating a struct. This module builds four
tools: a grep clone, a log viewer, a task manager with a JSON store, and a
terminal system monitor with ratatui.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | `minigrep` | clap derive; stdout vs stderr; exit codes; walking directories |
| 2 | `logview` | Subcommands; parsing lines; filters; streaming |
| 3 | `tasks` | A JSON store; env-var config; atomic saves |
| 4 | monitor | A TUI: state, input, render; testing with `TestBackend` |
| Bonus | completions | `clap_complete`; `NO_COLOR` |

## Key Concepts

### clap derive

```rust
#[derive(Parser)]
#[command(name = "minigrep", version, about)]
struct GrepArgs {
    pattern: String,                       // positional
    #[arg(required = true)]
    paths: Vec<PathBuf>,                   // one or more
    #[arg(short, long)]
    ignore_case: bool,                     // -i / --ignore-case
    #[arg(short = 'm', long)]
    max_count: Option<usize>,              // -m 3
    #[arg(long, value_enum, default_value_t = ColorChoice::Never)]
    color: ColorChoice,                    // --color always|never|auto
}
```

`--help`, `--version`, error messages and completions come for free.
`#[command(subcommand)]`, `#[command(flatten)]` and `#[arg(env = "...")]`
cover the rest.

### The contract

| Channel | Carries |
|---|---|
| stdout | results (pipeable) |
| stderr | diagnostics |
| exit code | 0 success / match, 1 "no" (no match), 2+ errors |
| args > env > default | configuration |

### Testable tools

```rust
pub fn run(args: &GrepArgs, out: &mut impl Write, err: &mut impl Write, is_terminal: bool) -> i32
```

`main` passes `io::stdout()`; tests pass a `Vec<u8>`. The same for TUIs:
`render(frame, &app)` draws into a real terminal or into `TestBackend`.

### Atomic saves

Write `store.json.tmpXXXX` next to the store, then `rename` it over
`store.json`: a crash leaves either the old or the new file, never half of one.

## Common Pitfalls

1. **Printing errors to stdout** -- breaks every pipeline
2. **Exit code 0 on failure** -- scripts can't tell
3. **`unwrap` on user input** -- a panic message is not an error message
4. **Reading whole files into memory** -- stream with `BufRead::lines`
5. **Writing the store in place** -- a crash corrupts it
6. **Colour codes in pipes** -- check `IsTerminal` and `NO_COLOR`
7. **Leaving the terminal in raw mode after a panic** -- `ratatui::init` installs a hook that restores it

## Running This Module

```bash
cargo run  -p m31-cli-tools -- 1                     # your code (1-4, bonus, all)
cargo test -p m31-cli-tools-tests --features mine    # test your code
cargo run  -p m31-cli-tools-solution -- all          # demos of all four tools
cargo run  -p m31-cli-tools-solution -- grep -rn fn 07-systems-programming/31-cli-tools/solution/src
cargo run  -p m31-cli-tools-solution -- monitor      # the real TUI (q quits)
cargo test -p m31-cli-tools-tests                    # 25 tests against the solution
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the exercises in
[the phase README](../README.md) (a grep-like search tool, a system monitor
TUI, a task manager, a log viewer).

- **Not covered**: progress bars (`indicatif`), interactive prompts
  (`dialoguer`) and signal handling -- module 34 handles signals.
- **The monitor's live mode isn't tested** (it needs a terminal); its state,
  input handling and rendering are, through `TestBackend`.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[32 · Network Programming](../32-network-programming/)
