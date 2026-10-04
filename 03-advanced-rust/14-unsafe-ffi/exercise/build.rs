//! Compiles csrc/shapes.c (provided -- you don't need to change it) into a
//! static library linked into this crate. Needs a C compiler.

fn main() {
    println!("cargo:rerun-if-changed=csrc/shapes.c");
    println!("cargo:rerun-if-changed=csrc/shapes.h");
    cc::Build::new()
        .file("csrc/shapes.c")
        // No SHAPES_PREFIX: the C symbols have exactly the names in shapes.h.
        .warnings(true)
        .compile("shapes_exercise");
}
