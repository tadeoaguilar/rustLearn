//! Exercise 1: Your First `macro_rules!`.

/// Squares an expression, evaluating it exactly once.
///
/// ```
/// use m13_macros_solution::square;
/// assert_eq!(square!(4), 16);
/// assert_eq!(square!(1 + 2), 9); // 9, not 1 + 2 * 1 + 2 = 5
/// ```
#[macro_export]
macro_rules! square {
    ($x:expr) => {{
        // Bind first. `$x * $x` would paste the expression twice -- running
        // any side effects twice. (An `expr` fragment is always kept whole, so
        // `1 + 2` can't be split by precedence; evaluation count is the issue.)
        let value = $x;
        value * value
    }};
}

/// The naive version, kept to show the bug: the argument is evaluated twice.
#[macro_export]
macro_rules! square_naive {
    ($x:expr) => {
        $x * $x
    };
}

/// The largest of one or more expressions.
///
/// ```
/// use m13_macros_solution::max;
/// assert_eq!(max!(3), 3);
/// assert_eq!(max!(3, 9, 2, 7), 9);
/// ```
#[macro_export]
macro_rules! max {
    ($x:expr $(,)?) => { $x };
    ($x:expr, $($rest:expr),+ $(,)?) => {{
        let first = $x;
        // `$crate::max!` -- not `max!` -- so the recursive call resolves even
        // when a user has imported this macro under another name.
        let rest = $crate::max!($($rest),+);
        if first > rest { first } else { rest }
    }};
}

/// A HashMap literal. Trailing comma allowed; capacity reserved up front.
///
/// ```
/// use m13_macros_solution::hashmap;
/// let m = hashmap! { "a" => 1, "b" => 2, };
/// assert_eq!(m["b"], 2);
/// ```
#[macro_export]
macro_rules! hashmap {
    () => { ::std::collections::HashMap::new() };
    ($($key:expr => $value:expr),+ $(,)?) => {{
        let mut map = ::std::collections::HashMap::with_capacity($crate::count!($($key)+));
        $( map.insert($key, $value); )+
        map
    }};
}

/// `vec_of_strings!["a", "b"]` -> `vec!["a".to_string(), "b".to_string()]`.
#[macro_export]
macro_rules! vec_of_strings {
    ($($s:expr),* $(,)?) => {
        vec![$(::std::string::ToString::to_string(&$s)),*]
    };
}

thread_local! {
    static CALLS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Returns 3 and counts how often it was called -- for the evaluation demo.
pub fn next_value() -> i32 {
    CALLS.with(|c| c.set(c.get() + 1));
    3
}

pub fn calls() -> u32 {
    CALLS.with(|c| c.get())
}

pub fn reset_calls() {
    CALLS.with(|c| c.set(0));
}

pub fn run() {
    println!("square!(4) = {}", square!(4));
    reset_calls();
    let v = square!(next_value());
    println!(
        "square!(next_value())       = {v}, next_value() ran {} time(s)",
        calls()
    );
    reset_calls();
    let v = square_naive!(next_value());
    println!(
        "square_naive!(next_value()) = {v}, next_value() ran {} time(s)",
        calls()
    );
    println!("max!(3, 9, 2, 7) = {}", max!(3, 9, 2, 7));
    let mut pairs: Vec<_> = hashmap! { "a" => 1, "b" => 2 }.into_iter().collect();
    pairs.sort();
    println!("hashmap! = {pairs:?}");
    println!("vec_of_strings! = {:?}", vec_of_strings!["a", 'b', 3]);
}
