# 35 · Memory Management

## Overview

Ownership tells Rust *when* to free memory; this module is about *how*
memory is obtained and arranged. A general-purpose allocator is a
compromise; arenas, slabs and pools are faster when you know the lifetime
pattern. Every allocation can be counted by swapping the global allocator.
And layout -- padding, field order, AoS versus SoA -- decides how much of
what the CPU fetches is actually used.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Bump allocator | Alignment; arenas; `GlobalAlloc`, lock-free |
| 2 | Slab, pool | Generational indices; free lists; RAII recycling |
| 3 | Counting allocator | Profiling allocations; per-thread counters |
| 4 | Layout | Size, alignment, padding, niches |
| 5 | Cache-friendly data | AoS vs SoA; access order |
| Bonus | Arena tree | Indices instead of `Rc<RefCell<..>>` |

## Key Concepts

### Allocators

```rust
unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 { ... }      // null on failure, never panic
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) { ... }
}
#[global_allocator]
static ALLOC: MyAllocator = MyAllocator::new();                    // const constructor
```

| Allocator | Alloc | Free | Good for |
|---|---|---|---|
| system (malloc) | general | individual | everything, reasonably |
| bump / arena | pointer bump | all at once | per-frame, per-request data |
| slab / pool | pop free list | push free list | many same-size objects |

### Layout

```
#[repr(C)] struct Padded { flag: bool, value: u64, tag: u8 }   // 24 bytes
           flag [.......] value [........] tag [.......]
#[repr(C)] struct Packed { value: u64, flag: bool, tag: u8 }   // 16 bytes
```

Rust's default layout reorders fields for you; `#[repr(C)]` doesn't.
`Option<&T>`, `Option<Box<T>>`, `Option<NonZeroU32>` cost nothing extra:
`None` uses a forbidden bit pattern (a niche).

### The cache

Memory arrives in 64-byte lines. Code that uses every byte of each line
it pulls in -- contiguous arrays, walked in order -- runs many times faster
than code that strides or chases pointers.

## Common Pitfalls

1. **Panicking in a global allocator** -- it aborts; return null
2. **Allocating inside the allocator** -- infinite recursion (lazy thread-locals, `println!`)
3. **Forgetting alignment** -- align the address, not just the offset
4. **Arena values with destructors** -- arenas don't run them; restrict to `Copy` or track them
5. **Stale indices without generations** -- silent wrong data
6. **Growing collections in a loop** -- `with_capacity` when the size is known
7. **Benchmarking in debug mode** -- measure with `--release`

## Running This Module

```bash
cargo run  -p m35-memory-management -- 1                     # your code (1-5, bonus, all)
cargo test -p m35-memory-management-tests --features mine    # test your code
cargo run  -p m35-memory-management-solution -- all          # every allocation counted
cargo run  --release -p m35-memory-management-solution -- 5  # AoS vs SoA, row vs column timings
cargo test -p m35-memory-management-tests                    # 19 tests against the solution
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the exercises in
[the phase README](../README.md) (a custom bump allocator, a memory pool,
profiling and optimizing memory usage, a cache-friendly data structure).

- **Profiling is done with your own counting allocator**, not valgrind or
  heaptrack (not available everywhere); `dhat` is the crate that adds stack
  traces to the same idea.
- **Timings are printed by the demo, not asserted by the tests**: they vary
  by machine, and timing assertions make flaky tests. The tests check that
  every layout computes the same results.
- **Not covered**: jemalloc/mimalloc (a one-line `#[global_allocator]`
  swap), NUMA, memory-mapped files (module 34).

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

Phase 8: [36 · Performance Optimization](../../08-advanced-topics/36-performance-optimization/)
