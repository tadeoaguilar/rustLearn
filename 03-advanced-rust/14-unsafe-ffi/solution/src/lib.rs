//! Module 14 -- Unsafe Rust & FFI. Reference solution.
//!
//! The pattern in every exercise: a small `unsafe` core, a safe public API
//! around it that makes misuse impossible, and a `// SAFETY:` comment on
//! every unsafe block saying which rule makes it sound.
//!
//! About symbol names: the C library is compiled with a "sol_" prefix and the
//! Rust functions exported to C use `#[unsafe(export_name = "sol_...")]`.
//! That's only so this crate can be linked into one test binary together with
//! the exercise crate, which uses the plain names. In your own code you'd
//! write `#[unsafe(no_mangle)]` and need no `link_name`.

// Rust 2024 makes this lint warn by default; deny it to be sure every unsafe
// operation inside an `unsafe fn` sits in its own explicit `unsafe {}` block.
#![deny(unsafe_op_in_unsafe_fn)]

pub mod ex01_raw_pointers;
pub mod ex02_libc;
pub mod ex03_c_library;
pub mod ex04_rust_from_c;
pub mod ex05_stack_vec;
pub mod ex06_allocator;
pub mod ex07_system;
