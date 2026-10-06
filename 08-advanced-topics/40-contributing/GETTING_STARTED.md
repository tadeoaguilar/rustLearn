# Getting Started with 40 · Contributing

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m40-contributing-solution -- all
```

shows:
- the four issues fixed;
- a semver check of two versions of a small API (Major: 1.4.2 -> 2.0.0);
- a changelog gaining a 2.0.0 section from five commits;
- a generated CI workflow that passes its own review, while a sloppy one
  gets five comments;
- a commit message linter.

## What Is Already Here

```
40-contributing/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/src/              # ← YOUR WORKSPACE (package m40-contributing)
│   ├── ex01_upstream.rs       #   Ex 1  complete and BUGGY -- fix the four issues in exercises.md
│   ├── ex02_semver.rs         #   Ex 2  (Bump, Change and small helpers provided)
│   ├── ex03_changelog.rs      #   Ex 3
│   ├── ex04_ci.rs             #   Ex 4  (the workflow structs and step helpers provided)
│   └── bonus_commit_lint.rs   #   bonus
├── solution/                  # ← REFERENCE (m40-contributing-solution) + ANSWERS.md
└── tests/                     # ← 15 tests (m40-contributing-tests)
```

## The Commands You Need

```bash
cargo test -p m40-contributing-tests --features mine ex1_       # 1 passes, 4 fail: one per issue
cargo test -p m40-contributing-tests --features mine ex1_issue_2
cargo test -p m40-contributing-tests --features mine
cargo test -p m40-contributing-tests                            # the solution: always green
```

Try the semver checker on your own code: save two versions of a `lib.rs`
and run
`cargo run -p m40-contributing -- semver old.rs new.rs 0.3.1`.

## If You Get Stuck

1. **#1** -- `text.len()` is bytes. `chars().count()` and `chars().take(n)` work in characters.
2. **#2** -- the loop decides the unit *before* rounding. After choosing, check whether the value rounded to one decimal reaches 1024.
3. **#3** -- track "a number has started" separately from its value (`Option<u64>`), and whether any unit was seen.
4. **#4** -- `checked_mul` and `checked_add` return `None` on overflow; `.ok_or(ParseError::Overflow)?`.
5. **Ex 2, `impl` keys** -- `imp.trait_` is `Some((_, path, _))` for trait impls. For inherent impls, use the `pub fn`s inside.
6. **Ex 2, additions** -- whether an added field or variant is breaking depends on the *old* parent (`parent_key` is provided).
7. **Ex 3, `insert_release`** -- the Unreleased section ends at the next line starting with `## ` or `[`. The links at the end need rewriting, not just appending.
8. **Ex 4, YAML** -- serialize the structs with `serde_yaml_ng::to_string`. In `validate`, parse to `serde_yaml_ng::Value` and use `.get("key")`, `.as_mapping()`, `.as_sequence()`, `.as_str()`.
9. Compare with `solution/src/` -- same file and function names.
