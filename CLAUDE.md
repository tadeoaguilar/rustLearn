# CLAUDE.md

Guidance for working on this repository. Progress, decisions and the plan for
the remaining phases are in [ROADMAP.md](ROADMAP.md).

## What this repository is

A Rust learning path in eight phases (40 modules) plus project ideas. Every
module has exercises, a reference solution, and tests that run against either
the solution or the learner's own code. The layout mirrors the sibling
repository `~/gitlab/netLearn` (.NET).

## Commands

All from the repository root.

```bash
cargo test                                   # every module, against the solutions -- must stay green
cargo clippy --workspace --all-targets       # must stay warning-free
cargo fmt --all                              # rustfmt (max_width 100 in some modules)

cargo run  -p m07-collections-iterators-solution -- all       # run a module's solution
cargo test -p m07-collections-iterators-tests                 # one module's tests vs the solution
cargo test -p m07-collections-iterators-tests --features mine # vs the exercise crate (fails until implemented)
```

Never run `cargo test --all-features` at the root: it enables `mine` in every
tests crate and tests every unfinished skeleton (some modules then fail to
*compile* by design — 11's `signatures`, 13's per-exercise features).

`10-crud-api-project/` and `06-blockchain-solana/` are excluded from the root
workspace; build them from inside their directories. Phase 6 is its own
workspace (rust-version 1.89 for Anchor) with an in-process Solana runtime in
`sim/`; `./build-sbf.sh` + `cargo test --features sbf` is the optional on-chain
track (needs the Solana toolchain). The root also defines a `min-size` profile
(module 36).

## Module layout

```
<phase>/<NN-name>/
├── README.md            concepts, pitfalls, "Running This Module", notes on exercises.md
├── exercises.md         the exercises (original, or written in the same format)
├── GETTING_STARTED.md   every command for the module, troubleshooting
├── exercise/            package mNN-name            -- learner's crate, todo!() bodies
├── solution/            package mNN-name-solution   -- reference + ANSWERS.md
└── tests/               package mNN-name-tests      -- `sut` = solution, or exercise with `mine`
```

- `solution/src/lib.rs` declares one module per exercise: `ex01_*.rs`,
  `ex02_*.rs`, …, `bonus_*.rs`; each has a `pub fn run()` (or `async fn`).
- `solution/src/main.rs` is a runner: `cargo run -p ... -- <n|bonus|all>`.
- `tests/src/lib.rs`:
  ```rust
  #[cfg(not(feature = "mine"))] pub use mNN_name_solution as sut;
  #[cfg(feature = "mine")]      pub use mNN_name as sut;
  #[cfg(test)] mod exercises;
  ```
  Test names start with `exN_` / `bonus_` so learners can filter.
- Exercise crates set `[lib] doctest = false` (doc examples would hit `todo!()`).
- Workspace members are globbed per phase in the root `Cargo.toml`. A glob
  that matches nothing is an error, so add a phase's globs once its first
  module has all three crates. Proc-macro modules (13, 38) have extra crates
  under `exercise/` and `solution/` (`derive/`, and in 38 `core/` holding the
  testable expansions), matched by their own globs.
- Shared dependency versions live in `[workspace.dependencies]`.

## Adding a module

1. Write `solution/` first (Cargo.toml, `src/`), get `cargo run -p ...-solution -- all` working.
2. `python3 tools/scaffold.py <module_dir> <mNN-name> <NN-name>` — creates
   `exercise/` (skeletons via `tools/skeleton.py`) and the `tests/` wiring.
   It doesn't overwrite existing files (`FORCE=1` to regenerate).
3. Hand-review the skeletons (see gotchas), give `todo!()`s exercise labels.
4. Write `tests/src/exercises.rs`; add `[dev-dependencies]` to `tests/Cargo.toml`.
5. Check all of:
   ```bash
   cargo test -p mNN-name-tests                                 # passes (run it several times if async)
   cargo test -p mNN-name-solution --doc                        # doc comments compile (indented blocks are Rust!)
   cargo test -p mNN-name-tests --features mine                 # compiles, fails (or opt-in, see module 13)
   cargo clippy --all-targets -p mNN-name-solution -p mNN-name -p mNN-name-tests
   ```
