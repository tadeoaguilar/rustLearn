//! Exercise 3: what crosses the JS boundary, and how.
//!
//! Numbers cross for free (they're WASM values). Strings and `Vec`s are
//! *copied* into and out of WASM memory. Structs exported with
//! `#[wasm_bindgen]` stay in WASM memory; JS holds a handle and calls
//! methods. Errors: return `Result<T, E>` with `E: Into<JsValue>` (a
//! `String` works) and the JS side gets an exception.

use wasm_bindgen::prelude::*;

/// Summary statistics of a list of numbers (a `Float64Array` from JS).
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats {
    pub count: u32,
    pub mean: f64,
    pub median: f64,
    pub min: f64,
    pub max: f64,
    /// Population standard deviation.
    pub std_dev: f64,
}

/// `Err` for an empty list or any NaN (JS gets an exception).
#[wasm_bindgen]
pub fn stats(values: &[f64]) -> Result<Stats, String> {
    todo!("Exercise 3")
}

/// Normalize to 0..=1 (a new `Float64Array` back to JS). A constant input
/// maps to all zeros.
#[wasm_bindgen]
pub fn normalize(values: &[f64]) -> Vec<f64> {
    todo!("Exercise 3")
}

/// A validated form, kept in WASM memory; JS reads it through getters.
#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq)]
pub struct Signup {
    email: String,
    age: u8,
}

#[wasm_bindgen]
impl Signup {
    /// Validates: an email with exactly one `@`, a non-empty local part and a
    /// dotted domain; an age from 13 to 120.
    #[wasm_bindgen(constructor)]
    pub fn new(email: &str, age: u32) -> Result<Signup, String> {
        todo!("Exercise 3")
    }

    /// A getter: in JS, `signup.email` (a copy of the string).
    #[wasm_bindgen(getter)]
    pub fn email(&self) -> String {
        todo!("Exercise 3")
    }

    #[wasm_bindgen(getter)]
    pub fn age(&self) -> u8 {
        todo!("Exercise 3")
    }

    /// The part after `@`.
    pub fn domain(&self) -> String {
        todo!("Exercise 3")
    }
}
