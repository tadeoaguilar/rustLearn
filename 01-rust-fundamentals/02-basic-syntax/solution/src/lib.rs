//! Module 02 -- Basic Syntax. Reference solution.
//!
//! A habit you will see in every exercise: functions *return* values and the
//! caller decides whether to print them. `fibonacci(10)` returning a `Vec` can
//! be tested; `print_fibonacci(10)` writing to the terminal cannot.

// Clippy (1.87+) suggests `n.is_multiple_of(3)` over `n % 3 == 0`. This
// module is where the `%` operator is taught, so it stays.
#![allow(clippy::manual_is_multiple_of)]

pub mod bonus_primes;
pub mod ex01_variables;
pub mod ex02_data_types;
pub mod ex03_functions;
pub mod ex04_if_else;
pub mod ex05_loops;
pub mod ex06_match;
pub mod ex07_calculator;
pub mod ex08_fizzbuzz;
pub mod ex09_temperature;
