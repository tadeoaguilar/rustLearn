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

    // `link_name` because this crate's copy of the C library is prefixed
    // with "sol_" (see lib.rs). With an unprefixed library you'd omit it.
    unsafe extern "C" {
        #[link_name = "sol_shapes_distance"]
        pub safe fn shapes_distance(a: Point, b: Point) -> f64;
        #[link_name = "sol_shapes_polygon_area"]
        pub fn shapes_polygon_area(points: *const Point, len: usize) -> f64;
        #[link_name = "sol_counter_new"]
        pub fn counter_new(name: *const c_char) -> *mut Counter;
        #[link_name = "sol_counter_add"]
        pub fn counter_add(c: *mut Counter, n: i64);
        #[link_name = "sol_counter_get"]
        pub fn counter_get(c: *const Counter) -> i64;
        #[link_name = "sol_counter_name"]
        pub fn counter_name(c: *const Counter, buf: *mut c_char, buf_len: usize) -> usize;
        #[link_name = "sol_counter_free"]
        pub fn counter_free(c: *mut Counter);
        #[link_name = "sol_shapes_for_each_point"]
        pub fn shapes_for_each_point(
            points: *const Point,
            len: usize,
            cb: extern "C" fn(Point, *mut c_void),
            user_data: *mut c_void,
        );
    }
}

/// Passing a `#[repr(C)]` struct by value is fine across the C ABI.
pub fn distance(a: Point, b: Point) -> f64 {
    ffi::shapes_distance(a, b)
}

pub fn polygon_area(points: &[Point]) -> f64 {
    // SAFETY: pointer and length describe a valid slice of repr(C) Points
    // that outlives the call; C only reads it.
    unsafe { ffi::shapes_polygon_area(points.as_ptr(), points.len()) }
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
        let c_name = CString::new(name)?;
        // SAFETY: c_name is a valid C string; counter_new copies it.
        let raw = unsafe { ffi::counter_new(c_name.as_ptr()) };
        let ptr = NonNull::new(raw).expect("counter_new returned NULL: out of memory");
        Ok(Counter { ptr })
    }

    /// `&mut self`: C mutates the counter, and Rust's rules say mutation
    /// needs exclusive access -- the wrapper's signature enforces it.
    pub fn add(&mut self, n: i64) {
        // SAFETY: ptr is valid until Drop, and &mut self means no one else is using it.
        unsafe { ffi::counter_add(self.ptr.as_ptr(), n) }
    }

    pub fn get(&self) -> i64 {
        // SAFETY: ptr is valid until Drop; counter_get only reads.
        unsafe { ffi::counter_get(self.ptr.as_ptr()) }
    }

    /// Asks C for the length first (buf_len = 0), then allocates exactly enough.
    pub fn name(&self) -> String {
        // SAFETY: a NULL buffer with length 0 is allowed by counter_name.
        let len = unsafe { ffi::counter_name(self.ptr.as_ptr(), std::ptr::null_mut(), 0) };
        let mut buf = vec![0u8; len + 1];
        // SAFETY: buf is writable for buf.len() bytes; C writes at most that many.
        unsafe {
            ffi::counter_name(
                self.ptr.as_ptr(),
                buf.as_mut_ptr().cast::<c_char>(),
                buf.len(),
            )
        };
        buf.truncate(len); // drop the NUL
        String::from_utf8_lossy(&buf).into_owned()
    }
}

impl Drop for Counter {
    fn drop(&mut self) {
        // SAFETY: ptr came from counter_new and this is the only place it's
        // freed; after drop the Counter can't be used again.
        unsafe { ffi::counter_free(self.ptr.as_ptr()) }
    }
}

/// Closures across FFI. C can only call a plain `extern "C" fn`, so we pass
/// a generic *trampoline* function as the callback and the closure itself as
/// `user_data`. Each closure type F gets its own monomorphised trampoline,
/// which knows how to turn `user_data` back into `&mut F`.
pub fn for_each_point<F: FnMut(Point)>(points: &[Point], mut f: F) {
    extern "C" fn trampoline<F: FnMut(Point)>(p: Point, user_data: *mut c_void) {
        // SAFETY: user_data is the `&mut f` below, alive for the whole C
        // call, and only used from this thread.
        let f = unsafe { &mut *user_data.cast::<F>() };
        f(p);
    }
    let user_data = (&raw mut f).cast::<c_void>();
    // SAFETY: points is a valid slice; trampoline::<F> matches the callback
    // signature and user_data points to an F.
    unsafe {
        ffi::shapes_for_each_point(points.as_ptr(), points.len(), trampoline::<F>, user_data)
    };
}

pub fn run() {
    let (a, b) = (Point { x: 0.0, y: 0.0 }, Point { x: 3.0, y: 4.0 });
    println!("distance = {}", distance(a, b));
    let square = [
        Point { x: 0.0, y: 0.0 },
        Point { x: 2.0, y: 0.0 },
        Point { x: 2.0, y: 2.0 },
        Point { x: 0.0, y: 2.0 },
    ];
    println!("polygon area = {}", polygon_area(&square));

    let mut c = Counter::new("visits").expect("no NUL in name");
    c.add(3);
    c.add(4);
    println!("counter {:?} = {}", c.name(), c.get());

    let mut xs = Vec::new();
    for_each_point(&square, |p| xs.push(p.x)); // a closure capturing `xs` mutably
    println!("for_each_point collected x values: {xs:?}");
}
