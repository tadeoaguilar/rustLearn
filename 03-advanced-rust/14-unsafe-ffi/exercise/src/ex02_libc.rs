//! Exercise 2: Calling the C standard library.
//!
//! These functions are in the C library every Rust program already links
//! against; no build script needed.

use std::ffi::{CString, NulError, c_char, c_int, c_void};

unsafe extern "C" {
    /// `safe fn`: abs can't cause UB for any input, so callers don't need
    /// `unsafe`. (abs(INT_MIN) is famously wrong, but it's not UB.)
    safe fn abs(x: c_int) -> c_int;

    /// Reads bytes until it finds a NUL. With a pointer that isn't a valid
    /// NUL-terminated string it reads out of bounds: unsafe.
    fn strlen(s: *const c_char) -> usize;

    fn qsort(
        base: *mut c_void,
        n: usize,
        size: usize,
        compar: extern "C" fn(*const c_void, *const c_void) -> c_int,
    );
}

pub fn c_abs(x: i32) -> i32 {
    todo!("Exercise 2")
}

/// `CString::new` appends the NUL and rejects interior NULs: C would see
/// "a\0b" as just "a", silently truncating the string.
pub fn c_strlen(s: &str) -> Result<usize, NulError> {
    todo!("Exercise 2")
}

/// The comparator C calls back. `extern "C"` gives it the C calling
/// convention. If it panicked, unwinding into C would be UB -- Rust aborts
/// the process instead. (`extern "C-unwind"` allows unwinding through C code
/// that is built to support it.) This one can't panic.
extern "C" fn compare_i32(a: *const c_void, b: *const c_void) -> c_int {
    todo!("Exercise 2")
}

pub fn c_sort(values: &mut [i32]) {
    todo!("Exercise 2")
}

pub fn run() {
    todo!("Exercise 2")
}
