//! Exercise 3: Bindings to our own C library (csrc/shapes.c).

use std::ffi::{CString, NulError, c_char, c_void};
use std::ptr::NonNull;

/// `#[repr(C)]`: lay the fields out exactly as C would. Without it, Rust may
/// reorder fields and the two sides would disagree about the bytes.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// The raw declarations, kept private: nobody outside this module should
/// have to think about pointers.
mod ffi {
    use super::Point;
    use std::ffi::{c_char, c_void};

    /// Opaque: Rust never knows the size or fields, so it can only ever be
    /// used behind a pointer. (The Rustonomicon's recommended pattern.)
    #[repr(C)]
    pub struct Counter {
        _private: [u8; 0],
        _not_send_sync: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
    }

    // TODO Exercise 3, Task 1: declare the C functions from csrc/shapes.h here:
    //
    // unsafe extern "C" {
    //     pub safe fn shapes_distance(a: Point, b: Point) -> f64;
    //     pub fn shapes_polygon_area(points: *const Point, len: usize) -> f64;
    //     pub fn counter_new(name: *const c_char) -> *mut Counter;
    //     ... counter_add, counter_get, counter_name, counter_free, shapes_for_each_point
    // }
}

/// Passing a `#[repr(C)]` struct by value is fine across the C ABI.
pub fn distance(a: Point, b: Point) -> f64 {
    todo!("Exercise 3")
}

pub fn polygon_area(points: &[Point]) -> f64 {
    todo!("Exercise 3")
}

/// Owns one C `Counter`. Created by `counter_new`, freed exactly once in Drop.
///
/// Send/Sync: a raw pointer makes this type neither, automatically. That's
/// the safe default. The C code has no thread-local state, so moving a
/// Counter to another thread would be fine and `unsafe impl Send` would be
/// sound -- but concurrent `add` calls would race (`value += n` isn't atomic),
/// so `Sync` would NOT be.
#[derive(Debug)]
pub struct Counter {
    ptr: NonNull<ffi::Counter>,
}

impl Counter {
    pub fn new(name: &str) -> Result<Counter, NulError> {
        todo!("Exercise 3")
    }

    /// `&mut self`: C mutates the counter, and Rust's rules say mutation
    /// needs exclusive access -- the wrapper's signature enforces it.
    pub fn add(&mut self, n: i64) {
        todo!("Exercise 3")
    }

    pub fn get(&self) -> i64 {
        todo!("Exercise 3")
    }

    /// Asks C for the length first (buf_len = 0), then allocates exactly enough.
    pub fn name(&self) -> String {
        todo!("Exercise 3")
    }
}

impl Drop for Counter {
    fn drop(&mut self) {
        // TODO Exercise 3: a todo!() here could abort the test run, so this is empty.
    }
}

/// Closures across FFI. C can only call a plain `extern "C" fn`, so we pass
/// a generic *trampoline* function as the callback and the closure itself as
/// `user_data`. Each closure type F gets its own monomorphised trampoline,
/// which knows how to turn `user_data` back into `&mut F`.
pub fn for_each_point<F: FnMut(Point)>(points: &[Point], mut f: F) {
    todo!("Exercise 3")
}

pub fn run() {
    todo!("Exercise 3")
}
