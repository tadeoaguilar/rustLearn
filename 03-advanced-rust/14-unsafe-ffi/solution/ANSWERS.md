# Answers · 14 Unsafe & FFI

## Exercise 1

**Why doesn't `(&mut slice[..mid], &mut slice[mid..])` compile?**

`error[E0499]: cannot borrow '*slice' as mutable more than once at a time`.
The borrow checker tracks borrows of whole values (and of struct fields), not
of index ranges. It can't prove `..mid` and `mid..` are disjoint, so it treats
both as borrowing all of `*slice`. We can prove it, so we build the two slices
from a raw pointer — after an `assert!` that guarantees the proof's premise.

## Exercise 2

**Why does `CString::new("a\0b")` fail?**

C strings end at the first NUL byte. `"a\0b"` would arrive in C as `"a"` —
silent truncation, or worse if the length matters (think file paths or
permission checks). `CString::new` refuses with a `NulError` instead.

**What goes wrong if the comparator panics?**

A panic unwinding out of an `extern "C"` function would have to pass through
`qsort`'s C stack frames, which have no unwind information: undefined
behaviour. Rust prevents it by aborting the process at the boundary.
`extern "C-unwind"` allows unwinding when *both* sides are built to support
it (e.g. C++ compiled with exceptions); for plain C, catch the panic with
`std::panic::catch_unwind` inside the callback, or make it impossible.

## Exercise 3

**Should `Counter` be `Send`? `Sync`?**

The raw pointer makes it neither, automatically — the safe default.
`Send` (moving it to another thread) would be sound: the C code keeps no
thread-local state, so `unsafe impl Send for Counter {}` is justified.
`Sync` (sharing `&Counter` between threads) would *also* be sound for the
API as written, because `add` takes `&mut self` — so two threads can never
call it at once through shared references. But if `add` took `&self` (as many
C-wrapping APIs are tempted to), concurrent `value += n` in C would be a data
race. Get the receivers right first, then decide on `Sync`.

## Exercise 4

**Why must the C side call your free function instead of `free()`?**

Memory must be released by the allocator that allocated it. `CString::into_raw`
memory comes from Rust's global allocator — which might be the system
`malloc`, or jemalloc, or the counting allocator from Exercise 6, and even with
`malloc` Rust is allowed to allocate with a different size or alignment than C
would assume. `CString::from_raw` re-creates the `CString` so it is dropped by
the right allocator with the right layout.

## Exercise 6

**Why is it sound to return `&str` tied to `&self` from `Arena::alloc_str`
even though `chunks` is mutated later?**

The returned bytes live inside a `Box<[u8]>` — a separate heap allocation
whose address never changes. Pushing a new chunk may reallocate the *outer*
`Vec<Box<[u8]>>`, which moves the `Box` pointers but not the chunks they point
to. We never write to bytes already handed out, and chunks are only freed when
the arena is dropped, which the borrow checker won't allow while any returned
`&str` is alive.

**What would break with `Vec<u8>` chunks that grow?**

`Vec::push` / `extend` may reallocate the buffer to a new address and free the
old one. Every `&str` previously returned would then point into freed memory:
a use-after-free, invisible to the borrow checker because the `unsafe` code
promised otherwise.
