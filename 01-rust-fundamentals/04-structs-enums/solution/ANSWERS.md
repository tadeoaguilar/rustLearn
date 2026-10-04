# Answers · 04 Structs & Enums

## Exercise 1, Task 3: Can you still use `user1`? Why or why not?

Not as a whole. `..user1` copies or **moves** every field not listed
explicitly. `username` is a `String`, so it is moved into `user2`, and `user1`
becomes *partially moved*:

```text
println!("{:?}", user1);
                 ^^^^^ error[E0382]: borrow of partially moved value: `user1`
```

The fields that weren't moved are still usable individually: `user1.email` (we
gave `user2` its own), and `user1.active` / `user1.sign_in_count` (they're Copy,
so they were copied, not moved). If *every* remaining field had been Copy,
`user1` would still be fully usable.

## Exercise 2, Task 1: Color and Point

`Color(0, 0, 0)` and `Point(0, 0, 0)` have identical layouts but are different
types. A function taking a `Point` refuses a `Color` with `error[E0308]:
mismatched types`. Wrapping a primitive in a one-field tuple struct to get this
protection (`struct Meters(f64);`) is called the **newtype pattern**; it costs
nothing at runtime.

## Exercise 2, Task 2: What is a unit struct for?

It has no fields and takes zero bytes. It's used as a marker type, a type
parameter, or something to hang a trait implementation on when there's no data
— e.g. `struct ConsoleLogger; impl Logger for ConsoleLogger { ... }`.

## Exercise 6, Task 3: Why is `_ =>` required after the guards?

The compiler checks exhaustiveness from the *patterns* only and ignores guard
conditions. `Celsius(t) if t < 0` and `Celsius(t) if t > 30` don't cover
`Celsius(15)` as far as it can tell, so without a catch-all you get
`error[E0004]: non-exhaustive patterns`.
