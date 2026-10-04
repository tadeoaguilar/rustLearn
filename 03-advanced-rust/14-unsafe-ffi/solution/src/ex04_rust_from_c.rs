//! Exercise 4: Calling Rust from C.
//!
//! (`export_name = "sol_..."` -- see lib.rs. In your crate:
//! `#[unsafe(no_mangle)]`, which exports the function under its own name.)

use std::ffi::{CStr, CString, c_char};

unsafe extern "C" {
    #[link_name = "sol_shapes_checksum_twice"]
    fn shapes_checksum_twice(data: *const u8, len: usize) -> u32;
}

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
#[unsafe(export_name = "sol_rust_checksum")]
pub unsafe extern "C" fn rust_checksum(data: *const u8, len: usize) -> u32 {
    if data.is_null() {
        return 0;
    }
    // SAFETY: the caller promises data points to len readable bytes.
    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    checksum(bytes)
}

/// The same algorithm for Rust callers.
pub fn checksum(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0x811c_9dc5_u32, |h, &b| {
        (h ^ u32::from(b)).wrapping_mul(0x0100_0193)
    })
}

/// Rust -> C -> Rust: C's shapes_checksum_twice calls rust_checksum twice.
pub fn checksum_twice_via_c(bytes: &[u8]) -> u32 {
    // SAFETY: valid pointer/length pair for the duration of the call.
    unsafe { shapes_checksum_twice(bytes.as_ptr(), bytes.len()) }
}

/// Returns a string allocated by *Rust's* allocator. The caller must give it
/// back to `rust_free_string`: memory must be freed by the allocator that
/// created it. C's `free()` on this pointer is UB -- Rust may not even use
/// malloc (see Exercise 6's #[global_allocator]).
#[unsafe(export_name = "sol_rust_greeting")]
///
/// # Safety
/// `name` must be null or a valid NUL-terminated string.
pub unsafe extern "C" fn rust_greeting(name: *const c_char) -> *mut c_char {
    let name = if name.is_null() {
        "stranger".into()
    } else {
        // SAFETY: the caller passes a valid NUL-terminated string.
        unsafe { CStr::from_ptr(name) }.to_string_lossy()
    };
    let greeting = CString::new(format!("Hello, {name}!")).expect("no interior NUL");
    greeting.into_raw() // ownership moves to the caller
}

/// # Safety
/// `s` must come from `rust_greeting` and not have been freed already.
#[unsafe(export_name = "sol_rust_free_string")]
pub unsafe extern "C" fn rust_free_string(s: *mut c_char) {
    if !s.is_null() {
        // SAFETY: the caller guarantees s came from CString::into_raw.
        drop(unsafe { CString::from_raw(s) });
    }
}

pub fn run() {
    let data = b"hello ffi";
    println!("checksum in Rust:           {:#010x}", checksum(data));
    println!(
        "checksum twice, through C:  {:#010x}",
        checksum_twice_via_c(data)
    );
    println!(
        "expected (2x, wrapping):    {:#010x}",
        checksum(data).wrapping_mul(2)
    );

    let name = CString::new("Ferris").unwrap();
    // SAFETY: name is a valid C string.
    let raw = unsafe { rust_greeting(name.as_ptr()) };
    // SAFETY: raw came from rust_greeting and is still live.
    let text = unsafe { CStr::from_ptr(raw) }
        .to_string_lossy()
        .into_owned();
    // SAFETY: raw came from rust_greeting; freed exactly once.
    unsafe { rust_free_string(raw) };
    println!("rust_greeting -> {text:?}");
}
