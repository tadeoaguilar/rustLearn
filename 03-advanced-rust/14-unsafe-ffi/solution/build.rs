//! Compiles csrc/shapes.c into a static library and links it into this crate.
//!
//! The `cc` crate finds the platform's C compiler (clang, gcc, MSVC), passes
//! the right flags, and prints the `cargo:rustc-link-lib=static=shapes`
//! instruction that makes rustc link the result.

fn main() {
    println!("cargo:rerun-if-changed=csrc/shapes.c");
    println!("cargo:rerun-if-changed=csrc/shapes.h");
    cc::Build::new()
        .file("csrc/shapes.c")
        // Prefix every C symbol with "sol_" -- see csrc/shapes.h for why.
        .define("SHAPES_PREFIX", "sol_")
        .warnings(true)
        .compile("shapes_solution");
}
