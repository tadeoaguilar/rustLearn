//! Module 06 -- Traits & Generics. Reference solution.
//!
//! Traits say what a type can *do*. Generics let one piece of code work for
//! every type that can do it. Two ways to use a trait:
//!
//! * `fn f<T: Shape>(x: &T)` / `impl Shape` -- **static dispatch**: the
//!   compiler makes a copy of `f` per concrete type. Fast, inlinable, but one
//!   `Vec<T>` can only hold one type.
//! * `&dyn Shape` / `Box<dyn Shape>` -- **dynamic dispatch**: one copy of
//!   `f`, calls go through a vtable. A `Vec<Box<dyn Shape>>` can mix types.

pub mod bonus_shapes;
pub mod ex01_basic_traits;
pub mod ex02_generic_functions;
pub mod ex03_trait_bounds;
pub mod ex04_operator_overloading;
pub mod ex05_associated_types;
pub mod ex06_trait_objects;
pub mod ex07_generic_stack;
