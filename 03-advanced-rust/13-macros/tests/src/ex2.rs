use crate::sut;
use crate::sut::ex02_items::{Color, Meters, Seconds};

#[test]
fn count_tokens() {
    assert_eq!(sut::count!(), 0);
    assert_eq!(sut::count!(a b c), 3);
    const N: usize = sut::count!(x y);
    assert_eq!(N, 2, "usable in const context");
}

#[test]
fn newtype_generates_struct_and_impls() {
    let m: Meters = 3.5.into();
    assert_eq!(m.value(), 3.5);
    assert_eq!(m.to_string(), "3.5");
    assert!(Meters::from(1.0) < Meters::from(2.0), "PartialOrd derived");
    let s = Seconds::from(90);
    let copy = s; // Copy derived
    assert_eq!(s, copy);
}

#[test]
fn newtype_works_in_the_caller_crate() {
    sut::newtype!(pub(crate) Kilograms(u32));
    assert_eq!(Kilograms::from(5).value(), 5);
}

#[test]
fn string_enum() {
    assert_eq!(Color::Green.as_str(), "Green");
    assert_eq!(Color::Blue.to_string(), "Blue");
    assert_eq!("Red".parse::<Color>(), Ok(Color::Red));
    assert!("Pink".parse::<Color>().is_err());
    assert_eq!(Color::ALL, [Color::Red, Color::Green, Color::Blue]);
    assert_eq!(Color::COUNT, 3);
}
