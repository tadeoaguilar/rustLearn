use crate::sut;
use crate::sut::ex03_validation::{HexError, Rgb, parse_hex_color};

// If these compile, the checks passed at compile time.
sut::const_assert!(std::mem::size_of::<u32>() == 4);
const ORANGE: Rgb = sut::hex_color!("#ff8800");

#[test]
fn hex_color_at_compile_time() {
    assert_eq!(
        ORANGE,
        Rgb {
            r: 255,
            g: 136,
            b: 0
        }
    );
    assert_eq!(
        sut::hex_color!("#00FFaa"),
        Rgb {
            r: 0,
            g: 255,
            b: 170
        }
    );
}

#[test]
fn parse_hex_color_errors() {
    assert_eq!(parse_hex_color("ff8800"), Err(HexError::MissingHash));
    assert_eq!(parse_hex_color(""), Err(HexError::MissingHash));
    assert_eq!(parse_hex_color("#ff88"), Err(HexError::WrongLength));
    assert_eq!(parse_hex_color("#gg0000"), Err(HexError::BadDigit));
    assert_eq!(parse_hex_color("#000000"), Ok(Rgb { r: 0, g: 0, b: 0 }));
}

#[test]
fn nonzero() {
    let n: std::num::NonZeroU32 = sut::nonzero!(5);
    assert_eq!(n.get(), 5);
    const N: std::num::NonZeroU32 = sut::nonzero!(1);
    assert_eq!(N.get(), 1);
}

// The compile-time *failures* (nonzero!(0), hex_color!("#ff88"), a false
// const_assert!) are checked as `compile_fail` doc tests in the solution:
//     cargo test -p m13-macros-solution --doc
