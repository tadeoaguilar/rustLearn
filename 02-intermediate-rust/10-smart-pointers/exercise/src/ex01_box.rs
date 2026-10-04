//! Exercise 1: Box and Recursive Types.
//!
//! ```text
//! enum List { Cons(i32, List), Nil }
//! ^^^^^^^^^ error[E0072]: recursive type `List` has infinite size
//!           help: insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle
//! ```
//!
//! To lay out a `List` the compiler needs its size, which includes a `List`,
//! which includes a `List`... A `Box<List>` is a pointer: always 8 bytes,
//! whatever it points to.

#[derive(Debug, PartialEq)]
pub enum List {
    Cons(i32, Box<List>),
    Nil,
}

use List::{Cons, Nil};

pub fn sum(list: &List) -> i32 {
    todo!("Exercise 1")
}

pub fn len(list: &List) -> usize {
    todo!("Exercise 1")
}

/// Built from the back: the last element is the innermost Cons.
pub fn from_slice(values: &[i32]) -> List {
    todo!("Exercise 1")
}

/// Task 2: an expression tree. Each operator owns its operands through a Box.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(f64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
}

impl Expr {
    // Small constructors keep tree-building readable. (Not `add`/`mul`/`neg`:
    // those names belong to the operator traits, and clippy would object.)
    pub fn num(n: f64) -> Expr {
        todo!("Exercise 1")
    }
    pub fn plus(a: Expr, b: Expr) -> Expr {
        todo!("Exercise 1")
    }
    pub fn times(a: Expr, b: Expr) -> Expr {
        todo!("Exercise 1")
    }
    pub fn negate(a: Expr) -> Expr {
        todo!("Exercise 1")
    }
}

/// `a` is a `&Box<Expr>`; `eval(a)` works because &Box<Expr> derefs to &Expr.
pub fn eval(expr: &Expr) -> f64 {
    todo!("Exercise 1")
}

pub fn to_string(expr: &Expr) -> String {
    todo!("Exercise 1")
}

pub fn run() {
    todo!("Exercise 1")
}
