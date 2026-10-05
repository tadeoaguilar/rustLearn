# Roadmap and Progress

What has been built, the decisions behind it, and the plan for what's left.
For *how* to work on the repository, see [CLAUDE.md](CLAUDE.md); for learners,
start at [GETTING_STARTED.md](GETTING_STARTED.md).

_Last updated: 2026-10-04._

## Status

| Phase | Modules | Exercises | Code + tests | Tests |
|---|---|---|---|---|
| 1 · Rust Fundamentals | 01–05 | ✅ | ✅ | 123 |
| 2 · Intermediate Rust | 06–10 | ✅ | ✅ | 143 (incl. module 09's 7 integration and property tests) |
| 3 · Advanced Rust | 11–15 | ✅ | ✅ | 128 |
| 4 · Web Development | 16–20 | ✅ | ✅ | 84 |
| 5 · Cloud Native | 21–25 | ✅ | ✅ | 89 |
| 6 · Blockchain & Solana | 26–30 | outline | — | — |
| 7 · Systems Programming | 31–35 | outline | — | — |
| 8 · Advanced Topics | 36–40 | outline | — | — |
| 09-projects | — | ideas list | — | — |
| 10-crud-api-project | — | — | ✅ standalone app | (its own) |

`cargo test` at the root: **567 tests, all passing** (per phase: 123 + 143 + 128 + 84 + 89). `cargo clippy
--workspace --all-targets`: no warnings. `cargo fmt --all --check`: clean.

### History

| Commit | What |
|---|---|
| `0ff400a` | Phases 1–3: root workspace, 15 modules, exercise/solution/tests crates |
| `f47ca8d` | Phase 4: modules 16–20 |
| (next) | CLAUDE.md, ROADMAP.md, tools/; Phase 5: modules 21–25 |

## Decisions So Far

These apply to every module unless its README says otherwise.

1. **Same shape as the sibling netLearn repository**: per module a README
   (concepts), `exercises.md`, `GETTING_STARTED.md` (every command), and three
   crates — `exercise/` (the learner's, `todo!()` bodies), `solution/`
   (+ `ANSWERS.md`), `tests/` (runs against either via the `mine` feature).
2. **One root Cargo workspace.** `cargo test` at the root runs every test
   against the solutions and must always pass. `10-crud-api-project` is
   excluded (it's a standalone app with its own lockfile).
3. **Offline and dependency-free by default.** No Docker, databases,
   clusters, networks or accounts are needed to build and test: SQLite in
   memory instead of PostgreSQL, an in-process mock OAuth2 provider instead of
   GitHub, servers on port 0, simulated web for the crawler. Real
   infrastructure appears as clearly marked *optional* steps.
4. **Original exercises are not silently rewritten.** Mistakes found in the
   original `exercises.md` files are listed under "Notes on `exercises.md`" in
   each module README and fixed in the solution and tests.
5. **Missing exercise files are written in the existing format** (modules
   08–11 and 13–25 so far), each README saying so.
6. **Edition 2024, Rust 1.87+.** Skeletons and solutions follow 2024 rules
   (`unsafe extern`, `#[unsafe(no_mangle)]`, `impl Trait` capture rules).
7. **Rocket** is covered in module 16's comparison and bonus but not
   implemented: a third framework would add a large dependency tree to every
   build for little extra learning.

### Mistakes found in the original material

| Module | Problem | Where documented |
|---|---|---|
| 01 | "missing semicolon in return" is backwards; `chrono` in build.rs needs a build-dependency | 01 README |
| 03 | `search("l")` on "Say: Hello World" is `[7, 8, 14]`, not `[9, 13]`; `drain_filter` was never stabilised (`extract_if`); `3.14` trips clippy's `approx_constant` | 03 README |
| 04 | 10×20 rectangle scaled ×2 has area 800, not 400; an "equilateral" triangle with base 6 and height 8 is impossible | 04 README |
| 05 | `retry`'s `max_retries` is really max *attempts*; `transpose` doesn't need an iterator | 05 README |
| 06 | duplicate `Summary`/`Pair` definitions; an unneeded `Clone` bound | 06 README |
| 07 | the Fibonacci iterator overflows and panics at item 94; `partial_cmp().unwrap()` panics on NaN | 07 README |
| 12 | `mpsc::Receiver` can't be cloned (the bonus doesn't compile); exercise 4 calls a live web API (replaced by an in-process mock); axum 0.7 / reqwest 0.11 versions and `:id` path syntax | 12 README |

## Plan for the Remaining Phases

Same structure and the same offline rule. For each module: what the code
does, what's tested offline, and what's optional.

### Phase 5 · Cloud Native — done

Built as planned, with these choices (details in each module README):

- 21: the Dockerfiles and Compose file are checked by the module's own
  linter and Compose validator; **they were not built with Docker** (the
  daemon wasn't running), which the README says.
- 22: kube 4.2 needs Rust 1.89, so module 22's crates set `rust-version = "1.89"`.
  `FakeCluster` reproduces defaulting, generations, finalizers and owner-reference
  GC; the real-cluster path compiles but wasn't run against a cluster.
- 23: tonic 0.14 + `protox` (no `protoc`); tonic reports an expired deadline
  as `CANCELLED` server-side; tests accept either.
- 24: traces are collected by a `tracing` Layer the learner writes, rather
  than the OpenTelemetry SDK; the Prometheus/Grafana stack is optional and unrun.
- 25: the mesh's features are built as a Rust sidecar (rustls, rcgen,
  x509-parser); real meshes are optional.

### Phase 6 · Blockchain & Solana

Solana programs normally need the Solana toolchain (`cargo build-sbf`) and a
validator. Plan: a **separate Cargo workspace** under `06-blockchain-solana/`
(excluded from the root, so its very large dependency tree doesn't slow every
other build), with:

- program logic written so it's testable natively (instruction parsing,
  account state transitions, invariants, security checks) — `cargo test`
  works with a normal Rust install
- integration tests with an in-process SVM (LiteSVM/Mollusk) and the Anchor
  versions as the *optional* track for learners who install the toolchain
- modules: 26 accounts/transactions/PDAs basics, 27 Anchor (counter, voting),
  28 multisig + escrow + security review, 29 tokens/NFTs (SPL), 30 AMM +
  lending math (constant product, interest rates, liquidation) with
  property tests

### Phase 7 · Systems Programming

All of it runs offline:

| Module | Built around |
|---|---|
| 31 CLI tools | `clap` grep clone, a log viewer with filters, a task manager with a JSON store; a `ratatui` system monitor (rendering tested with ratatui's test backend) |
| 32 Network programming | TCP echo, an HTTP/1.1 server from scratch, a TCP proxy, a framed custom protocol, UDP |
| 33 Embedded | `no_std` + `embedded-hal` traits: blink, an I2C sensor driver tested with `embedded-hal-mock`, COBS + CRC framing; real boards optional |
| 34 OS concepts | a process supervisor with restarts, a tiny shell (pipes, redirection), IPC over pipes and Unix sockets, shared memory via `memmap2` |
| 35 Memory management | bump allocator, object pool/slab, SoA vs AoS cache experiment, allocation profiling with `dhat` |

### Phase 8 · Advanced Topics

| Module | Built around |
|---|---|
| 36 Performance | profile-guided fixes to a slow program, SIMD (`std::arch` + portable fallbacks), cache blocking, release-profile binary-size reduction; criterion benchmarks |
| 37 WASM | a `wasm-bindgen` library tested natively (wasm-pack optional), WASI file operations; Yew optional |
| 38 Proc macros | beyond module 13: a serialization derive, an ORM-style derive generating SQL, a function-like DSL parsed with `syn` |
| 39 Compiler internals | reading MIR/HIR output, borrow-checker case studies, a lint tool built on `syn` that walks real Rust files (dylint/clippy lints as an optional nightly track) |
| 40 Contributing | a simulated open-source contribution: an issue, a failing test, a fix, docs and semver checks (`cargo-semver-checks`) |

### Not planned

- `09-projects` stays a list of project ideas: they're meant to be built from scratch.
- Phases are built in order 5 → 8; each lands as its own commit on `main`.
