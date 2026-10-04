# Getting Started with 04 · Structs & Enums

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m04-structs-enums-solution -- all
```

## What Is Already Here

```
04-structs-enums/
├── README.md               # The concepts
├── exercises.md            # The work: 9 exercises plus a bonus
├── GETTING_STARTED.md      # This file
│
├── exercise/               # ← YOUR WORKSPACE (package m04-structs-enums)
│   └── src/
│       ├── main.rs         #   ready-made runner: `-- 1` … `-- 9`, `-- bonus`, `-- all`
│       └── ex01_structs.rs … bonus_json.rs
│                           #   types are already declared; method bodies are todo!()
│
├── solution/               # ← REFERENCE IMPLEMENTATION (m04-structs-enums-solution)
│   ├── ANSWERS.md
│   └── src/
│
└── tests/                  # ← 25 tests (m04-structs-enums-tests)
```

The structs and enums are already declared in `exercise/src` — exactly as in
`exercises.md` plus a few `#[derive]`s the tests need (`PartialEq` so results
can be compared with `assert_eq!`, `Debug` so failures can be printed). Your job
is the `impl` blocks and functions.

## The Commands You Need

```bash
# Run your own work
cargo run -p m04-structs-enums -- 3

# Check your work
cargo test -p m04-structs-enums-tests --features mine

# One exercise's tests (names start ex1_ … ex9_, bonus_)
cargo test -p m04-structs-enums-tests --features mine ex8_

# The reference solution
cargo run -p m04-structs-enums-solution -- 1       # Basic structs
cargo run -p m04-structs-enums-solution -- 2       # Tuple and unit structs
cargo run -p m04-structs-enums-solution -- 3       # Methods
cargo run -p m04-structs-enums-solution -- 4       # Basic enums
cargo run -p m04-structs-enums-solution -- 5       # Option<T>
cargo run -p m04-structs-enums-solution -- 6       # Pattern matching
cargo run -p m04-structs-enums-solution -- 7       # Result<T, E>
cargo run -p m04-structs-enums-solution -- 8       # Game state machine
cargo run -p m04-structs-enums-solution -- 9       # Shape calculator
cargo run -p m04-structs-enums-solution -- bonus   # JSON-like value
cargo run -p m04-structs-enums-solution -- all     # Everything

cargo test -p m04-structs-enums-tests              # always green
```

## Tips for Exercise 8

Two ways to replace `*self` while reading its fields:

```rust
// 1. Match on *self -- works when the fields are Copy (u32, u8 here)
if let GameState::Playing { level, score, lives } = *self {
    *self = GameState::Paused { level, score, lives };
}

// 2. Take the old value out, leaving a placeholder -- works for any fields
let old = std::mem::replace(self, GameState::Menu);
*self = match old { ... };
```

To change one field *in place*, match on `self` itself:

```rust
if let GameState::Playing { score, .. } = self {   // score: &mut u32
    *score += points;
}
```

## If You Get Stuck

1. **`borrow of partially moved value`** — a `..other` update or a `match` moved a field out.
2. **`cannot assign to *self because it is borrowed`** — copy the fields out first (tip 1 above).
3. **`non-exhaustive patterns`** — add the missing variant, or `_ =>`.
4. **`cannot move out of *self which is behind a mutable reference`** — use `std::mem::replace` or `std::mem::take`.
5. Compare against `solution/src/` — same file and function names.
