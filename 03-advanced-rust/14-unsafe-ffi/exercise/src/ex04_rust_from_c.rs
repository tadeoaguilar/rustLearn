//! Exercise 4: Calling Rust from C.
//!
//! The C library calls `rust_checksum` -- it must exist with exactly that
//! symbol name, so it's declared below already, with a placeholder body.
//!
//! NEVER put `todo!()` / `unwrap()` / anything that can panic in an
//! `extern "C" fn`: a panic can't unwind into C, so Rust aborts the whole
//! process -- including the test runner.

use std::ffi::{CStr, CString, c_char};

// TODO: declare `fn shapes_checksum_twice(data: *const u8, len: usize) -> u32;`
// in an `unsafe extern "C"` block.

/// Called *by C*. A simple multiplicative checksum (FNV-1a).
/// `unsafe` in the attribute: an exported symbol can collide with another
/// with the same name, which is UB at link time -- you vouch it won't.
///
/// # Safety
/// `data` must be null or point to `len` readable bytes. It's an `unsafe fn`
/// because it dereferences a pointer it can't check: a *safe* `pub fn` doing
/// that would let safe Rust code cause UB by passing a bad pointer (clippy's
/// `not_unsafe_ptr_arg_deref` catches this). C callers are unaffected --
/// `unsafe` is a Rust-side contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_checksum(data: *const u8, len: usize) -> u32 {
    0 // TODO Exercise 4: checksum the bytes (see `checksum` below)
}

/// The same algorithm for Rust callers.
pub fn checksum(bytes: &[u8]) -> u32 {
    todo!("Exercise 4")
}

/// Rust -> C -> Rust: C's shapes_checksum_twice calls rust_checksum twice.
pub fn checksum_twice_via_c(bytes: &[u8]) -> u32 {
    todo!("Exercise 4")
}

/// Returns a string allocated by *Rust's* allocator. The caller must give it
/// back to `rust_free_string`: memory must be freed by the allocator that
/// created it. C's `free()` on this pointer is UB -- Rust may not even use
/// malloc (see Exercise 6's #[global_allocator]).
#[unsafe(no_mangle)]
///
/// # Safety
/// `name` must be null or a valid NUL-terminated string.
pub unsafe extern "C" fn rust_greeting(name: *const c_char) -> *mut c_char {
    std::ptr::null_mut() // TODO Exercise 4 (no todo!() in extern "C" fns)
}

/// # Safety
/// `s` must come from `rust_greeting` and not have been freed already.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_free_string(s: *mut c_char) {
    // TODO Exercise 4 (no todo!() in extern "C" fns)
}

pub fn run() {
    todo!("Exercise 4")
}
