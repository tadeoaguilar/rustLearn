# Exercises: CLI Tools

A command-line tool's interface is its arguments, its stdout, its stderr and
its exit code -- and scripts depend on all four. `clap` turns a struct into
an argument parser with help, validation and completions; the rest is
discipline: results to stdout, diagnostics to stderr, meaningful exit codes,
configuration from flags *and* the environment, state saved safely.

**Setup**: `clap` (derive), `regex`, `serde_json`, `ratatui`, `sysinfo`,
`tempfile`. Every exercise is tested in-process -- each tool's `run`
function takes its output streams as parameters -- so no test spawns a
process or needs a terminal. The real tools run from `main.rs`:
`cargo run -p m31-cli-tools -- grep ...`.

In the exercise files, the clap structs have their fields but not their
`#[arg(...)]` attributes (marked `TODO`): declaring the interface is part of
each exercise.

---

## Exercise 1: `minigrep`

**Difficulty**: Easy
**Time**: 1 hour

**Learning Objectives**:
- Declare a CLI with clap derive: positionals, short/long flags, options, value enums
- Follow grep's conventions for output and exit codes
- Walk directories

1. `GrepArgs`: `PATTERN PATHS...` (at least one path), `-i/--ignore-case`,
   `-n/--line-number`, `-v/--invert-match`, `-c/--count`, `-r/--recursive`,
   `-F/--fixed-strings`, `-m/--max-count NUM`, `--color always|never|auto`
   (default never).
2. `build_regex` (escape with `-F`, case-insensitive with `-i`).
3. `search_lines(reader, re, invert, max_count)` -> `(line number, line)`.
4. `collect_files`: a directory without `-r` is an error; with it, walk in
   sorted order, skipping hidden entries.
5. `highlight`: matches in ANSI bold red (`\x1b[1;31m` ... `\x1b[0m`).
6. `run`: prefix `file:` when searching several files (or `-r`), `n:` with
   `-n`; `-c` prints counts; errors to `err` as `minigrep: ...`. Exit 0 if
   any line matched, 1 if none, 2 if any error occurred.

**Question**: why do errors go to stderr and not stdout?

---

## Exercise 2: `logview`

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- Subcommands, flattened argument groups, `ValueEnum`
- Parse a line format robustly
- Stream large inputs line by line

Lines look like `2026-10-05T12:00:04Z WARN  app::db: slow query`.

1. `Level` (ordered Trace < ... < Error): `FromStr` (case-insensitive,
   `WARNING` too) and `Display` (padding-aware: `{:<5}` must work).
2. `is_timestamp`, `parse_line` (errors: `Timestamp`, `Level`, `Missing`).
3. `LineFilter::matches`: minimum level; target prefix *by path segment*
   (`app::db` matches `app::db::pool`, not `app::dbx`); `since` inclusive,
   `until` exclusive; `contains` case-insensitive.
4. `stats`: counts by level and target, first and last timestamp,
   malformed lines (blank lines don't count).
5. The CLI: `filter [-l LEVEL] [-t TARGET] [--since TS] [--until TS] [-g TEXT] [--json] [FILE]`,
   `stats [FILE]`, `tail [-n N] [-l LEVEL] [FILE]`, and `run` for each.

**Question**: `tail` keeps only N lines in memory. Why does that matter
for a log viewer?

---

## Exercise 3: `tasks`

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- Persist state as JSON with serde
- Take configuration from a flag, an environment variable or a default
- Save atomically

1. `valid_date` (`YYYY-MM-DD`, month 1-12, day 1-31).
2. `Store::load` (a missing file is empty; a corrupt one is `TaskError::Json`);
   `Store::save` -- write a temporary file in the same directory and rename
   it over the store (`tempfile::NamedTempFile::persist`).
3. `add` (trimmed, non-empty title; valid due date), `complete`, `remove`,
   `edit`; ids are never reused.
4. `list`: open tasks first, then priority high to low, then id; filters for
   done, tag and priority. `format_table`.
5. The CLI: `--file` (env `TASKS_FILE`, default `tasks.json`) and the
   subcommands `add TITLE [-p PRIORITY] [-t TAG]... [--due DATE]`,
   `list [-a] [-t TAG] [-p PRIORITY]`, `done ID`, `remove ID`,
   `edit ID [--title T] [-p P]`; `run`.

**Question**: why must the temporary file be in the *same directory* as
the store?

---

## Exercise 4: A System Monitor TUI

**Difficulty**: Medium
**Time**: 1.5 hours

**Learning Objectives**:
- Separate state, input and rendering
- Lay out ratatui widgets: gauges, a sparkline, a table
- Test a TUI with `TestBackend`

`SysinfoSource` (provided) reads the real system; tests use fakes.

1. `Snapshot::cpu_average`, `human_bytes` (`1536` -> `"1.5 KiB"`).
2. `App::update`: ignored while paused; push the CPU average to `history`
   (at most `HISTORY`); keep `selected` in range.
3. `App::on_key`: `q`/Esc quit, `p` pause, `c`/`m`/`i`/`n` sort by
   CPU/memory/pid/name, Up/Down move the selection within the list.
4. `sorted_processes`: CPU and memory descending, pid and name ascending
   (names case-insensitively).
5. `render`: a CPU gauge titled `CPU (N cores)` labelled `12.3%`; a memory
   gauge labelled `used / total`; a `CPU history` sparkline; a
   `Processes (N)` table whose sort column header ends in `▼` and whose
   selected row is reversed; a help line ending in `PAUSED` when paused.

Run it for real: `cargo run -p m31-cli-tools -- monitor`.

---

## Bonus: Completions and Colour

**Difficulty**: Easy
**Time**: 20 minutes

1. `completions(shell)`: `minigrep`'s completion script, from
   `clap_complete::generate`.
2. `should_color(choice, is_terminal, NO_COLOR)`: `auto` colours only on a
   terminal and when `NO_COLOR` is unset or empty (https://no-color.org).
