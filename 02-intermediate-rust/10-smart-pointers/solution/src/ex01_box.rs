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
    match list {
        Cons(value, rest) => value + sum(rest),
        Nil => 0,
    }
}

pub fn len(list: &List) -> usize {
    match list {
        Cons(_, rest) => 1 + len(rest),
        Nil => 0,
    }
}

/// Built from the back: the last element is the innermost Cons.
pub fn from_slice(values: &[i32]) -> List {
    values
        .iter()
        .rev()
        .fold(Nil, |rest, &v| Cons(v, Box::new(rest)))
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
        Expr::Num(n)
    }
    pub fn plus(a: Expr, b: Expr) -> Expr {
        Expr::Add(Box::new(a), Box::new(b))
    }
    pub fn times(a: Expr, b: Expr) -> Expr {
        Expr::Mul(Box::new(a), Box::new(b))
    }
    pub fn negate(a: Expr) -> Expr {
        Expr::Neg(Box::new(a))
    }
}

/// `a` is a `&Box<Expr>`; `eval(a)` works because &Box<Expr> derefs to &Expr.
pub fn eval(expr: &Expr) -> f64 {
    match expr {
        Expr::Num(n) => *n,
        Expr::Add(a, b) => eval(a) + eval(b),
        Expr::Mul(a, b) => eval(a) * eval(b),
        Expr::Neg(a) => -eval(a),
    }
}

pub fn to_string(expr: &Expr) -> String {
    match expr {
        Expr::Num(n) => n.to_string(),
        Expr::Add(a, b) => format!("({} + {})", to_string(a), to_string(b)),
        Expr::Mul(a, b) => format!("({} * {})", to_string(a), to_string(b)),
        Expr::Neg(a) => format!("-{}", to_string(a)),
    }
}

pub fn run() {
    let list = from_slice(&[1, 2, 3]);
    println!("{list:?}\nsum = {}, len = {}", sum(&list), len(&list));
    println!(
        "size_of::<List>() = {} bytes, however long the list",
        std::mem::size_of::<List>()
    );

    let e = Expr::times(
        Expr::plus(Expr::num(1.0), Expr::num(2.0)),
        Expr::negate(Expr::num(3.0)),
    );
    println!("{} = {}", to_string(&e), eval(&e));
}
