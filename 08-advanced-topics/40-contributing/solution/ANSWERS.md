# Answers · 40 Contributing

## Exercise 1: The commit message for #1

```text
fix(truncate): count characters, not bytes

`truncate` compared and sliced the string by byte length, so text with
multi-byte characters was cut too early or panicked when the cut fell
inside a character ("año nuevo", 3). A `max_chars` of 0 also underflowed.

Count with `chars()`, cut with `take`, and return an empty string for 0.
Existing ASCII behaviour is unchanged.

Fixes #1
```

Header: imperative, under 72 characters, says *what*. Body: *why* it was
wrong and *what* the fix is, wrapped at 72. Footer: links the issue
(GitHub closes it on merge).

## Exercise 1: The PR description for #3 and #4

```markdown
## parse_duration: reject malformed and overflowing input

Fixes #3, fixes #4.

### What changed
- A trailing number without a unit (`"90"`, `"1h30"`) is now
  `Err(MissingUnit)` instead of being silently dropped.
- Input with no components (`""`, `"   "`) is `Err(Empty)`, and a unit
  without a number (`"h"`) is `Err(InvalidChar('h'))`. All were `Ok(0s)`.
- All arithmetic is checked: values that don't fit in `u64` seconds are
  `Err(Overflow)` instead of panicking (debug) or wrapping (release).

The public API is unchanged: these `ParseError` variants already existed
but were never returned.

### Testing
Regression tests for every example in both issues, plus edge cases
(`u64::MAX` seconds still parses; one more second overflows). Existing
tests pass unchanged.

### For the reviewer
Inputs that used to return `Ok(0s)` now return errors. That's what #3
asks for, but it is a behaviour change: should this go in a minor release
(my suggestion, with a changelog note) or wait for the next major?
```

## Exercise 1: Is #3's fix breaking?

Formally, semver is about the *documented* API. `parse_duration("90")`
returning `Ok(0s)` was never documented. It's a bug, and bug fixes go in
patch releases. In practice, someone may depend on it, as Hyrum's law
says: "all observable behaviours will be depended on". The usual
compromise is what the PR asks:
- release it as a **minor** version, not a patch, so `~1.4` pins don't
  pick it up silently;
- call it out under "Changed" in the changelog;
- maintainers of widely used crates sometimes also run crater (rustc) or
  test reverse dependencies first.

The panic fix (#4) is a plain bug fix: a patch. No one depends on a
panic.

## Exercise 2: Breaking changes a syntax checker can't see

1. **Behaviour**: a function that returns different results, panics on
   new inputs, or gets slower by orders of magnitude. The signature is
   unchanged.
2. **Auto traits**: adding a field of type `Rc<T>` or `*const T` to a
   public struct makes it lose `Send`/`Sync`. Nothing in the source
   *says* `Send`, but every caller who sent it to a thread breaks. Only
   type information shows it, which is why cargo-semver-checks works
   from rustdoc JSON, not syntax.
3. **Type aliases and re-exports**: changing `pub type Id = u32` to `u64`
   only changes a type expression. Moving an item and re-exporting it
   under the old path is *not* breaking, but a syntax diff sees a
   removal.
4. Also invisible to it:
   - **trait impls via generics or macros** (`impl<T: Display> MyTrait for T`
     coherence changes);
   - **MSRV bumps**;
   - **new default features**;
   - **sealed traits** (a new required method on a sealed trait is fine).

## Exercise 4: `-D warnings` in CI, not in the crate

New compiler and clippy versions add warnings. With `#![deny(warnings)]`
in the source, a crate that compiled yesterday fails to compile today for
every *user* on a newer toolchain, and you can't fix your dependents'
builds. (Cargo caps lints for dependencies with `--cap-lints`, but
`deny(warnings)` still breaks your own builds and contributors'.) In CI,
the same policy applies only to the project's own checks. When a new
toolchain adds a warning, CI goes red and someone fixes it in a PR;
nobody downstream is affected. The same reasoning covers the MSRV job:
it catches use of newer APIs before users on the old toolchain do.
