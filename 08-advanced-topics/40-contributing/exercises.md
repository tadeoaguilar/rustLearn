# Exercises: Contributing

Most of a contribution isn't code:
- understanding the report;
- reproducing it;
- a test that proves the fix;
- a change small enough to review;
- the right version number;
- a changelog entry;
- CI that keeps it fixed.

These exercises practise each step on code that runs offline. Opening
real issues and pull requests is the optional last step.

**Setup**: `syn`, `serde`, `serde_yaml_ng`.

---

## Exercise 1: Fix Four Reported Bugs

**Difficulty**: Medium
**Time**: 1.5 hours

**Learning Objectives**:
- Reproduce a bug from a report, and capture it as a regression test
- Make the smallest fix that is correct, without changing the public API
- Write a commit message and a PR description a maintainer can merge from

`exercise/src/ex01_upstream.rs` is a small crate as you'd find it upstream:
working, and buggy. These issues were filed against it:

> **#1 `truncate` panics on non-ASCII text**
>
> ```rust
> humanize::truncate("año nuevo", 3)
> ```
> panics with `byte index 2 is not a char boundary; it is inside 'ñ'`.
> Expected `"añ…"`. Also, `truncate("abc", 0)` panics with an overflow in
> debug builds.

> **#2 `format_bytes(1_048_575)` prints "1024.0 KiB"**
>
> One byte short of a MiB displays as `1024.0 KiB`. I'd expect `1.0 MiB`,
> since `1024.0` is never shown for the other units.

> **#3 `parse_duration` silently accepts malformed input**
>
> `parse_duration("90")` returns `Ok(0s)`: the number without a unit is
> dropped, and our config typo went unnoticed for a week. `""` and `"h"`
> are also `Ok(0s)`. The `ParseError` enum already has `MissingUnit` and
> `Empty`; are they meant for this?

> **#4 `parse_duration` panics on large numbers**
>
> `parse_duration("99999999999999999999h")` panics in debug ("attempt to
> multiply with overflow") and returns garbage in release. It's parsing
> user input, so it shouldn't panic. `ParseError::Overflow` exists.

For each issue, in this order:
1. reproduce it (`cargo run -p m40-contributing -- 1` runs the examples);
2. find the matching regression test (`ex1_issue_N_*`) and watch it fail;
3. fix it;
4. keep `ex1_existing_behaviour` green.

Then write the commit message for the fix of #1. Write the PR
description for #3 and #4 together: what changed, why, how it was tested,
and anything a reviewer should decide.

**Question**: is the fix for #3 itself a breaking change? Some callers may
rely on `parse_duration("90")` being `Ok`. What version would you release
it in?

---

## Exercise 2: A Semver Checker

**Difficulty**: Hard
**Time**: 2.5 hours

**Learning Objectives**:
- Know exactly which API changes are breaking in Rust
- Extract a crate's public API with `syn`
- Apply Cargo's semver rules, including `0.x`

1. `public_api(source)`: the map described in the file's doc comment.
2. `diff(old, new)`: each added, removed or changed item, with the bump it
   requires:
   - removed or changed: Major;
   - added: Minor, except that these are Major:
     - a new variant of an exhaustive enum;
     - a new pub field on a struct that could be built with a literal
       (every field pub, not `#[non_exhaustive]`);
     - a new required method on an existing trait;
   - an enum or struct becoming `#[non_exhaustive]`: Major. The reverse
     is Minor;
   - a required trait method gaining a default: Minor.
3. `required_bump` and `next_version`. In `0.y.z`, `y` is the major
   version; in `0.0.z`, every release is breaking.

**Question**: list three breaking changes this checker can't see. (Hint:
think about types, traits, and behaviour.)

---

## Exercise 3: Conventional Commits and the Changelog

**Difficulty**: Medium
**Time**: 1.5 hours

**Learning Objectives**:
- Parse a commit convention
- Derive the version bump and the release notes from the history
- Edit a Keep a Changelog file, including its compare links

1. `parse_commit`: `type(scope)!: description` and `BREAKING CHANGE:`
   footers.
2. `bump_for`: breaking -> Major, `feat` -> Minor, `fix`/`perf` -> Patch,
   otherwise none.
3. `release_section` and `insert_release`, as documented in the file.
   `latest_version` too.

---

## Exercise 4: CI for a Rust Project

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- Know the checks Rust projects run on every pull request, and why
- Generate YAML from typed data instead of templates
- Review a workflow for common mistakes

1. `workflow(project)` generates three jobs:
   - `check`: fmt + clippy `-D warnings`;
   - `test`: an OS matrix;
   - `msrv`: if set.
2. `validate(yaml)`: the review checks listed in the file.

**Question**: why is `-D warnings` in CI and not in the crate
(`#![deny(warnings)]`)?

---

## Bonus: Commit Message Lint

**Difficulty**: Easy
**Time**: 30 minutes

The problems reviewers point out:
- a header longer than 72 characters;
- a trailing period;
- not imperative ("Fixed", "Adding");
- no blank line after the header;
- long body lines.

---

## Optional: A Real Contribution (Not Run in This Repository's Checks)

1. Pick a crate you use. Read its CONTRIBUTING.md and look at issues
   labelled `good first issue` or `E-easy` (rust-lang repos label mentored
   ones `E-mentor`).
2. Comment before starting anything non-trivial. Maintainers may already
   have a plan.
3. Fork, branch, and reproduce with a failing test. Fix it, then run the
   project's own checks (its CI file says which).
4. Open the PR with the description structure from Exercise 1. Link the
   issue (`Fixes #123`), and respond to review by pushing commits, not by
   force-pushing over the discussion (unless the project asks).
5. Documentation counts: unclear docs you had to puzzle over are a
   contribution waiting to happen.
