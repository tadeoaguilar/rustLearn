//! Exercise 4: #[memoize] used for real.

use std::cell::Cell;

use crate::sut::memoize;

#[memoize]
fn fib(n: u64) -> u64 {
    if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

thread_local! {
    static CALLS: Cell<u32> = const { Cell::new(0) };
}

#[memoize]
pub fn slow_square(x: i32) -> i64 {
    CALLS.with(|c| c.set(c.get() + 1));
    x as i64 * x as i64
}

#[memoize]
fn greet(name: String, excited: bool) -> String {
    if excited {
        return format!("Hello, {name}!");
    }
    format!("Hello, {name}.")
}

#[test]
fn ex4_recursion_through_the_cache() {
    assert_eq!(
        fib(90),
        2_880_067_194_370_816_120,
        "instant: every subproblem is cached"
    );
    assert_eq!(fib_cache_len(), 91);
}

#[test]
fn ex4_the_body_runs_once_per_input() {
    assert_eq!(slow_square(12), 144);
    assert_eq!(slow_square(12), 144);
    assert_eq!(slow_square(-3), 9);
    assert_eq!(CALLS.with(Cell::get), 2);
    assert_eq!(slow_square_cache_len(), 2);
}

#[test]
fn ex4_several_arguments_and_early_return() {
    assert_eq!(greet("Ana".into(), true), "Hello, Ana!");
    assert_eq!(greet("Ana".into(), false), "Hello, Ana.");
    assert_eq!(greet("Ana".into(), true), "Hello, Ana!");
    assert_eq!(greet_cache_len(), 2);
}
