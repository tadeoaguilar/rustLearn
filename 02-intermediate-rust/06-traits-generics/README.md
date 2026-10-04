# 06 · Traits & Generics

## Overview

Traits describe what a type can **do**; generics let one function or struct
work with **any** type that can do it. Together they replace inheritance: there
are no base classes in Rust, only behaviour that types opt into. The central
design question of this module is *static or dynamic dispatch* — and when to
pick which.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Traits | Required vs default methods; `impl Trait` arguments |
| 2 | Generic functions | `largest<T: PartialOrd>`, generic structs |
| 3 | Trait bounds | `where` clauses, conditional `impl` blocks, `impl Trait` returns |
| 4 | Operator overloading | `Add`, `Sub`, `Mul<f64>`, `Neg`, `AddAssign`, `Sum`, the orphan rule |
| 5 | Associated types | `Iterator::Item`; a `Graph` trait with `Node`/`Edge` |
| 6 | Trait objects | `Vec<Box<dyn Plugin>>` and fat pointers |
| 7 | Generic stack | `Stack<T>`, `IntoIterator` for owned and borrowed, `FromIterator` |
| Bonus | Shape system | Supertraits, upcasting, blanket impls |

## Key Concepts

### Static vs dynamic dispatch

```rust
fn total_area<T: Shape>(shapes: &[T]) -> f64          // static
fn total_area_dyn(shapes: &[Box<dyn Drawable>]) -> f64  // dynamic
```

| | Generics (`T: Trait`) | Trait objects (`dyn Trait`) |
|---|---|---|
| Resolved | at compile time (monomorphised) | at runtime (vtable) |
| One collection holds | one concrete type | many types |
| Speed | inlinable, no indirection | one pointer hop per call |
| Binary size | a copy per type used | one copy |
| Restrictions | none | trait must be dyn-compatible (no generic methods, no `Self` returns) |

Default to generics; reach for `dyn` when you need a heterogeneous collection
or a plugin boundary.

### Bound only what you use

The exercise's `compare_and_print<T: PartialOrd + Display + Clone>` calls
`b.clone()` on a value it already owns. Removing `Clone` makes the function
accept *more* types for free. Every bound is a restriction on callers.

### Associated type or generic parameter?

```rust
trait Iterator { type Item; ... }   // one Item per iterator type
trait Mul<Rhs> { type Output; ... } // Vector2D * f64 AND Vector2D * Vector2D
```

If a type should implement the trait **once**, use an associated type. If it
makes sense to implement it **several times** for different inputs, use a
generic parameter.

### The orphan rule

You can implement a trait for a type only if your crate defines the trait *or*
the type. `impl Mul<Vector2D> for f64` is allowed (our `Vector2D` appears);
`impl Display for Vec<i32>` is not. The workaround is a newtype —
`Pretty(&shape)` in the bonus.

## Common Pitfalls

1. **`&list[0]` in a generic function** — panics on empty input; return `Option`
2. **Unnecessary bounds** (`Clone`, `Copy`, `Debug`) — they shrink what callers can pass
3. **`Box<dyn Trait>` everywhere** — generics are usually simpler and faster
4. **Generic methods on a trait you want as `dyn`** — not dyn-compatible
5. **Two traits with the same method name in scope** — call it as `Trait::method(&x)`

## Running This Module

```bash
cargo run  -p m06-traits-generics -- 4                      # your code, exercise 4
cargo test -p m06-traits-generics-tests --features mine     # test your code
cargo run  -p m06-traits-generics-solution -- all           # the reference solution
cargo test -p m06-traits-generics-tests                     # 18 tests against the solution
```

## Notes on `exercises.md`

- **Exercise 1**: Tasks 1 and 2 each define a trait named `Summary`, and
  Exercises 2 and 3 each define a struct named `Pair`. In one crate those names
  would collide; the solution puts Exercise 1's tasks in modules `task1` and
  `task2`, and each exercise in its own file.
- **Exercise 3, Task 3**: the `Clone` bound and `b.clone()` aren't needed — see
  "Bound only what you use" above.
- **Exercise 6**: `execute` returns its report as a `String` instead of
  printing, so the manager and the tests can observe it. Same for `render` in
  the bonus (`draw` is a default method that prints `render()`).
- **Exercise 5, Task 2** leaves the graph type open. The solution implements a
  directed road map (`Node = String`, `Edge = Road`).
- **Exercise 7 bonus**: `IntoIterator` is implemented for both `Stack<T>` and
  `&Stack<T>`; both yield the top of the stack first.

## Next

[07 · Collections & Iterators](../07-collections-iterators/)
