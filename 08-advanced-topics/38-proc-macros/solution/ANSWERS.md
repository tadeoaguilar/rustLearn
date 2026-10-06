# Answers · 38 Procedural Macros

## Exercise 1: Absolute paths and `extern crate self`

A macro's output is pasted into the *user's* code, where nothing guarantees
`ToJson` is in scope -- or that it means the same trait: the user may have
their own `ToJson`, or have imported ours under another name. Every path in
generated code should be absolute (`::std::vec::Vec`, `::krate::Trait`), and
the leading `::` stops a local module named `std` from capturing it. This
is called *hygiene*. `macro_rules!` has some built in; proc macros get only
span-based hygiene for local variables, so the discipline is on you.

The absolute path names the runtime crate as a dependency would see it.
Inside that crate itself, `::m38_proc_macros_solution` doesn't exist (a
crate refers to itself as `crate`), so the derives used in its own
`main.rs` or doc tests would fail. `extern crate self as <name>;` adds the
missing name. serde and others do the same; the alternative is the
`proc-macro-crate` crate, which finds the name a user gave the dependency
in `Cargo.toml`.

## Exercise 4: Why a closure, and why no borrow

**The closure.** The body may contain `return` (the `greet` test does). If
it were pasted directly into the wrapper, an early `return` would leave
the wrapper *before* the result is stored in the cache, so those calls
would never be memoized. The closure turns every `return` into "the
closure's value". The same trick is used by `#[tracing::instrument]` and
`async-trait` (with an async block). `?` behaves the same way inside it.

**No borrow.** `fib(n)` calls `fib(n - 1)` from inside its body. If the
wrapper held `cache.borrow_mut()` across the body, the inner call's
`borrow()` would find the cell already mutably borrowed and panic
("already mutably borrowed"). So the wrapper borrows briefly to look up
(and clones the hit), releases, runs the body, then borrows again to
insert. Two recursive branches may compute the same value; the second
insert just overwrites it, so the results stay correct.

The macro can't see the types it is given. `Clone + Hash + Eq` on the
arguments and `Clone` on the result aren't checked by the macro at all:
the generated `HashMap<(A, B), R>` and `.clone()` calls make the
compiler report them, at the user's types.
