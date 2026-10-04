# Getting Started with 11 · Advanced Lifetimes

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m11-lifetimes-advanced-solution -- all
```

## What Is Already Here

```
11-lifetimes-advanced/
├── README.md               # The concepts
├── exercises.md            # The work: 7 exercises plus a bonus
├── GETTING_STARTED.md      # This file
│
├── exercise/               # ← YOUR WORKSPACE (package m11-lifetimes-advanced)
│   └── src/ex01_annotations.rs … bonus_str_split.rs
│                           #   bodies are todo!(); FIVE signatures are deliberately too strict
│
├── solution/               # ← REFERENCE IMPLEMENTATION (m11-lifetimes-advanced-solution)
│   ├── ANSWERS.md
│   └── src/                #   broken versions quoted with their real errors
│
└── tests/
    ├── src/exercises.rs    # ← 18 behaviour tests
    └── src/signatures.rs   # ← 6 lifetime checks (compile-time), feature `signatures`
```

## The Commands You Need

```bash
# Run your own work
cargo run -p m11-lifetimes-advanced -- 1

# Behaviour: do your functions return the right values?
cargo test -p m11-lifetimes-advanced-tests --features mine

# Lifetimes: are your signatures loose enough? (compile errors = failures)
cargo test -p m11-lifetimes-advanced-tests --features mine,signatures

# The reference solution
cargo run -p m11-lifetimes-advanced-solution -- 1       # Annotations and elision
cargo run -p m11-lifetimes-advanced-solution -- 2       # Structs that borrow
cargo run -p m11-lifetimes-advanced-solution -- 3       # Zero-copy HTTP parser
cargo run -p m11-lifetimes-advanced-solution -- 4       # Borrowing iterators
cargo run -p m11-lifetimes-advanced-solution -- 5       # 'static
cargo run -p m11-lifetimes-advanced-solution -- 6       # HRTB
cargo run -p m11-lifetimes-advanced-solution -- 7       # Fixing errors
cargo run -p m11-lifetimes-advanced-solution -- bonus   # StrSplit
cargo run -p m11-lifetimes-advanced-solution -- all     # Everything

cargo test -p m11-lifetimes-advanced-tests              # always green
```

## Suggested Workflow

1. Implement the bodies until the behaviour tests pass.
2. Run with `--features mine,signatures`. You'll see five errors like:
   ```text
   error[E0597]: `excerpt` does not live long enough
     --> tests/src/signatures.rs
   ```
3. For each one, read the test: *what* does it drop, and what does it keep
   using afterwards? Then change your signature so the result doesn't depend
   on the thing that was dropped.

## Reading Lifetime Errors

The useful parts of a borrow-checker error are the three labels:

```text
   |         let excerpt = Excerpt::first_sentence(&novel);
   |             ------- binding `excerpt` declared here
   |         word = excerpt.longest_word();
   |                ^^^^^^^ borrowed value does not live long enough
   |     }
   |     - `excerpt` dropped here while still borrowed
   |     assert_eq!(word, "Ishmael");
   |     ---------------------------- borrow later used here
```

"borrowed value" + "dropped here" + "later used here". Ask: *should* `word`
depend on `excerpt` at all? (No — it points into `novel`.)

## If You Get Stuck

1. **`missing lifetime specifier`** — the elision rules can't decide; name the lifetime that the output really borrows from.
2. **`does not live long enough`** — either a signature ties the result to the wrong input, or you really are keeping a reference to something dying.
3. **`cannot return value referencing local variable`** — return an owned value; no annotation helps.
4. **`lifetime may not live long enough` in an iterator** — the item must be `&'a T`; for `&'a mut T`, take the slice out with `mem::take`.
5. Compare against `solution/src/` — same file and function names.
