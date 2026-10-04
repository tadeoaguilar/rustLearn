# Getting Started with 06 · Traits & Generics

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m06-traits-generics-solution -- all
```

## What Is Already Here

```
06-traits-generics/
├── README.md               # The concepts
├── exercises.md            # The work: 7 exercises plus a bonus
├── GETTING_STARTED.md      # This file
│
├── exercise/               # ← YOUR WORKSPACE (package m06-traits-generics)
│   └── src/ex01_basic_traits.rs … bonus_shapes.rs
│                           #   traits, structs and impl headers are declared;
│                           #   method bodies are todo!()
│
├── solution/               # ← REFERENCE IMPLEMENTATION (m06-traits-generics-solution)
│
└── tests/                  # ← 18 tests (m06-traits-generics-tests)
```

## The Commands You Need

```bash
# Run your own work
cargo run -p m06-traits-generics -- 1

# Check your work
cargo test -p m06-traits-generics-tests --features mine

# One exercise's tests (names start ex1_ … ex7_, bonus_)
cargo test -p m06-traits-generics-tests --features mine ex4_

# The reference solution
cargo run -p m06-traits-generics-solution -- 1       # Basic traits
cargo run -p m06-traits-generics-solution -- 2       # Generic functions
cargo run -p m06-traits-generics-solution -- 3       # Trait bounds
cargo run -p m06-traits-generics-solution -- 4       # Operator overloading
cargo run -p m06-traits-generics-solution -- 5       # Associated types
cargo run -p m06-traits-generics-solution -- 6       # Trait objects
cargo run -p m06-traits-generics-solution -- 7       # Generic stack
cargo run -p m06-traits-generics-solution -- bonus   # Shape system
cargo run -p m06-traits-generics-solution -- all     # Everything

cargo test -p m06-traits-generics-tests              # always green
```

## See Monomorphisation for Yourself

Generic code is copied per concrete type. `cargo-show-asm` lets you see it:

```bash
cargo install cargo-show-asm
cargo asm -p m06-traits-generics-solution --lib largest
```

It lists one `largest::<T>` per type the crate instantiates it with.

## If You Get Stuck

1. **`binary operation > cannot be applied to type T`** — add a `T: PartialOrd` bound.
2. **`the trait X is not implemented for Y`** — read which bound is missing; add it or implement it.
3. **`the trait cannot be made into an object`** — a method is generic or returns `Self`; move it to a separate trait or add `where Self: Sized`.
4. **`only traits defined in the current crate can be implemented for types defined outside`** — the orphan rule; wrap the type in a newtype.
5. **`multiple applicable items in scope`** — two traits define the same method; call `Trait::method(&value)`.
6. Compare against `solution/src/` — same file and function names.
