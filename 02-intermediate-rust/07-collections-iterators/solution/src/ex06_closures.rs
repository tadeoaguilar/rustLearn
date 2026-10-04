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
    f(x)
}

/// Calls `f` n times. FnMut because the closure may update state.
pub fn repeat<F: FnMut()>(mut f: F, n: usize) {
    for _ in 0..n {
        f();
    }
}

/// Calls `f` once. FnOnce accepts *any* closure.
pub fn call_once<F: FnOnce() -> String>(f: F) -> String {
    f()
}

/// Task 2: a counter. `move` puts `count` *inside* the closure, so the closure
/// can outlive this function. Each call to make_counter gets its own count.
pub fn make_counter() -> impl FnMut() -> u32 {
    let mut count = 0;
    move || {
        count += 1;
        count
    }
}

/// Task 3: function composition. compose(f, g)(x) == g(f(x)) -- "f, then g".
pub fn compose<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C) -> impl Fn(A) -> C {
    move |x| g(f(x))
}

/// A pipeline of any number of steps, built at runtime. Each step has a
/// different closure type, so they must be boxed into `dyn Fn`.
pub struct Pipeline<T> {
    steps: Vec<Box<dyn Fn(T) -> T>>,
}

impl<T> Pipeline<T> {
    pub fn new() -> Self {
        Pipeline { steps: Vec::new() }
    }

    /// Builder style: takes and returns `self`, so calls chain.
    pub fn then(mut self, step: impl Fn(T) -> T + 'static) -> Self {
        self.steps.push(Box::new(step));
        self
    }

    pub fn run(&self, input: T) -> T {
        self.steps.iter().fold(input, |acc, step| step(acc))
    }
}

impl<T> Default for Pipeline<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn run() {
    let x = 5;
    let add_x = |y| x + y; // Fn: reads x
    println!("add_x(3) = {}, add_x(4) = {}", add_x(3), add_x(4));
    println!("apply(add_x, 10) = {}", apply(add_x, 10));

    let mut count = 0;
    repeat(|| count += 1, 3); // FnMut: mutates count
    println!("count after repeat = {count}");

    let message = String::from("Hello");
    let consume = move || message + ", world"; // FnOnce: moves message out
    println!("{}", call_once(consume));
    // consume(); // error[E0382]: use of moved value: `consume`

    let mut c1 = make_counter();
    let mut c2 = make_counter();
    println!("c1: {} {} {}, c2: {}", c1(), c1(), c1(), c2());

    let add_one_then_double = compose(|x: i32| x + 1, |x| x * 2);
    println!("compose(+1, *2)(5) = {}", add_one_then_double(5));
    let shout = compose(
        |s: &str| s.trim().to_string(),
        |s: String| s.to_uppercase() + "!",
    );
    println!("{}", shout("  hello  "));

    let p = Pipeline::new()
        .then(|x: i32| x + 1)
        .then(|x| x * 10)
        .then(|x| x - 3);
    println!("pipeline(4) = {}", p.run(4));
}
