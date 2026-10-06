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
    if values.is_empty() {
        return Err("no values".into());
    }
    if values.iter().any(|v| v.is_nan()) {
        return Err("NaN in the input".into());
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let n = values.len();
    let mean = values.iter().sum::<f64>() / n as f64;
    let median = if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    };
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    Ok(Stats {
        count: n as u32,
        mean,
        median,
        min: sorted[0],
        max: sorted[n - 1],
        std_dev: variance.sqrt(),
    })
}

/// Normalize to 0..=1 (a new `Float64Array` back to JS). A constant input
/// maps to all zeros.
#[wasm_bindgen]
pub fn normalize(values: &[f64]) -> Vec<f64> {
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;
    values
        .iter()
        .map(|v| if range > 0.0 { (v - min) / range } else { 0.0 })
        .collect()
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
        let email = email.trim();
        let valid_email = match email.split_once('@') {
            Some((local, domain)) => {
                !local.is_empty()
                    && !domain.contains('@')
                    && domain.contains('.')
                    && !domain.starts_with('.')
                    && !domain.ends_with('.')
            }
            None => false,
        };
        if !valid_email {
            return Err(format!("invalid email: {email}"));
        }
        if !(13..=120).contains(&age) {
            return Err(format!("age must be 13 to 120, not {age}"));
        }
        Ok(Signup {
            email: email.to_lowercase(),
            age: age as u8,
        })
    }

    /// A getter: in JS, `signup.email` (a copy of the string).
    #[wasm_bindgen(getter)]
    pub fn email(&self) -> String {
        self.email.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn age(&self) -> u8 {
        self.age
    }

    /// The part after `@`.
    pub fn domain(&self) -> String {
        self.email
            .split_once('@')
            .map(|(_, d)| d.to_string())
            .unwrap_or_default()
    }
}
