# Answers · 11 Advanced Lifetimes

## Exercise 1

**Why does `longest(&x, &y)` reject Task 2's code?**

`longest<'a>(x: &'a str, y: &'a str) -> &'a str` says the result may borrow
from *either* argument, so the caller must keep both alive for as long as the
result is used. In Task 2, `y` is dropped at the end of the inner block while
`result` is used after it: `error[E0597]: 'y' does not live long enough`. The
compiler doesn't look inside `longest` to see which one is actually returned —
the signature is the contract.

## Exercise 2

**With a single lifetime `Highlighter<'a>`, why does the keyword test fail?**

Both fields are `&'a str`, so `'a` can be no longer than the shorter of the two
borrows: the keyword's. `matching_words` returns `Vec<&'a str>`, so the results
are capped at the keyword's lifetime too, even though they point into the text.
Dropping the keyword first gives `error[E0597]: 'keyword' does not live long
enough`. Separate lifetimes `'t` and `'k` record that results depend on `'t`
only.

## Exercise 5

**Why does `spawn_and_join(&local_string)` fail but `spawn_and_join(local_string)` work?**

`thread::spawn` requires its closure (and so everything it captures) to be
`'static`: the new thread might outlive the function that spawned it. `&local`
is a `&'x String` for some `'x` shorter than the function — not `'static`.
`local` itself is a `String`, which owns its data and contains no borrows, so
`String: 'static` holds; it's *moved* into the thread.

**What would happen with `Box<dyn Fn(&str) -> String>` (no `+ 'a`)?**

In a struct field, `Box<dyn Trait>` means `Box<dyn Trait + 'static>`.
Registering `|e| format!("{prefix}{e}")`, which borrows the local `prefix`,
would fail: `error[E0373]: closure may outlive the current function, but it
borrows 'prefix'` — "note: function requires argument type to outlive
'static". `+ 'a` says the closures may borrow anything that outlives the
`Callbacks`.

## Exercise 6

**Why can't `apply_to_all` be `fn apply_to_all<'a, F: Fn(&'a str) -> &'a str>`?**

Lifetime parameters on a function are chosen by the **caller**, and they must
outlive the whole call. A `String` created inside `apply_to_all` lives for
part of the call only, so `f(&local)` doesn't match the caller's `'a`:
`error[E0597]: 'local' does not live long enough`. `for<'a>` moves the choice
into the bound: `f` must work for *every* lifetime, including ones that exist
only inside the function.
