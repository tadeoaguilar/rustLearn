//! Exercise 1: Variables and Mutability.

/// A constant: always annotated, always known at compile time, usable anywhere
/// (even at module level), and inlined at every use.
pub const MAX_POINTS: u32 = 100_000;

pub fn run() {
    // An immutable binding.
    let x = 5;
    println!("x = {x}");

    // x = 6;
    // ^ error[E0384]: cannot assign twice to immutable variable `x`
    //   help: consider making this binding mutable: `mut x`

    // A mutable binding can be reassigned -- but only with the same type.
    let mut y = 10;
    println!("y = {y}");
    y = 20;
    println!("y = {y} (after mutation)");

    // Shadowing: a brand new variable that happens to reuse the name.
    // Unlike `mut`, the new one may have a different type.
    let x = "five";
    println!("x = {x} (shadowed with a &str)");

    // Explicit type annotation.
    let z: f64 = 2.5;
    println!("z = {z}");

    println!("shadowing() = {}", shadowing());
    println!("MAX_POINTS = {MAX_POINTS}");
}

/// The classic shadowing example from the Rust Book: the text `spaces` is
/// replaced by the number `spaces`, without inventing `spaces_str`/`spaces_num`.
///
/// With `let mut spaces = "   "; spaces = spaces.len();` you would get
/// error[E0308]: mismatched types -- `mut` cannot change a variable's type.
// Clippy would rather we wrote `"   ".len()`; the binding is the lesson here.
#[allow(clippy::let_and_return)]
pub fn shadowing() -> usize {
    let spaces = "   ";
    let spaces = spaces.len();
    spaces
}

/// Shadowing a mutable variable with an immutable one is allowed: the new
/// binding decides. This is a common way to "freeze" a value after setup.
// In real code `vec![1, 2]` is simpler; the point is the re-binding.
#[allow(clippy::vec_init_then_push)]
pub fn freeze() -> Vec<i32> {
    let mut v = Vec::new();
    v.push(1);
    v.push(2);
    let v = v; // from here on, v is immutable
    // v.push(3); // error[E0596]: cannot borrow `v` as mutable
    v
}
