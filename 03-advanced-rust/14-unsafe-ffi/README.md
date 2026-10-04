# 14 · Unsafe Rust & FFI

## Overview

Safe Rust guarantees no use-after-free, no data races, no out-of-bounds
access — by refusing programs it can't prove correct. `unsafe` is how you
write the ones it can't prove: talking to C, implementing data structures with
raw memory, writing allocators. It doesn't switch the borrow checker off; it
unlocks five extra operations and makes **you** responsible for their rules.

The skill this module teaches isn't writing unsafe code. It's writing a
**safe API around a small unsafe core**, so that no caller — however careless
— can trigger undefined behaviour.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Raw pointers | `*const`/`*mut`, `ptr.add`, `split_at_mut` from scratch |
| 2 | Calling libc | `unsafe extern "C"`, `safe fn` items, `CString`, a `qsort` callback |
| 3 | Your own C library | `#[repr(C)]`, opaque handles + `Drop`, closures through `void *user_data` |
| 4 | Rust called from C | `#[unsafe(no_mangle)]`, who frees what |
| 5 | `StackVec` | `MaybeUninit<T>` and an invariant that makes it sound |
| 6 | Allocators | `GlobalAlloc`, `#[global_allocator]`, a bump arena |
| 7 | System calls | POSIX `getpid`/`gethostname`, `errno` |
| Bonus | Miri | Finding UB your tests can't see |

## Key Concepts

### The five unsafe superpowers

1. Dereference a raw pointer
2. Call an `unsafe` function (including foreign functions)
3. Implement an `unsafe` trait (`Send`, `Sync`, `GlobalAlloc`)
4. Access or modify a `static mut`
5. Access a `union` field

Everything else — borrow checking, type checking — still applies inside
`unsafe { }`.

### Safe API, unsafe core

```rust
pub fn my_split_at_mut(slice: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
    assert!(mid <= slice.len());          // makes the unsafe part impossible to misuse
    let ptr = slice.as_mut_ptr();
    // SAFETY: both ranges are in bounds and don't overlap.
    unsafe { (from_raw_parts_mut(ptr, mid), from_raw_parts_mut(ptr.add(mid), slice.len() - mid)) }
}
```

Without the `assert!` the function would still compile — and safe code could
cause UB through it. A function is only allowed to be safe (`pub fn`, not
`pub unsafe fn`) if **no** input can make it misbehave. Clippy's
`not_unsafe_ptr_arg_deref` catches the commonest violation.

### FFI checklist

| Concern | Rust tool |
|---|---|
| Same struct layout as C | `#[repr(C)]` |
| C strings | `CString` (Rust → C), `CStr` (C → Rust) — interior NULs are errors |
| Opaque C types | a zero-sized `#[repr(C)] struct` used only behind pointers |
| C resources | a wrapper with `Drop` calling the C free function — exactly once |
| Callbacks with state | a generic `extern "C" fn trampoline<F>` + `user_data: *mut c_void` |
| Panics | never let one cross into C; `extern "C"` aborts if it tries |
| Memory ownership | free with the allocator that allocated |

### Rust 2024 made unsafety more explicit

- `unsafe extern "C" { ... }` — declaring foreign functions is itself a promise
  that the signatures are right
- `safe fn` inside such a block for functions that can't cause UB
- `#[unsafe(no_mangle)]` / `#[unsafe(export_name = "...")]` — symbol clashes are UB
- an `unsafe fn` body is no longer an implicit `unsafe` block

### `MaybeUninit` and invariants

`StackVec` stores `[MaybeUninit<T>; N]` and one rule: *slots `..len` are
initialised, slots `len..` aren't*. Every method is short because it only has
to preserve that rule — and `Drop` drops exactly `..len`. A test with a drop
counter checks it: three pushes, one pop, drop → exactly three drops.

## Common Pitfalls

1. **A safe function that trusts a raw pointer argument** — it must be `unsafe fn`
2. **Missing `#[repr(C)]`** — Rust is free to reorder fields
3. **`CString::new(s).unwrap().as_ptr()`** — the `CString` is dropped at the end of the statement; the pointer dangles
4. **Freeing Rust memory with C's `free()`** (or the reverse)
5. **`todo!()` / `unwrap()` in an `extern "C" fn`** — a panic there aborts the process
6. **Reading a `MaybeUninit` twice** — `assume_init_read` moves the value out; the second read is a double drop
7. **Trusting tests alone** — UB can pass every test; run Miri

## Running This Module

```bash
cargo run  -p m14-unsafe-ffi -- 3                        # your code
cargo test -p m14-unsafe-ffi-tests --features mine       # test your code
cargo run  -p m14-unsafe-ffi-solution -- all             # the reference solution
cargo test -p m14-unsafe-ffi-tests                       # 17 tests against the solution
```

Needs a C compiler (Xcode Command Line Tools on macOS, `build-essential` on
Linux). Exercise 7 is Unix-only.

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of the others and the exercise list in
[the phase README](../README.md) (bindings for a C library, a custom
allocator, a safe wrapper around unsafe code, system libraries).

- **The C library** (`csrc/shapes.c`) is provided in both `exercise/` and
  `solution/`; a `build.rs` compiles it with the `cc` crate.
- **Symbol prefixes**: the tests crate links the exercise and solution crates
  into one binary, so their C and `#[no_mangle]` symbols must not clash. The
  solution's copy is compiled with `SHAPES_PREFIX=sol_`, which is why its
  bindings carry `#[link_name = "sol_..."]` and its exports
  `#[unsafe(export_name = "sol_...")]`. Your crate uses the plain names.
- **Skeleton placeholders**: `extern "C"` functions in `exercise/` return
  dummy values instead of `todo!()` — a panic can't unwind into C. And
  `#[global_allocator]` is commented out in `exercise/src/main.rs` until your
  allocator works, or the program couldn't start.
- **`rust_checksum`/`rust_greeting` are `unsafe extern "C" fn`**, not plain
  `extern "C" fn` — see "Safe API, unsafe core" above.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[15 · Concurrency](../15-concurrency/)
