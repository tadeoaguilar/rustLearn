# Answers · 03 Ownership & Borrowing

## Exercise 1

**What happens when a value goes out of scope?**

Rust calls `drop` on it. For a `String`, `Vec` or `Box` that frees the heap
memory; for a `File` it closes the handle; for a `MutexGuard` it unlocks. This
is deterministic — it happens at the closing `}`, not "whenever the GC runs" —
and it happens exactly once, because only the owner drops.

**What's the difference between move and copy semantics?**

Both copy the value's bytes on the stack. After a **move** the source variable
is invalid and the compiler stops you using it, because the bytes contain a
pointer to heap memory that only one owner may free. After a **copy** both
variables are valid, because the bytes *are* the whole value (an `i32`, a
`char`). Which one you get depends only on whether the type implements `Copy`.

**Which types implement the Copy trait?**

All integer and float types, `bool`, `char`, shared references `&T`, raw
pointers, function pointers, and tuples/arrays whose elements are all `Copy`.
Your own structs and enums can `#[derive(Copy, Clone)]` if every field is
`Copy`. Never `Copy`: `String`, `Vec<T>`, `Box<T>`, `&mut T`, or anything that
implements `Drop`.

## Exercise 5, Task 1

| Value | Copy? |
|---|---|
| `a: i32` | yes |
| `b: f64` | yes |
| `c: bool` | yes |
| `d: char` | yes |
| `e: &str` | yes — copying a shared reference just copies the pointer |
| `f: String` | no — `clone()` |
| `g: Vec<i32>` | no — `clone()` |
| `h: (i32, String)` | no — a tuple is Copy only if all its fields are |

`tests/src/exercises.rs` checks this table with `fn assert_copy<T: Copy>() {}`,
a function that only compiles for Copy types.

## Exercise 5, Task 3

**Why can't `Person { name: String, age: u32 }` derive Copy?**

`error[E0204]: the trait 'Copy' cannot be implemented for this type`. A bitwise
copy of `Person` would copy `name`'s heap pointer, giving two values that both
think they own — and will both free — the same buffer.

## Exercise 6, Task 4

**Can we call `get` and `set` at the same time?**

No. `get(&self)` returns a reference *into* the HashMap, which keeps the whole
cache immutably borrowed while that reference is alive. `set(&mut self)` needs
an exclusive borrow, and inserting might make the map reallocate and move the
value we're pointing at. Fix: clone the value out (ending the borrow) before
calling `set`. See `Cache::copy_value`.
