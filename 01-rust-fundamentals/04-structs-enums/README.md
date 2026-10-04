# 04 · Structs & Enums

## Overview

Structs group data that belongs together — a User has a name **and** an email.
Enums describe data that is one of several shapes — a Shape is a Circle **or**
a Rectangle — and each shape can carry different data. With `match` to take
them apart, these two let you design types where **invalid states can't be
represented at all**, which is the single most useful habit Rust teaches.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Structs | Field init shorthand, `..base` update syntax, partial moves |
| 2 | Tuple & unit structs | Newtypes: same data, different type |
| 3 | Methods | `&self`, `&mut self`, `self`, and associated functions |
| 4 | Enums | Variants that carry different data |
| 5 | `Option<T>` | No null; `map`, `and_then`, `unwrap_or_else`, `find`, `position` |
| 6 | Patterns | `|`, ranges, destructuring, guards, `if let`, `while let`, `let else` |
| 7 | `Result<T, E>` | Your own error enum, `Display`, and `?` |
| 8 | Game state | An enum as a state machine |
| 9 | Shapes | Methods on enums; mutating through `&mut` matches |
| Bonus | JSON value | A recursive enum, `Display`, pretty printing, `From` |

## Key Concepts

### Make invalid states unrepresentable

Compare two designs for Exercise 8:

```rust
// A struct of flags: nothing stops "paused" AND "game over" AND "in the menu".
struct Game { in_menu: bool, paused: bool, game_over: bool, score: Option<u32>, ... }

// An enum: exactly one state at a time, and each carries only its own data.
enum GameState {
    Menu,
    Playing { level: u32, score: u32, lives: u8 },
    Paused  { level: u32, score: u32, lives: u8 },
    GameOver { final_score: u32 },
}
```

With the enum there's no score to read in the Menu — the compiler won't let you.

### Method receivers are a contract

| Signature | Caller writes | Meaning |
|---|---|---|
| `fn new(..) -> Self` | `Rectangle::new(1, 2)` | associated function (constructor) |
| `fn area(&self)` | `r.area()` | reads |
| `fn scale(&mut self, ..)` | `r.scale(2)` | modifies in place |
| `fn to_square(self)` | `r.to_square()` | consumes `r` |

### Matching through references

```rust
pub fn scale(&mut self, factor: f64) {
    match self {                                   // self: &mut Shape
        Shape::Circle { radius } => *radius *= factor,  // radius: &mut f64
        ...
    }
}
```

Matching on a reference makes every binding a reference of the same kind
("default binding modes"). That's how you update an enum's fields in place.

### `Option` combinators replace `if x != null`

| You want | Use |
|---|---|
| a default | `unwrap_or(v)`, or `unwrap_or_else(|| ...)` if it's expensive |
| to transform the inside | `map` |
| a step that can itself fail | `and_then` |
| to keep it only if… | `filter` |
| to turn it into a `Result` | `ok_or(err)` |
| to bail out early | `?` inside a function returning `Option` |

## Common Pitfalls

1. **`..user1` moves `user1`'s non-Copy fields** — `user1` can't be used as a whole afterwards
2. **Forgetting `mut` on the binding** — there is no per-field `mut`
3. **`unwrap()` in real code** — use `expect("reason")`, `?`, or a combinator
4. **Guards and exhaustiveness** — the compiler doesn't reason about guards; you still need `_`
5. **HashMap order** — it's random; sort keys before printing or comparing output

## Running This Module

```bash
cargo run  -p m04-structs-enums -- 8                       # your code, exercise 8
cargo test -p m04-structs-enums-tests --features mine      # test your code
cargo run  -p m04-structs-enums-solution -- all            # the reference solution
cargo test -p m04-structs-enums-tests                      # 25 tests against the solution
```

## Notes on `exercises.md`

- **Exercise 3** asserts `rect3.area() == 400` after scaling a 10×20 rectangle
  by 2. Both sides double, so it's 20×40 = **800**. Fixed in the solution and
  tests.
- **Exercise 9** says "for triangle, assume equilateral". An equilateral
  triangle's height is determined by its base (`h = b·√3/2`), so
  `Triangle { base: 6.0, height: 8.0 }` can't be one. The solution assumes
  **isosceles** — the triangle a base and a height *do* define — giving each
  slanted side `sqrt((b/2)² + h²)`.
- **Exercise 8** leaves the size of the `next_level` bonus open; the solution
  awards `LEVEL_BONUS (100) × level`. Transitions that make no sense (pausing
  the menu, scoring while paused) are ignored.
- **Bonus**: `to_pretty_string` sorts object keys — `HashMap` iteration order
  changes from run to run, which makes unsorted output untestable.
- Answers to the written questions: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[05 · Error Handling](../05-error-handling/)
