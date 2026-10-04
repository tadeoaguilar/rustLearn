use crate::sut;
use crate::sut::ex01_basics::{calls, next_value, reset_calls};

#[test]
fn square_evaluates_its_argument_once() {
    assert_eq!(sut::square!(4), 16);
    assert_eq!(sut::square!(1 + 2), 9);
    reset_calls();
    assert_eq!(sut::square!(next_value()), 9);
    assert_eq!(
        calls(),
        1,
        "square! must evaluate its argument exactly once"
    );
}

#[test]
fn square_naive_shows_the_bug() {
    reset_calls();
    assert_eq!(sut::square_naive!(next_value()), 9);
    assert_eq!(calls(), 2);
}

#[test]
fn max_any_number_of_arguments() {
    assert_eq!(sut::max!(3), 3);
    assert_eq!(sut::max!(3, 9), 9);
    assert_eq!(sut::max!(3, 9, 2, 7), 9);
    assert_eq!(sut::max!(-1, -5, -3,), -1, "trailing comma");
    assert_eq!(sut::max!(1.5, 0.5), 1.5);
}

#[test]
fn hashmap_literal() {
    let m = sut::hashmap! { "a" => 1, "b" => 2, };
    assert_eq!(m.len(), 2);
    assert_eq!(m["b"], 2);
    let empty: std::collections::HashMap<i32, i32> = sut::hashmap! {};
    assert!(empty.is_empty());
}

#[test]
fn vec_of_strings() {
    let v: Vec<String> = sut::vec_of_strings!["a", 'b', 3];
    assert_eq!(v, vec!["a", "b", "3"]);
    let none: Vec<String> = sut::vec_of_strings![];
    assert!(none.is_empty());
}
