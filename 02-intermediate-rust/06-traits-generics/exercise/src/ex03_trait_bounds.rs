//! Exercise 3: Trait Bounds.

use std::fmt::Display;

/// Task 1: inline bounds...
pub fn print_pair<T: Display, U: Display>(a: T, b: U) -> String {
    todo!("Exercise 3")
}

/// ...and the same with a `where` clause, which reads better once bounds get long.
pub fn print_pair_where<T, U>(a: T, b: U) -> String
where
    T: Display,
    U: Display,
{
    todo!("Exercise 3")
}

/// Task 2: a second `impl` block whose methods only exist when T meets the
/// bounds. `Pair<Vec<i32>>` can be created, but has no `cmp_display`.
#[derive(Debug, Clone, PartialEq)]
pub struct Pair<T> {
    pub x: T,
    pub y: T,
}

impl<T> Pair<T> {
    pub fn new(x: T, y: T) -> Self {
        todo!("Exercise 3")
    }
}

impl<T: Display + PartialOrd> Pair<T> {
    pub fn cmp_display(&self) -> String {
        todo!("Exercise 3")
    }
}

/// Task 3. The exercise's version has a `Clone` bound and calls `b.clone()`
/// -- unnecessary: we own `b`, so we can just return it. Only bound what you
/// use: every extra bound is a type someone can no longer pass in.
pub fn compare_and_print<T>(a: T, b: T) -> T
where
    T: PartialOrd + Display,
{
    todo!("Exercise 3")
}

/// `impl Trait` in *return* position: "some type implementing Display; the
/// caller doesn't need to know which".
pub fn describe(n: i32) -> impl Display {
    todo!("Exercise 3")
}

pub fn run() {
    todo!("Exercise 3")
}
