# Getting Started with 35 · Memory Management

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m35-memory-management-solution -- all
```

allocates from an arena (0 heap allocations for 1000 values), reuses slab
slots and pooled buffers, compares a naive and a reserved string join by
allocation count (the program's own global allocator counts them), prints
struct layouts, compares AoS and SoA, and walks an arena tree. For
meaningful timings in part 5, add `--release`.

## What Is Already Here

```
35-memory-management/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/src/              # ← YOUR WORKSPACE (package m35-memory-management)
│   ├── ex01_bump.rs           #   Ex 1  (GlobalBump::new provided: statics need it)
│   ├── ex02_pool.rs           #   Ex 2
│   ├── ex03_counting.rs       #   Ex 3  (the forwarding GlobalAlloc impl provided)
│   ├── ex04_layout.rs         #   Ex 4
│   ├── ex05_soa.rs            #   Ex 5
│   ├── bonus_arena_tree.rs    #   bonus
│   └── main.rs                #   provided: installs your Counting as the global allocator
├── solution/                  # ← REFERENCE (m35-memory-management-solution) + ANSWERS.md
└── tests/
    ├── src/exercises.rs       # ← 15 tests
    └── tests/counting.rs      # ← 4 tests with your allocator as the global allocator
```

## The Commands You Need

```bash
cargo test -p m35-memory-management-tests --features mine ex1_
cargo test -p m35-memory-management-tests --features mine --test counting
cargo test -p m35-memory-management-tests --features mine
cargo test -p m35-memory-management-tests                    # the solution: always green
```

## If You Get Stuck

1. **A test binary dies with SIGABRT** -- something panicked inside an allocator (or allocated in it). In `record_alloc`, use only atomics and `const` thread-locals, and `try_with`.
2. **Misaligned allocations** -- align `base + offset`, then convert back to an offset.
3. **The pool never recycles** -- `Pooled`'s `Drop` must reset the value and push it back.
4. **A stale handle still works** -- `remove` must bump the slot's generation.
5. **`layout_c` disagrees with `offset_of!`** -- round each offset up to the field's alignment *before* placing it, and the final size up to the struct's alignment.
6. **SoA looks slower** -- you're in debug mode, or indexing with bounds checks; use zipped iterators.
7. Compare with `solution/src/` -- same file and function names.
