//! Exercise 3: Trait Bounds.

use std::fmt::Display;

/// Task 1: inline bounds...
pub fn print_pair<T: Display, U: Display>(a: T, b: U) -> String {
    format!("{a} and {b}")
}

/// ...and the same with a `where` clause, which reads better once bounds get long.
pub fn print_pair_where<T, U>(a: T, b: U) -> String
where
    T: Display,
    U: Display,
{
    format!("{a} and {b}")
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
        Pair { x, y }
    }
}

impl<T: Display + PartialOrd> Pair<T> {
    pub fn cmp_display(&self) -> String {
        if self.x >= self.y {
            format!("Largest: {}", self.x)
        } else {
            format!("Largest: {}", self.y)
        }
    }
}

/// Task 3. The exercise's version has a `Clone` bound and calls `b.clone()`
/// -- unnecessary: we own `b`, so we can just return it. Only bound what you
/// use: every extra bound is a type someone can no longer pass in.
pub fn compare_and_print<T>(a: T, b: T) -> T
where
    T: PartialOrd + Display,
{
    if a > b {
        println!("{a} is greater");
        a
    } else {
        println!("{b} is greater");
        b
    }
}

/// `impl Trait` in *return* position: "some type implementing Display; the
/// caller doesn't need to know which".
pub fn describe(n: i32) -> impl Display {
    if n % 2 == 0 {
        format!("{n} is even")
    } else {
        format!("{n} is odd")
    }
}

pub fn run() {
    println!("{}", print_pair(1, "two"));
    println!("{}", print_pair_where(3.5, 'c'));
    println!("{}", Pair::new(3, 7).cmp_display());
    println!("{}", Pair::new("b", "a").cmp_display());
    let _no_cmp_display = Pair::new(vec![1], vec![2]); // fine -- just no cmp_display()
    println!(
        "returned {}",
        compare_and_print(String::from("pear"), String::from("apple"))
    );
    println!("{}", describe(7));
}
