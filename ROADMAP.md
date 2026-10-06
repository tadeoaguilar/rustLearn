# Roadmap and Progress

What has been built, the decisions behind it, and the plan for what's left.
For *how* to work on the repository, see [CLAUDE.md](CLAUDE.md); for learners,
start at [GETTING_STARTED.md](GETTING_STARTED.md).

_Last updated: 2026-10-05._

## Status

| Phase | Modules | Exercises | Code + tests | Tests |
|---|---|---|---|---|
| 1 · Rust Fundamentals | 01–05 | ✅ | ✅ | 123 |
| 2 · Intermediate Rust | 06–10 | ✅ | ✅ | 143 (incl. module 09's 7 integration and property tests) |
| 3 · Advanced Rust | 11–15 | ✅ | ✅ | 128 |
| 4 · Web Development | 16–20 | ✅ | ✅ | 84 |
| 5 · Cloud Native | 21–25 | ✅ | ✅ | 89 |
| 6 · Blockchain & Solana | 26–30 | ✅ | ✅ (own workspace) | 136 + 13 runtime (+ 15 on-chain, optional) |
| 7 · Systems Programming | 31–35 | ✅ | ✅ | 117 |
| 8 · Advanced Topics | 36–40 | ✅ | ✅ | 69 |
| 09-projects | — | ideas list | — | — |
| 10-crud-api-project | — | — | ✅ standalone app | (its own) |

`cargo test` at the root: **753 tests, all passing** (per phase: 123 + 143 + 128 + 84 + 89 + 117 + 69).
`cargo clippy --workspace --all-targets`: no warnings. `cargo fmt --all --check`: clean.
Phase 6, from `06-blockchain-solana/`: `cargo test` 149 tests; `./build-sbf.sh && cargo test
--features sbf` adds 15 on-chain tests (needs the Solana toolchain).

### History

| Commit | What |
|---|---|
| `0ff400a` | Phases 1–3: root workspace, 15 modules, exercise/solution/tests crates |
| `f47ca8d` | Phase 4: modules 16–20 |
| `3fea4d0` | CLAUDE.md, ROADMAP.md, tools/; Phase 5: modules 21–25 |
| (next) | Phases 6, 7 and 8: modules 26–40 |

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
   08–11 and 13–40), each README saying so.
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

## How the Later Phases Were Built

Same structure and the same offline rule. What each phase chose (details in
each module README):

### Phase 5 · Cloud Native

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

- A **separate workspace** (`06-blockchain-solana/`, Rust 1.89 for Anchor
  1.2), excluded from the root.
- **`sim/` (solsim)**: an in-process Solana runtime written for the phase.
  It enforces the account rules (ownership, signers, writability, rent,
  lamport balance), executes CPIs through the SDK's syscall stubs, and has
  System, SPL Token (the real processor) and ATA builtins. Programs run
  natively under `cargo test`, with no toolchain or validator.
- **On-chain track** (optional): every program also builds with
  `cargo build-sbf` (`build-sbf.sh`); the tests' `sbf` feature runs the
  same scenarios on LiteSVM. Verified locally with solana-cli 3.1 /
  platform-tools v1.52.
- 26 raw programs (accounts, PDAs, CPI), 27 Anchor, 28 multisig + escrow +
  staking + a security review, 29 tokens/NFTs (metadata modelled in the
  program, compressed NFTs as a Merkle-tree bonus), 30 AMM and lending maths
  with property tests.

### Phase 7 · Systems Programming

| Module | Built around |
|---|---|
| 31 CLI tools | `clap` grep clone, a log viewer, a task manager with a JSON store; a `ratatui` system monitor tested with the test backend |
| 32 Network programming | echo servers, HTTP/1.1 from scratch, a proxy + load balancer, a framed chat protocol, UDP ping; DNS messages bonus |
| 33 Embedded | a `no_std` library on `embedded-hal` 1.0, tested with `embedded-hal-mock`; real boards optional and unrun |
| 34 OS concepts | a supervisor, a shell with pipes and redirection, IPC, shared memory (`memmap2`), signals and rlimits (`nix`); Unix only |
| 35 Memory management | bump allocator, slab/pool, a counting global allocator (instead of `dhat`), layout, AoS vs SoA |

### Phase 8 · Advanced Topics

| Module | Built around |
|---|---|
| 36 Performance | a hot path fixed by measurement, SIMD (SSE2/AVX2 with runtime detection + portable fallback), cache-blocked matmul, "measure first" on branches, a `min-size` profile (−39%); criterion benches |
| 37 WASM | a `#[wasm_bindgen]` library tested natively; wasm-pack demo and WASI build documented but **not run** (no wasm target installed); Yew left as a pointer |
| 38 Proc macros | `core/` (expansions on `proc_macro2`, unit-tested) + `derive/` (thin proc-macro crate): ToJson and ORM derives, a `state_machine!` DSL, `#[memoize]`; usage tests opt-in per exercise like module 13 |
| 39 Compiler internals | a `syn` lint tool, a MIR reader, borrow-checker cases checked with rustc's JSON diagnostics, `for`/`?` desugaring verified by compiling and running both versions |
| 40 Contributing | a buggy upstream crate to fix from four issues (the only exercise that isn't `todo!()`), a semver API-diff checker, Conventional Commits + Keep a Changelog, a CI workflow generator/reviewer |

### Not planned

- `09-projects` stays a list of project ideas: they're meant to be built from scratch.
