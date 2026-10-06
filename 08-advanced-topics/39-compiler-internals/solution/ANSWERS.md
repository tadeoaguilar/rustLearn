# Answers · 39 Compiler Internals

## Exercise 1: Why `println!("{}", x as u8)` isn't reported

To `syn`, a macro invocation is a path plus an opaque token stream. The
arguments only *become* expressions when the macro expands, and
`format_args!` is expanded by the compiler itself. A syntax-level tool sees
`x as u8` only as tokens. It could try parsing them as expressions, but
that works only for macros whose syntax it knows.

Clippy's lints are `LateLintPass`es. They run on HIR, *after* macro
expansion and type checking, so the cast inside `println!` is an ordinary
`ExprKind::Cast` with known types. (That's also how clippy can lint only
casts that truncate, e.g. `u64 as u8` but not `u8 as u64`; a syntax tool
doesn't know `x`'s type.) Clippy then checks `expr.span.from_expansion()`
to avoid linting code a macro generated, which a user can't change.

## Exercise 2: Does the release binary check bounds?

MIR optimizations are modest, and the bounds check is still there in
release MIR. LLVM runs later. It sees that `i` comes from `0..v.len()`, so
`i < v.len()` always holds, and it deletes the check. To confirm, look at
the machine code, not the MIR:
- `cargo asm` (cargo-show-asm);
- `RUSTFLAGS=--emit=asm`;
- `rustc -O --emit=llvm-ir`, then look for `panic_bounds_check`.

godbolt.org does this for a single function. Module 36 measured it: the
indexed and iterator sums run at the same speed. A check LLVM *can't*
remove (an index from data) is visible as a call to
`core::panicking::panic_bounds_check`.

## Exercise 3: Why E0499 is rejected for `v[0]` and `v[1]`

`v[0]` on a `Vec` is `*IndexMut::index_mut(&mut *v, 0)`, a method call that
takes `&mut` to the *whole* vector. The borrow checker reasons about
places (`v`, `v.field`), not about values: it doesn't know that two
`index_mut` calls with different integers return disjoint memory, because
that is a fact about the implementation, not the types. Struct fields *are*
disjoint places, so `let a = &mut s.x; let b = &mut s.y;` is accepted.

`split_at_mut` is where that knowledge lives. Its body creates two slices
from one raw pointer:
`from_raw_parts_mut(ptr, mid)` and `from_raw_parts_mut(ptr.add(mid), len - mid)`.
That needs `unsafe`, because the compiler can't prove the two don't overlap.
The `mid <= len` assertion is the human-checked proof. Every "two mutable
references into one collection" API (`split_at_mut`, `iter_mut`,
`get_disjoint_mut`) is a small audited `unsafe` block behind a safe
signature.

## Exercise 4: Why `match` instead of `let`

Temporaries. In `for x in make_vec().iter() { .. }`, the `Vec` returned by
`make_vec()` is a temporary. In a `match` scrutinee, temporaries live until
the end of the whole `match`, which here contains the loop. With
`let mut iter = make_vec().iter();`, the temporary `Vec` would be dropped at
the end of the `let` statement, leaving `iter` borrowing a freed value
(E0716). The `match` form makes the loop behave as if the iterator
expression were evaluated *around* the loop body.
