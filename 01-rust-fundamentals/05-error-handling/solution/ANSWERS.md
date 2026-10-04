# Answers · 05 Error Handling

## Exercise 1: When should you use panic vs return Result?

**Panic** when continuing would be wrong because the program itself is broken:
an invariant your code relies on doesn't hold, an index you computed is out of
range, a "can't happen" branch happened. Panicking stops the thread and prints
a backtrace pointing at the bug.

**Return `Result`** whenever the failure can happen in a correct program: the
user typed letters into a number field, the file was deleted, the network is
down, the config has a typo. The caller is in a better position to decide
whether to retry, fall back to a default, or report the problem.

Rules of thumb:

- Library code almost never panics on input; it returns `Result`.
- `unwrap()` / `expect()` are fine in tests, examples and quick prototypes, and
  in real code when you can explain in the `expect` message why it can't fail.
- If the caller *could* check a precondition but forgetting would be a bug
  (`v[i]`, `slice.split_at(n)`), panicking is acceptable and documented with a
  `# Panics` section. Offer a non-panicking twin (`v.get(i)`) as well.