6. Write README.md, GETTING_STARTED.md, solution/ANSWERS.md (and exercises.md
   if the module had none — say so in the README).
7. Update the phase README ("Running This Phase" table), the root README
   status table and test count, GETTING_STARTED's module index,
   EXERCISES_SUMMARY.md and ROADMAP.md.

### Skeleton gotchas

`tools/skeleton.py` replaces function bodies with `todo!()`. Things it can't
do right on its own:

- **`-> impl Trait`** returns: a bare `todo!()` doesn't compile; add a
  placeholder after it (`todo!(); std::iter::empty()`). The script warns.
- **`Drop::drop`** gets an empty body: a panic during unwinding aborts the
  whole test process.
- **`extern "C" fn`**: never `todo!()` — panics can't unwind into C; return a dummy value.
- **`const fn`** gets a bare `todo!()` (formatted panics aren't const);
  constructors used by `static`s must be implemented in the skeleton.
- **Middleware** (axum/actix) should pass requests through, or every route's
  tests fail before the middleware exercise is attempted.
- **Infrastructure** the learner shouldn't write (mock servers, test
  helpers) is copied verbatim from the solution, not skeletonised.
- **Signatures that *are* the exercise** (module 11) are written naively by hand.
- **Attributes that give the answer away** (`#[instrument(fields(..))]`,
  schema attributes on CRDs) are copied too: strip them and leave a TODO.
- **Restoring provided bodies** from the solution by signature: if the same
  signature appears first in a trait declaration, slice by `impl` block instead.
- Solution doc comments are copied; remove any that give the answer away.
- **Const-generic braces in a return type** (`-> Foo<{ N }>`) confuse the
  script: use a type alias.
- **Global allocator hooks** must not `todo!()` (a panic inside `alloc`
  aborts): forward to `System` in the skeleton.
- **clap `#[arg(..)]` / Anchor `#[account(..)]` constraints** are part of the
  answer: strip them from the exercise copy and leave a TODO.
- **A skeleton can trip lints its solution doesn't** (`ptr_arg` on a
  `&mut Vec` whose pushing body is `todo!()`): targeted `#[allow]` with a comment.
- **Bench targets** (`[[bench]]`) aren't copied by the scaffold: keep them in
  the solution only.

## Conventions

- Edition 2024, `rust-version = 1.87`. Format with rustfmt; keep clippy clean
  (use a targeted `#[allow(..)]` with a comment when a lint fights the lesson).
- Comments explain *why*; every `unsafe` block has a `// SAFETY:` comment.
- Offline by default: no test may need Docker, a database server, a cluster,
  the internet or an account. Use in-memory/in-process substitutes and mark
  real-infrastructure steps as optional in the docs.
- Timing-sensitive tests use Tokio's paused clock (`start_paused = true`) or
  generous deadlines; anything that could deadlock runs under a watchdog
  (module 15's `deadline` helper). Verify new async/concurrent tests by
  running them several times.
- Don't edit an original `exercises.md` to fix its mistakes; document them in
  the module README and fix them in the solution and tests.
- Verify claims about compiler errors and library APIs by compiling a scratch
  file or reading the crate source in `~/.cargo/registry/src` — several crates
  here (SQLx 0.9, jsonwebtoken 11, argon2 0.6, utoipa 6, askama 0.16, Anchor
  1.2, ratatui 0.30) are newer than most documentation. SQLx 0.9 rejects
  non-literal SQL unless wrapped in `AssertSqlSafe`.
- Keep the 1.87 MSRV in mind: resolver 3 picks MSRV-compatible versions, and
  newer std APIs (e.g. `as_chunks`, 1.88) aren't available outside Phase 6.

## Git

- Work on `main` (after review) — each phase has landed as one commit.
- Commit messages end with the `Co-Authored-By` trailer requested in the session.
- `Cargo.lock` is committed; `target/` and `PROGRESS.md` (a learner's copy of
  `PROGRESS.template.md`) are ignored.
