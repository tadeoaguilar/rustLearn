//! Exercise 5: Hygiene, `$crate` and Debugging Macros.
//!
//! Hygiene: identifiers a macro *invents* live in the macro's own syntax
//! context. So
//!
//! ```text
//! macro_rules! make_x { () => { let x = 42; }; }
//! make_x!();
//! println!("{x}");   // error[E0425]: cannot find value `x` in this scope
//! ```
//!
//! fails: the `x` inside the macro is a different `x` from the caller's. The
//! same rule stops a macro's temporaries from clobbering your variables. To
//! create a name the caller can use, the *caller* must supply it.

use std::cell::RefCell;
use std::fmt::Debug;

/// `make_var!(answer, 42)` -- `answer` comes from the caller, so it's visible there.
#[macro_export]
macro_rules! make_var {
    ($name:ident, $value:expr) => {
        let $name = $value;
    };
}

/// Hygiene the other way round: this macro uses a temporary called `value`.
/// A caller's own `value` is untouched.
#[macro_export]
macro_rules! double {
    ($e:expr) => {{
        let value = $e;
        value * 2
    }};
}

/// assert_eq! with the *source text* of both sides and the location.
///
/// `stringify!` turns tokens into a string literal; `file!`/`line!` expand to
/// the *caller's* location, because they're expanded in the caller's code.
#[macro_export]
macro_rules! my_assert_eq {
    ($left:expr, $right:expr $(,)?) => {
        match (&$left, &$right) {
            (left, right) => {
                if !(*left == *right) {
                    ::std::panic!(
                        "assertion failed: `{} == {}` (left: {:?}, right: {:?}) at {}:{}",
                        stringify!($left),
                        stringify!($right),
                        left,
                        right,
                        file!(),
                        line!()
                    );
                }
            }
        }
    };
}

thread_local! {
    static TRACE_LOG: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Called by `traced!`. `#[doc(hidden)]` + `pub`: macros expand in the
/// caller's crate, so anything they call must be public -- but it isn't
/// meant to be used directly.
#[doc(hidden)]
pub fn __record<T: Debug>(location_and_expr: &str, value: &T) {
    TRACE_LOG.with(|log| {
        log.borrow_mut()
            .push(format!("{location_and_expr} = {value:?}"))
    });
}

/// Takes and clears this thread's trace log.
pub fn take_trace_log() -> Vec<String> {
    TRACE_LOG.with(|log| std::mem::take(&mut *log.borrow_mut()))
}

/// Evaluates `expr`, records "file:line: expr = value", returns the value.
/// `$crate::` makes the call to `__record` work from any crate.
#[macro_export]
macro_rules! traced {
    ($e:expr) => {{
        let value = $e;
        $crate::ex05_hygiene::__record(
            concat!(file!(), ":", line!(), ": ", stringify!($e)),
            &value,
        );
        value
    }};
}

pub fn hygiene_demo() -> (i32, i32) {
    crate::make_var!(answer, 42);
    let value = 10;
    let doubled = crate::double!(value + 1); // the macro's own `value` doesn't clash
    (answer, doubled + value)
}

pub fn run() {
    println!("hygiene_demo() = {:?}", hygiene_demo());
    crate::my_assert_eq!(1 + 1, 2);
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {})); // keep the demo's output tidy
    let result = std::panic::catch_unwind(|| crate::my_assert_eq!(1 + 2, 4));
    std::panic::set_hook(default_hook);
    if let Err(e) = result {
        println!(
            "my_assert_eq! message: {}",
            e.downcast_ref::<String>().cloned().unwrap_or_default()
        );
    }
    let sum = crate::traced!(2 + 3);
    let words = crate::traced!("a b c".split(' ').count());
    println!("traced values: {sum}, {words}");
    for line in take_trace_log() {
        println!("  {line}");
    }
    println!(
        "See expansions: cargo install cargo-expand && cargo expand -p m13-macros-solution ex05_hygiene"
    );
}
