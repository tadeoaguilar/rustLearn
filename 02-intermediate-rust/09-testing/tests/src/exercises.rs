use crate::sut::*;
use proptest::prelude::*;

// ---- Bug 1 and 2: stats ------------------------------------------------------

#[test]
fn bug1_mean_of_empty_input_is_none() {
    assert_eq!(stats::mean(&[]), None);
}

#[test]
fn bug2_median_of_even_length_is_the_average_of_the_middle_two() {
    assert_eq!(stats::median(&[1.0, 2.0, 3.0, 4.0]), Some(2.5));
}

#[test]
fn stats_still_work() {
    assert_eq!(stats::mean(&[1.0, 2.0, 3.0]), Some(2.0));
    assert_eq!(stats::median(&[3.0, 1.0, 2.0]), Some(2.0));
    assert_eq!(stats::mode(&[1, 2, 2, 3]), Some(2));
    assert_eq!(
        stats::std_dev(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]),
        Some(2.0)
    );
}

// ---- Bug 3: account ----------------------------------------------------------

#[test]
fn bug3_failed_transfer_to_a_frozen_account_loses_no_money() {
    use account::{Account, AccountError, transfer};
    let mut from = Account::new("A", 100);
    let mut to = Account::new("B", 0);
    to.freeze();
    assert_eq!(transfer(&mut from, &mut to, 30), Err(AccountError::Frozen));
    assert_eq!(from.balance() + to.balance(), 100, "money vanished");
    assert_eq!(from.balance(), 100);
}

#[test]
fn accounts_still_work() {
    use account::{Account, AccountError, transfer};
    let mut a = Account::new("A", 100);
    let mut b = Account::new("B", 0);
    transfer(&mut a, &mut b, 40).unwrap();
    assert_eq!((a.balance(), b.balance()), (60, 40));
    assert_eq!(
        a.withdraw(1_000),
        Err(AccountError::InsufficientFunds {
            balance: 60,
            requested: 1_000
        })
    );
    assert_eq!(a.deposit(0), Err(AccountError::ZeroAmount));
}

#[test]
#[should_panic(expected = "owner name must not be empty")]
fn account_rejects_an_empty_owner() {
    account::Account::new("", 1);
}

// ---- Bug 4: weather ----------------------------------------------------------

struct Fixed(f64);
impl weather::WeatherApi for Fixed {
    fn temperature_c(&self, _: &str) -> Result<f64, weather::WeatherError> {
        Ok(self.0)
    }
}

#[test]
fn bug4_exactly_25_degrees_is_hot() {
    let advisor = weather::Advisor::new(Fixed(25.0));
    assert_eq!(advisor.advise("x"), Ok(weather::Advice::Hot));
}

#[test]
fn weather_other_boundaries() {
    use weather::Advice::*;
    let at = |t| weather::Advice::for_temperature(t);
    assert_eq!(
        (at(-0.1), at(0.0), at(9.9), at(10.0), at(24.9)),
        (Freezing, Cold, Cold, Mild, Mild)
    );
}

// ---- Bug 5: roman ------------------------------------------------------------

#[test]
fn bug5_ninety_and_nine_hundred_use_subtractive_notation() {
    assert_eq!(roman::to_roman(90).as_deref(), Ok("XC"));
    assert_eq!(roman::to_roman(900).as_deref(), Ok("CM"));
    assert_eq!(roman::to_roman(1994).as_deref(), Ok("MCMXCIV"));
}

proptest! {
    #[test]
    fn roman_roundtrip_holds_even_with_bug5(n in 1u32..=3999) {
        prop_assert_eq!(roman::from_roman(&roman::to_roman(n).unwrap()), Ok(n));
    }
}

// ---- Bug 6: slug -------------------------------------------------------------

#[test]
fn bug6_runs_of_separators_collapse_to_one_hyphen() {
    assert_eq!(slug::slugify("Hello, World!"), "hello-world");
    assert_eq!(slug::slugify("a  b"), "a-b");
}

#[test]
fn slug_still_works() {
    assert_eq!(slug::slugify("Hello World"), "hello-world");
    assert_eq!(slug::slugify("  trim me  "), "trim-me");
}

// ---- Not bugs: the bonus and the benchmark subject ---------------------------

#[test]
fn words_implementations_agree() {
    let text = "a b a c b a";
    let split = words::count_words_split(text);
    let borrowed = words::count_words_borrowed(text);
    assert_eq!(split["a"], 3);
    assert_eq!(borrowed["a"], 3);
    assert_eq!(split.len(), borrowed.len());
}

#[test]
fn bonus_bowling() {
    assert_eq!(bowling::score(&[0; 20]), 0);
    assert_eq!(bowling::score(&[1; 20]), 20);
    assert_eq!(bowling::score(&[10; 12]), 300);
    assert_eq!(bowling::score(&[5; 21]), 150);
}
