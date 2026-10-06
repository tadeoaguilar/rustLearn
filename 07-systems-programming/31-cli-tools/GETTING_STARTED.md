# Getting Started with 31 · CLI Tools

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m31-cli-tools-solution -- all
```

runs each tool on sample data: `minigrep` searches a temporary directory
(with each exit code), `logview` filters and summarises a sample log,
`tasks` builds a store and prints the JSON on disk, and the monitor renders
one frame of your machine off-screen.

## What Is Already Here

```
31-cli-tools/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/src/              # ← YOUR WORKSPACE (package m31-cli-tools)
│   ├── ex01_grep.rs           #   Ex 1  (clap attributes TODO)
│   ├── ex02_logview.rs        #   Ex 2  (clap attributes TODO)
│   ├── ex03_tasks.rs          #   Ex 3  (clap attributes TODO)
│   ├── ex04_monitor.rs        #   Ex 4  (SysinfoSource provided)
│   ├── bonus_completions.rs   #   bonus
│   └── main.rs                #   provided: demos and the real tools
├── solution/                  # ← REFERENCE (m31-cli-tools-solution) + ANSWERS.md
└── tests/                     # ← 25 tests (m31-cli-tools-tests)
```

## The Commands You Need

```bash
cargo test -p m31-cli-tools-tests --features mine ex1_
cargo test -p m31-cli-tools-tests --features mine
cargo test -p m31-cli-tools-tests                    # the solution: always green

# your tools, for real
cargo run -q -p m31-cli-tools -- grep -n -i todo 07-systems-programming/31-cli-tools/exercise/src/ex01_grep.rs; echo "exit $?"
cargo run -q -p m31-cli-tools -- grep --help
printf '2026-10-05T12:00:00Z WARN a: b\n' | cargo run -q -p m31-cli-tools -- logview filter -l warn
TASKS_FILE=/tmp/t.json cargo run -q -p m31-cli-tools -- tasks add "Try it"
cargo run -q -p m31-cli-tools -- monitor
cargo run -q -p m31-cli-tools -- completions zsh > _minigrep
```

## If You Get Stuck

1. **`Argument 'ignore_case' is positional and it must take a value but action is SetTrue`** -- that's the skeleton before you add `#[arg(short, long)]`: without it, a field is a positional argument. clap checks the whole definition the first time it parses (in debug builds), so every test that parses fails until the attributes are right.
2. **"Short option names must be unique"** -- clap generates `-h` and `-V` itself; two fields can't share a letter either (`short = 'x'` picks one).
3. **`{:<5}` doesn't pad your `Level`** -- `Display` must use `f.pad(...)`, not `write!`.
4. **`list` order looks random** -- sort by `(done, priority descending, id)` explicitly.
5. **`TestBackend` buffer is empty** -- call `terminal.draw(|f| render(f, &app))` before reading `backend().buffer()`.
6. **The monitor shows 0% CPU** -- sysinfo needs two samples some time apart; the first snapshot is always 0.
7. Compare with `solution/src/` -- same file and function names.
