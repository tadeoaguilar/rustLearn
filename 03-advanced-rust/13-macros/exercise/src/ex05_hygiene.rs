//! Exercise 5: Hygiene, `$crate` and Debugging Macros.  -- see exercises.md
//!
//! Write, each with #[macro_export]:
//!
//!     make_var!(name, value)   `let name = value;` visible to the caller
//!     double!(e)               uses an internal `let value = ...` -- show it
//!                              doesn't clash with a caller's `value`
//!     my_assert_eq!(a, b)      panics with "assertion failed: `a == b` (left: .., right: ..) at file:line"
//!     traced!(e)               records "file:line: e = value" with __record, returns the value
//!
//! __record and take_trace_log are provided.

use std::cell::RefCell;
use std::fmt::Debug;

thread_local! {
    static TRACE_LOG: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Called by `traced!` as `$crate::ex05_hygiene::__record(..)`. It's `pub`
/// because macros expand in the *caller's* crate.
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

// TODO Exercise 5: your macros here.

/// Use make_var! and double! to return (42, 32):
/// make_var!(answer, 42); let value = 10; double!(value + 1) + value == 32.
pub fn hygiene_demo() -> (i32, i32) {
    todo!("Exercise 5")
}

pub fn run() {
    todo!("Exercise 5")
}
