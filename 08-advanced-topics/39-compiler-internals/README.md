# 39 · Compiler Internals

## Overview

Knowing how rustc works turns its errors from obstacles into explanations:
- why the borrow checker reasons about *places*;
- what a `for` loop really is;
- where the bounds checks are, and when they disappear.

This module builds small tools at several points of the pipeline:
- a clippy-style lint on the syntax tree;
- a reader for `--emit=mir` output;
- a borrow-checker lab driven by rustc's JSON diagnostics;
- a desugaring pass whose output is compiled and run to prove it means the
  same thing.

## What You'll Learn

| Exercise | Stage | The point |
|---|---|---|
| 1 | AST | Visitors, spans, why lints run after expansion |
| 2 | MIR | Basic blocks; bounds and overflow checks; what `-O` changes |
| 3 | Borrow checking | E0382 ... E0597: what each means and the right fix; JSON diagnostics |
| 4 | Lowering | `for` and `?` are sugar; `VisitMut` |
| Bonus | Lints | A liveness-style check, compared with rustc's |

## Key Concepts

### The pipeline

| Stage | What happens | See it |
|---|---|---|
| Parsing | tokens -> AST | `syn` (Ex 1, 4); `rustc -Zunpretty=ast-tree` (nightly) |
| Expansion | macros, `#[derive]` | `cargo expand` |
| HIR | desugared AST: no `for`, no `?` | `rustc -Zunpretty=hir` (nightly) |
| Type check | inference, trait resolution | error messages |
| MIR | control-flow graph; **borrow checking** | `--emit=mir` (Ex 2) |
| Codegen | LLVM IR, optimizations, machine code | `--emit=llvm-ir`, `--emit=asm`, godbolt.org |

### Reading MIR

```text
fn first(_1: &[u8]) -> u8 {
    bb0: {
        _2 = const 0_usize;
        _3 = PtrMetadata(copy _1);
        _4 = Lt(copy _2, copy _3);
        assert(move _4, "index out of bounds: ...") -> [success: bb1, unwind continue];
    }
    bb1: {
        _0 = copy (*_1)[_2];
        return;
    }
}
```

`_0` is the return value, `_1..` the arguments, then the locals. Each
`bbN` ends in exactly one terminator. The format is unstable.

### Machine-readable diagnostics

`rustc --error-format=json` (or `cargo build --message-format=json`) prints
one JSON object per diagnostic:
- `level`;
- `code.code`;
- `message`;
- `spans` with file, line and column;
- the rendered text.

Editors, CI annotations and `cargo fix` are built on it.

## Common Pitfalls

1. **Parsing compiler output strictly** -- MIR and `rendered` text change between versions; key on stable markers
2. **Expecting `syn` to know types** -- `x.unwrap()` might be a user's own method; clippy has the types, a syntax tool doesn't
3. **Columns** -- proc-macro2's are 0-based; editors and rustc are 1-based. Line numbers need the `span-locations` feature outside a macro
4. **Rewriting parents before children** -- visit the children first, or nested loops are left unrewritten
5. **Unhygienic generated names** -- `__iter` twice in nested loops shadows; number them
6. **Fixing a borrow error with `clone()` everywhere** -- sometimes right (E0382), often hiding a design problem (E0502: finish reading, then write)

## Running This Module

```bash
cargo run  -p m39-compiler-internals -- 1                       # your code (1-4, bonus, all)
cargo test -p m39-compiler-internals-tests --features mine      # test your code
cargo run  -p m39-compiler-internals-solution -- all
cargo run  -p m39-compiler-internals-solution -- lint src/*.rs  # lint real files
cargo run  -p m39-compiler-internals-solution -- mir file.rs    # summarize a file's MIR
cargo test -p m39-compiler-internals-tests                      # 11 tests against the solution
```

The tests for Exercises 2-4 and the bonus run your installed `rustc`, which
works offline. The expected MIR counts were checked with rustc 1.98. The
format is unstable, so a future compiler may need the parser adjusted.

## About This Module's Exercises

This module had no `exercises.md` in the original repository. It was written
to match the format of earlier modules and the exercises in
[the phase README](../README.md).

| Phase README exercise | Covered by |
|---|---|
| inspect MIR | Ex 2 |
| a custom clippy lint | Ex 1, the same visitor model on `syn` |
| contribute a fix to rustc | an optional pointer in exercises.md |
| understand a borrow checker error | Ex 3 |

A real clippy lint or a rustc patch needs nightly, a multi-gigabyte
checkout, and an upstream project. Those steps are described in
exercises.md as optional and were not run here. Desugaring (Ex 4) and the
unused-variable check (bonus) were added for HIR lowering and lints.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[40 · Contributing](../40-contributing/)
