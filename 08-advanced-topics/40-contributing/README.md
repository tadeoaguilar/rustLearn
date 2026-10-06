# 40 · Contributing

## Overview

The last module is about everything around the code:
- turning a bug report into a regression test and a minimal fix;
- knowing which changes are breaking and what version they need;
- writing the changelog;
- CI that keeps a project healthy.

The exercises are tools maintainers use, built small enough to read: a
patched "upstream" crate, a semver checker in the spirit of
cargo-semver-checks, a Conventional Commits release tool, a CI workflow
generator and reviewer, and a commit message linter.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Bug fixes | Reproduce, regression test, minimal fix, commit message, PR description |
| 2 | Semver | What breaks a Rust API; Cargo's `0.x` rules |
| 3 | Releases | Conventional Commits -> version bump -> Keep a Changelog |
| 4 | CI | fmt, clippy `-D warnings`, test matrix, MSRV; reviewing a workflow |
| Bonus | Commit messages | The conventions reviewers ask for |

## Key Concepts

### Breaking changes in Rust (abridged from the Cargo book's SemVer chapter)

| Change | Bump |
|---|---|
| Remove or rename a public item; change a signature | major |
| Add a public item | minor |
| Add a variant to an exhaustive enum | **major** (breaks `match`) |
| Add a pub field to a struct whose fields are all pub | **major** (breaks struct literals) |
| Add a required method to a trait | **major** (breaks implementors) |
| Add a provided (default) method | minor (usually) |
| Mark an enum/struct `#[non_exhaustive]` | major |
| Bump the MSRV | minor, by most projects' policy |

`#[non_exhaustive]` exists so that future variants or fields are minor
changes.

### Conventional Commits -> versions

```text
feat(parser): support comments         -> minor
fix: handle empty input                -> patch
feat(api)!: rename `run` to `execute`  -> major  (or a BREAKING CHANGE: footer)
docs: fix typo                         -> no release
```

Tools like release-plz, cargo-release and git-cliff automate the
version, changelog and tag from these.

### A good pull request

- **One concern**: a bug fix doesn't also reformat the file.
- **A test** that fails before and passes after.
- **A description**: what, why, how it was tested, and open questions.
- **The project's checks** run locally (its CI file lists them).
- **Linked issue**: `Fixes #123`.

## Common Pitfalls

1. **Fixing more than the issue** -- unrelated changes make review slow and reverts painful
2. **No regression test** -- the bug comes back with the next refactor
3. **"Just a bug fix" that changes observable behaviour** -- consider a minor release and a changelog note
4. **Forgetting `0.x` rules** -- in `0.3.1`, a breaking change is `0.4.0`, not `1.0.0`
5. **Unpinned actions** -- `uses: some/action` follows its default branch; pin to a tag or SHA
6. **Unquoted versions in YAML** -- `toolchain: 1.80` is the float `1.8`; quote it (serde does)
7. **`#![deny(warnings)]` in the crate** -- new toolchains break your users' builds; deny in CI instead

## Running This Module

```bash
cargo run  -p m40-contributing -- 1                    # reproduce the bugs (1 panics: that's issue #1)
cargo test -p m40-contributing-tests --features mine   # test your code
cargo run  -p m40-contributing-solution -- all
cargo run  -p m40-contributing-solution -- semver old.rs new.rs 1.4.2
cargo test -p m40-contributing-tests                   # 15 tests against the solution
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository. It was written
to match the format of earlier modules. [The phase README](../README.md)
lists *how* to contribute (find a project, read the guidelines, start
small, communicate, quality PRs) rather than exercises. Those steps are
turned into practice that runs offline.

| Step | Practised in |
|---|---|
| bug reports to fixes | Ex 1 |
| what maintainers check before a release | Ex 2, Ex 3 |
| CI | Ex 4 |
| commit hygiene | bonus |

- **Exercise 1 is different from every other exercise in the repository**:
  the exercise crate contains working, buggy code instead of `todo!()`,
  and `ex1_existing_behaviour` passes before you start.
- **A real contribution** (an upstream issue and PR) is described in
  exercises.md as optional. It needs an account, a network and a project's
  maintainers, so it isn't part of the tests.
- The RFC process, governance and community aren't code. The README's
  resources and the phase README cover them.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Resources

- [The Cargo Book: SemVer Compatibility](https://doc.rust-lang.org/cargo/reference/semver.html)
- [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks)
- [Conventional Commits](https://www.conventionalcommits.org/) · [Keep a Changelog](https://keepachangelog.com/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [The RFC process](https://github.com/rust-lang/rfcs) · [rustc-dev-guide](https://rustc-dev-guide.rust-lang.org/)

## Next

This is the last module. See [the projects](../../09-projects/) to put
everything together.
