# Getting Started with 39 · Compiler Internals

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m39-compiler-internals-solution -- all
```

It runs each tool:
- the linter flags an `unwrap`, a `dbg!`, an `as` cast and missing docs
  (but not the test module);
- the MIR table shows bounds and overflow checks in debug and `-O` builds;
- each borrow-checker case is rejected with its error code, and its fix
  compiles;
- a program prints the same output before and after desugaring;
- the bonus reports unused variables.

## What Is Already Here

```
39-compiler-internals/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/src/              # ← YOUR WORKSPACE (package m39-compiler-internals)
│   ├── ex01_lint.rs           #   Ex 1  (Lint, Finding display, Config provided)
│   ├── ex02_mir.rs            #   Ex 2
│   ├── ex03_borrowck.rs       #   Ex 3  (CASES provided)
│   ├── ex04_desugar.rs        #   Ex 4  (desugar, count_sugar provided)
│   ├── bonus_unused.rs        #   bonus
│   └── toolchain.rs           #   provided: run rustc on a snippet, emit MIR, run a program
├── solution/                  # ← REFERENCE (m39-compiler-internals-solution) + ANSWERS.md
└── tests/                     # ← 11 tests (m39-compiler-internals-tests)
```

## The Commands You Need

```bash
cargo test -p m39-compiler-internals-tests --features mine ex1_
cargo test -p m39-compiler-internals-tests --features mine
cargo test -p m39-compiler-internals-tests                    # the solution: always green
```

Looking at the compiler yourself:

```bash
rustc --crate-type=lib --emit=mir -o - some.rs                 # MIR to stdout
rustc --crate-type=lib --emit=mir -O -o - some.rs              # after MIR optimizations
rustc --crate-type=lib --error-format=json some.rs             # diagnostics as JSON
rustc --explain E0502                                          # the long explanation of an error
```

## If You Get Stuck

1. **Columns off by one** -- `span.start().column` is 0-based; add 1. A line number of 0 means the `span-locations` feature isn't on (it is in Cargo.toml).
2. **Nothing found inside a `#[cfg(test)]` module, or too much** -- override `visit_item_mod` and don't call the default visitor for test modules.
3. **Long-function length** -- from the `fn` token's line to the closing brace's line (`block.brace_token.span.close()`), inclusive.
4. **`error_codes` returns warnings too** -- keep `level == "error"`; and skip lines that aren't JSON.
5. **Desugared program doesn't compile** -- print it; `parse_quote!` with a label: `#label loop` works when `label` is an `Option<Label>`.
6. **`total` differs after desugaring** -- `break`/`continue` with a label must target the generated `loop`, so the label goes on it.
7. **Bonus finds too many** -- a name used only in `println!("{x}")` is a use: scan the macro's tokens and the `{...}` in its string literals.
8. Compare with `solution/src/` -- same file and function names.
