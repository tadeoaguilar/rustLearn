//! Exercise 6: Closures -- Fn, FnMut, FnOnce.
//!
//! The trait a closure implements depends on what its body does with the
//! variables it captures:
//!
//! | Body...                            | Implements          | Call it   |
//! |------------------------------------|---------------------|-----------|
//! | only reads captures                | Fn + FnMut + FnOnce | any times |
//! | mutates a capture                  | FnMut + FnOnce      | any times, needs `mut` |
//! | moves a capture out (drops/returns)| FnOnce              | once      |
//!
//! Accept the *most permissive* trait you can: a function taking `FnOnce`
//! accepts every closure; one taking `Fn` rejects closures that mutate.

/// Task 1: apply a closure. `impl Fn(i32) -> i32` is sugar for a generic F.
pub fn apply<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    todo!("Exercise 6")
}

/// Calls `f` n times. FnMut because the closure may update state.
pub fn repeat<F: FnMut()>(mut f: F, n: usize) {
    todo!("Exercise 6")
}

/// Calls `f` once. FnOnce accepts *any* closure.
pub fn call_once<F: FnOnce() -> String>(f: F) -> String {
    todo!("Exercise 6")
}

/// Task 2: a counter. `move` puts `count` *inside* the closure, so the closure
/// can outlive this function. Each call to make_counter gets its own count.
pub fn make_counter() -> impl FnMut() -> u32 {
    // A bare `todo!()` doesn't compile here: `impl FnMut` needs *some*
    // closure behind it. Replace both lines.
    todo!("Exercise 6");
    || 0
}

/// Task 3: function composition. compose(f, g)(x) == g(f(x)) -- "f, then g".
pub fn compose<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C) -> impl Fn(A) -> C {
    // Placeholder closure so this compiles; replace both lines.
    todo!("Exercise 6");
    |_: A| -> C { unreachable!() }
}

/// A pipeline of any number of steps, built at runtime. Each step has a
/// different closure type, so they must be boxed into `dyn Fn`.
pub struct Pipeline<T> {
    steps: Vec<Box<dyn Fn(T) -> T>>,
}

impl<T> Pipeline<T> {
    pub fn new() -> Self {
        todo!("Exercise 6")
    }

    /// Builder style: takes and returns `self`, so calls chain.
    pub fn then(mut self, step: impl Fn(T) -> T + 'static) -> Self {
        todo!("Exercise 6")
    }

    pub fn run(&self, input: T) -> T {
        todo!("Exercise 6")
    }
}

impl<T> Default for Pipeline<T> {
    fn default() -> Self {
        todo!("Exercise 6")
    }
}

pub fn run() {
    todo!("Exercise 6")
}
