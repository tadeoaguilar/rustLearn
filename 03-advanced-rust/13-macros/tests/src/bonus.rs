use crate::sut;
use crate::sut::bonus_attribute::*;

#[test]
fn timed_functions_return_the_same_values() {
    assert_eq!(slow_sum(1_000), 499_500);
    assert_eq!(parse_positive("42"), Ok(42));
    assert_eq!(
        parse_positive("-1"),
        Err("-1 is not positive".to_string()),
        "early return still works"
    );
    assert!(parse_positive("x").is_err(), "`?` still works");
    assert_eq!(greet("Ferris".into()), "Hello, Ferris!");
}

#[test]
fn timed_works_in_another_crate_and_on_unit_functions() {
    #[sut::timed]
    fn noop() {}
    #[sut::timed("custom label")]
    fn add(a: i32, b: i32) -> i32 {
        a + b
    }
    noop();
    assert_eq!(add(2, 3), 5);
}
