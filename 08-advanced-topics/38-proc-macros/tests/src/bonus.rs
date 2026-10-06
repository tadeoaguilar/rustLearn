//! Bonus: #[derive(EnumIter)] used for real.

use crate::sut::EnumIter;

#[derive(EnumIter, Debug, Clone, Copy, PartialEq)]
enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

#[test]
fn bonus_enum_iter() {
    assert_eq!(Suit::COUNT, 4);
    assert_eq!(
        Suit::ALL,
        [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades]
    );
    assert_eq!(Suit::Hearts.name(), "Hearts");
    assert_eq!(Suit::from_name("Spades"), Some(Suit::Spades));
    assert_eq!(Suit::from_name("spades"), None, "exact names");
    for suit in Suit::ALL {
        assert_eq!(Suit::from_name(suit.name()), Some(suit));
    }
}
