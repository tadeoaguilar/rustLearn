# Exercises: Memory Management

Rust frees memory for you, but it doesn't decide *how* memory is obtained,
laid out or reused -- you do. This module writes allocators (a bump arena,
a slab, an object pool), measures allocations with a counting global
allocator, reasons about layout and padding, and shows how data layout
decides cache behaviour.

**Setup**: no dependencies. Exercise 3's integration tests
(`tests/tests/counting.rs`) install your counting allocator as the global
allocator of their test binary. Timings in Exercise 5 only mean something
in release mode (`cargo run --release ... -- 5`).

---

## Exercise 1: A Bump Allocator

**Difficulty**: Medium
**Time**: 1.5 hours

**Learning Objectives**:
- Align addresses; allocate from a buffer
- Hand out several `&mut` from `&self` soundly
- Implement `GlobalAlloc` lock-free

1. `align_up(n, align)` (power-of-two alignment, `None` on overflow).
2. `Bump`: `alloc(layout)` -- align the *address*, `None` when full;
   `alloc_value`, `alloc_slice`, `alloc_str` (for `Copy` data: the arena
   never runs destructors); `used`, `remaining`, `reset(&mut self)`.
3. `GlobalBump<N>`: `GlobalAlloc` over a fixed buffer, the offset claimed
   with `compare_exchange_weak`; null when full; `dealloc` does nothing.

**Question**: why does `reset` take `&mut self` while `alloc` takes `&self`?

---

## Exercise 2: A Slab and an Object Pool

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- Replace pointers with generational indices
- Reuse slots through a free list
- Recycle objects with an RAII guard

1. `Slab<T>`: `insert` (reuse the most recently freed slot, else grow),
   `get`, `get_mut`, `remove` (bump the generation: old handles go stale),
   `len`, `capacity`, `iter`.
2. `ObjectPool<T>`: `take` returns a `Pooled` guard -- a recycled object or
   a new one from `make`; dropping the guard runs `reset` and returns it;
   `created`, `available`.

**Question**: what bug does the generation prevent?

---

## Exercise 3: Counting Allocations

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- Wrap the system allocator
- Count globally and per thread without allocating
- Find allocation hot spots

1. `Counting<A>`: forward `alloc`/`dealloc`/`realloc` to `A` (provided),
   and count in `record_alloc`/`record_dealloc`: allocations, frees, live
   bytes, peak (`fetch_max`), plus the per-thread counters.
   `snapshot`. (A realloc counts as a free plus an allocation.)
2. `measure(f)`: this thread's allocations and bytes during `f`.
3. `join_naive` / `join_reserved` (one allocation: reserve the exact
   length) / `count_long_words` (zero allocations).

**Question**: why must the per-thread counters be `const`-initialised
thread-locals?

---

## Exercise 4: Layout

**Difficulty**: Easy
**Time**: 45 minutes

**Learning Objectives**:
- Compute `repr(C)` layouts by hand
- Reduce padding by reordering
- Know which types have niches

1. `layout_c(fields)`: offsets, size, alignment (fields are `(size, align)`).
2. `padding`, `best_order` (decreasing alignment).
3. `option_is_free::<T>()`. The tests compare your results with
   `size_of`, `align_of` and `offset_of!` on real structs.

---

## Exercise 5: Cache-Friendly Layouts

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- Convert an array of structs to a struct of arrays
- See when each wins
- Walk 2D data in memory order

1. `ParticlesAos`: `step`, `kinetic_energy`, `mean_x`, `to_soa`.
2. `ParticlesSoa`: the same, and `to_aos`. Results must be identical.
3. `sum_row_major` / `sum_col_major`.

Then time them: `cargo run --release -p m35-memory-management -- 5`.

**Question**: SoA wins clearly for `mean_x` but barely for `step`. Why?

---

## Bonus: A Tree in an Arena

**Difficulty**: Easy
**Time**: 30 minutes

`Tree<T>`: nodes in a `Vec`, linked by `NodeId`; `add_child`,
`path_to_root`, `depth`, `descendants` (iterative pre-order -- a 100,000-deep
tree must work), `subtree_sum`.
