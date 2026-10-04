//! Exercise 3: Compile-Time Validation.
//!
//! A `panic!` while evaluating a constant is a *compile* error. So: write the
//! check as a `const fn`, and call it from a `const { }` block in the macro.

/// Fails the build if the condition is false.
///
/// ```compile_fail,E0080
/// m13_macros_solution::const_assert!(1 + 1 == 3);
/// ```
#[macro_export]
macro_rules! const_assert {
    ($cond:expr $(,)?) => {
        const _: () = ::std::assert!($cond);
    };
}

/// A `NonZeroU32` checked at compile time.
///
/// ```
/// let n = m13_macros_solution::nonzero!(5);
/// assert_eq!(n.get(), 5);
/// ```
///
/// ```compile_fail,E0080
/// let n = m13_macros_solution::nonzero!(0); // error: evaluation of constant value failed
/// ```
#[macro_export]
macro_rules! nonzero {
    ($n:expr) => {
        const {
            match ::std::num::NonZeroU32::new($n) {
                ::std::option::Option::Some(v) => v,
                ::std::option::Option::None => ::std::panic!("nonzero! needs a non-zero value"),
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HexError {
    MissingHash,
    WrongLength,
    BadDigit,
}

/// Parses "#rrggbb". A `const fn`: no iterators, no `?`, no `str` methods
/// that aren't const -- plain loops over bytes.
pub const fn parse_hex_color(s: &str) -> Result<Rgb, HexError> {
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes[0] != b'#' {
        return Err(HexError::MissingHash);
    }
    if bytes.len() != 7 {
        return Err(HexError::WrongLength);
    }
    let mut values = [0u8; 3];
    let mut i = 0;
    while i < 3 {
        let hi = match hex_digit(bytes[1 + i * 2]) {
            Some(v) => v,
            None => return Err(HexError::BadDigit),
        };
        let lo = match hex_digit(bytes[2 + i * 2]) {
            Some(v) => v,
            None => return Err(HexError::BadDigit),
        };
        values[i] = hi * 16 + lo;
        i += 1;
    }
    Ok(Rgb {
        r: values[0],
        g: values[1],
        b: values[2],
    })
}

const fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// A colour literal, validated at compile time.
///
/// ```
/// use m13_macros_solution::{hex_color, ex03_validation::Rgb};
/// const ORANGE: Rgb = hex_color!("#ff8800");
/// assert_eq!(ORANGE, Rgb { r: 255, g: 136, b: 0 });
/// ```
///
/// ```compile_fail,E0080
/// let c = m13_macros_solution::hex_color!("#ff88"); // too short: build fails
/// ```
///
/// ```compile_fail,E0080
/// let c = m13_macros_solution::hex_color!("ff8800"); // no '#': build fails
/// ```
#[macro_export]
macro_rules! hex_color {
    ($s:literal) => {
        const {
            match $crate::ex03_validation::parse_hex_color($s) {
                ::std::result::Result::Ok(c) => c,
                ::std::result::Result::Err(_) => ::std::panic!(concat!("invalid hex color: ", $s)),
            }
        }
    };
}

crate::const_assert!(std::mem::size_of::<u64>() == 8);
crate::const_assert!(std::mem::size_of::<Rgb>() == 3);

pub const ORANGE: Rgb = crate::hex_color!("#ff8800");

pub fn run() {
    println!("ORANGE = {ORANGE:?}");
    println!("nonzero!(5) = {}", crate::nonzero!(5));
    println!(
        "runtime parse of \"#12345\" = {:?}",
        parse_hex_color("#12345")
    );
    println!("const_assert! checks ran at compile time -- or this wouldn't have built.");
}
