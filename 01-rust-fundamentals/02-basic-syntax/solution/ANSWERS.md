# Answers · 02 Basic Syntax

## Exercise 1: Variables and Mutability

**What's the difference between shadowing and mutability?**

`mut` lets you assign a new value to the *same* variable; the type can never
change. Shadowing (`let x = ...` again) creates a *new* variable that hides the
old one; it can have a different type, and it can be immutable even if the old
one was mutable. Shadowing inside a `{ }` block ends with the block — the outer
`x` is visible again afterwards.

**When would you use `const` vs `let`?**

`const` for values known at compile time that never change and may be needed
anywhere, including at module level: `const MAX_POINTS: u32 = 100_000;`. It must
have a type annotation and is inlined wherever it's used. `let` for everything
computed at runtime. (`static` is the third option: one fixed memory location
for the whole program run.)

**Can you shadow a mutable variable with an immutable one?**

Yes. `let mut v = Vec::new(); v.push(1); let v = v;` — from then on `v` cannot
be mutated. It is a common way to "freeze" a value after building it.

## Exercise 4, Task 2

**Explain why both arms must return the same type.**

`let number = if condition { 5 } else { 6 };` gives `number` a single type
decided at compile time. If one arm were `5` and the other `"six"`, the compiler
couldn't know the type of `number` without running the program — so it rejects
it with `error[E0308]: 'if' and 'else' have incompatible types`.
