# 11 · Advanced Lifetimes

## Overview

A lifetime annotation never makes anything live longer. It's a **claim about
how references in a signature relate** — "the result borrows from `x`, not
`y`" — that the compiler checks inside the function and then relies on at
every call site. Most lifetime errors are one of two things: a signature that
claims too much (the result is tied to something it doesn't actually borrow
from), or code that really does keep a reference to something that's about to
die. This module trains you to tell them apart.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Annotations & elision | The three rules; when you must annotate |
| 2 | Structs that borrow | `-> &'a str` vs `-> &str` on methods; two independent lifetimes |
| 3 | Zero-copy parser | An HTTP request parsed into slices of the input |
| 4 | Borrowing iterators | `Item = &'a T`; a mutable iterator without `unsafe` |
| 5 | `'static` | `T: 'static` vs `&'static T`; `thread::spawn`; `dyn Trait + 'a`; `OnceLock` |
| 6 | HRTB | `for<'a> Fn(&'a str) -> &'a str` and why the caller can't pick `'a` |
| 7 | Fixing errors | Five real errors; most are fixed by ownership, not annotations |
| Bonus | `StrSplit` | Why a delimiter shouldn't share the haystack's lifetime |

## Key Concepts

### The elision rules

1. Each reference parameter gets its own lifetime.
2. Exactly one input lifetime → it's used for every output.
3. A `&self` / `&mut self` parameter → its lifetime is used for every output.

Rule 3 is the trap. In `impl<'a> Excerpt<'a>`, `fn text(&self) -> &str` means
the result borrows from **the `Excerpt`**, not from the novel it points into.
Write `-> &'a str` and callers can keep the text after the `Excerpt` is gone.

### Don't tie things together that aren't related

```rust
fn pick_first<'a>(x: &'a str, y: &'a str) -> &'a str   // y must outlive the result. Why?
fn pick_first<'a>(x: &'a str, y: &str) -> &'a str      // only x matters
```

Same for structs: `Highlighter<'a> { text: &'a str, keyword: &'a str }` caps
every result at the keyword's lifetime. Two parameters, `<'t, 'k>`, let results
depend on the text alone.

### `T: 'static` is not `&'static T`

`T: 'static` means "contains no borrows shorter than `'static`" — every owned
type qualifies: `String`, `Vec<u8>`, `i32`. It's why `thread::spawn(move || ...)`
works with owned data and rejects borrowed locals. `&'static T` is a reference
that's valid forever: a literal, a `static`, or a leaked `Box`.

### Higher-ranked bounds

`F: for<'a> Fn(&'a str) -> &'a str` — "for *every* lifetime". Needed when the
function calls `f` on borrows the caller can't name (locals inside the
function). Writing `fn f<'a, F: Fn(&'a str) -> &'a str>` lets the *caller*
choose one `'a`, which no local inside can satisfy.

### When the borrow checker is wrong

Exercise 7c — returning a borrow from one branch and mutating on the other —
is correct code that today's checker rejects (NLL "problem case #3"). The
entry API or a `contains_key` check restructures it. The same limitation shows
up in Exercise 4's mutable iterator; the solution's comment points it out.

## Common Pitfalls

1. **Adding `'a` everywhere until it compiles** — usually over-constrains; annotate only relationships that exist
2. **`-> &str` on a method of a borrowing struct** — ties the result to `&self`
3. **Returning a reference to a local** — no annotation can fix it; return an owned value
4. **`Box<dyn Fn>` in a struct** — means `+ 'static`; add `+ 'a` to allow borrowing closures
5. **Fighting a mutable iterator** — `mem::take` the slice, then `split_first_mut`

## Running This Module

```bash
cargo run  -p m11-lifetimes-advanced -- 2                                    # your code
cargo test -p m11-lifetimes-advanced-tests --features mine                   # behaviour tests on your code
cargo test -p m11-lifetimes-advanced-tests --features mine,signatures        # + lifetime signature checks
cargo run  -p m11-lifetimes-advanced-solution -- all                         # the reference solution
cargo test -p m11-lifetimes-advanced-tests                                   # 24 tests against the solution
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of the others and the topics listed in
[the phase README](../README.md). Two things are specific to it:

- **The skeleton has naive signatures.** In `exercise/src`, `pick_first`,
  `Excerpt::text`/`longest_word`, `Highlighter`, `Request::header` and
  `Parser::next_token` compile but are too strict. Fixing them *is* the
  exercise.
- **Signature tests are separate.** `tests/src/signatures.rs` holds six tests
  that keep a result alive after something it shouldn't depend on is dropped.
  A too-strict signature makes them fail to *compile*, which would block every
  other test — so against your code they only run with
  `--features mine,signatures`. Against the untouched skeleton you'll see
  exactly five compile errors, one per signature to fix.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[12 · Async/Await](../12-async-await/)
