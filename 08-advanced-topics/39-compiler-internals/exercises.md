# Exercises: Compiler Internals

`rustc` is a pipeline:

```text
source -> tokens -> AST -> (macro expansion, name resolution) -> HIR
       -> (type checking, trait resolution) -> THIR -> MIR
       -> (borrow checking, optimizations) -> LLVM IR -> machine code
```

You don't need to build rustc to look inside it. Its outputs, from
`--emit=mir` to `--error-format=json`, are there to inspect, and the AST
stage can be reproduced with `syn`. These exercises build tools at three
points of the pipeline:
- a lint on the syntax tree;
- a MIR reader;
- a borrow-checker lab.

They also rewrite code the way lowering to HIR does.

**Setup**: `syn` with `visit`/`visit-mut`, `serde_json`. Exercises 2-4 run
your installed `rustc` through the provided `toolchain` module. That works
offline, and the `RUSTC` environment variable overrides the compiler.

---

## Exercise 1: A Lint Tool

**Difficulty**: Medium
**Time**: 1.5 hours

**Learning Objectives**:
- Walk a syntax tree with `syn::visit::Visit`
- Report precise locations from spans
- See why real lints run after macro expansion

Report, with 1-based line:column:
- `unwrap_used`: `.unwrap()`;
- `expect_used`: `.expect(x)`;
- `debug_macro`: `todo!`, `unimplemented!` and `dbg!`;
- `as_cast`: at the `as`;
- `long_function`: over `max_fn_lines`, at the name;
- `missing_docs`: a `pub` fn, struct, enum or trait without a doc
  comment, at the name.

Skip `#[cfg(test)]` modules, and sort the findings by position.

**Question**: `println!("{}", x as u8)` isn't reported. Why not, and how
does clippy see it?

---

## Exercise 2: Reading MIR

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- Read MIR: basic blocks, statements, terminators
- See the safety checks Rust inserts, and which ones `-O` removes

`parse_mir`: per function, the name and the counts of:
- basic blocks;
- bounds checks;
- overflow checks;
- calls.

The rules are in the file's doc comment. Then run it on real code
(`cargo run -p m39-compiler-internals -- 2`).

**Question**: in release MIR, `sum_indexed` (indexing `v[i]` in a loop)
still has its bounds check, and `sum_iter` has none. Does the release
*binary* check bounds in `sum_indexed`? How would you find out?

---

## Exercise 3: Borrow Checker Case Studies

**Difficulty**: Medium
**Time**: 1.5 hours

**Learning Objectives**:
- Use rustc's machine-readable diagnostics
- Recognize the classic borrow errors, and fix each one properly

1. `error_codes(source)`: run rustc with `--error-format=json` and collect
   the codes of the errors (not the warnings).
2. `fixed(code)` for every case in `CASES`: E0382, E0499, E0502, E0505,
   E0506, E0515, E0597, E0106 and E0373. Each fix must compile and keep
   `pub fn f`.
3. The real functions, each the fixed form of an error:
   - `longest` (E0106);
   - `add_into` (two mutable borrows of a slice, E0499);
   - `push_after_zeros` (E0502);
   - `spawn_sum` (E0373);
   - `owned_words` (E0597/E0515).

**Question**: the E0499 case is rejected even though `v[0]` and `v[1]` are
different elements. Why can't the borrow checker see that? Why does
`split_at_mut` need `unsafe` inside?

---

## Exercise 4: Desugaring

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- Rewrite a syntax tree with `syn::visit_mut::VisitMut`
- Know what `for` and `?` really are

Rewrite every `for` loop (keeping labels) and every `?` into the forms
shown in the file's doc comment. The tests compile the program before and
after with rustc, run both, and compare their output.

**Question**: why does rustc's `for` desugaring use
`match IntoIterator::into_iter(x) { mut iter => loop { .. } }` instead of
`let mut iter = ..; loop { .. }`?

---

## Bonus: Unused Variables

**Difficulty**: Medium
**Time**: 1 hour

Find the parameters and `let`/closure/`for` bindings that are never used,
per function. Uses include identifiers inside macros and `{name}` inside
format strings. The test checks that you report the same names as rustc's
`unused_variables` lint.

---

## Optional: Beyond This Module (Not Run in This Repository's Checks)

- **A real clippy lint**: clone rust-clippy and run
  `cargo dev new_lint --name=my_lint --pass=late`. Implement
  `LateLintPass::check_expr`, then add UI tests (`tests/ui/my_lint.rs` +
  `.stderr`). This needs nightly and a large download.
- **A rustc contribution**: start from the rustc-dev-guide's "Getting
  Started", `./x check`, and an `E-easy` + `E-mentor` issue.
- **Miri**: `rustup +nightly component add miri`, then `cargo +nightly miri
  test` on module 35's unsafe code.
