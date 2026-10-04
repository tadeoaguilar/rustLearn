use crate::sut::*;

fn transcript<F: FnOnce(&[u8], &mut Vec<u8>)>(input: &str, f: F) -> String {
    let mut out = Vec::new();
    f(input.as_bytes(), &mut out);
    String::from_utf8(out).unwrap()
}

// ---- Exercise 1: variables ---------------------------------------------------

#[test]
fn ex1_shadowing_changes_the_type() {
    assert_eq!(ex01_variables::shadowing(), 3);
}

#[test]
fn ex1_freeze() {
    assert_eq!(ex01_variables::freeze(), vec![1, 2]);
}

// ---- Exercise 2: data types --------------------------------------------------

#[test]
fn ex2_type_sizes() {
    let sizes = ex02_data_types::type_sizes();
    let size = |name: &str| sizes.iter().find(|(n, _)| *n == name).map(|(_, s)| *s);
    assert_eq!(size("i8"), Some(1));
    assert_eq!(size("i128"), Some(16));
    assert_eq!(size("f32"), Some(4));
    assert_eq!(
        size("char"),
        Some(4),
        "a char is a 4-byte Unicode scalar, not a byte"
    );
    assert_eq!(size("isize"), Some(std::mem::size_of::<usize>()));
}

#[test]
fn ex2_overflow_strategies() {
    let (wrapping, checked, saturating, overflowing) = ex02_data_types::overflow_strategies(127, 1);
    assert_eq!(wrapping, -128);
    assert_eq!(checked, None);
    assert_eq!(saturating, 127);
    assert_eq!(overflowing, (-128, true));

    let (_, checked, _, overflowing) = ex02_data_types::overflow_strategies(1, 1);
    assert_eq!(checked, Some(2));
    assert_eq!(overflowing, (2, false));
}

#[test]
fn ex2_compound_types() {
    assert_eq!(ex02_data_types::array_example().len(), 5);
    let (_, _, c, _) = ex02_data_types::tuple_example();
    assert!(c.len_utf8() >= 1);
}

// ---- Exercise 3: functions ---------------------------------------------------

#[test]
fn ex3_functions_from_the_exercise() {
    use ex03_functions::*;
    assert_eq!(greeting("Alice"), "Hello, Alice!");
    assert_eq!(add(2, 3), 5);
    assert!(is_even(4));
    assert!(!is_even(7));
    assert!(is_even(-2));
    assert_eq!(celsius_to_fahrenheit(0.0), 32.0);
    assert_eq!(celsius_to_fahrenheit(100.0), 212.0);
    assert_eq!(swap(1, 2), (2, 1));
}

#[test]
fn ex3_bonus_apply_twice() {
    fn inc(n: i32) -> i32 {
        n + 1
    }
    assert_eq!(ex03_functions::apply_twice(inc, 5), 7);
    assert_eq!(ex03_functions::apply_twice(|n| n * 10, 2), 200);
}

// ---- Exercise 4: if/else -----------------------------------------------------

#[test]
fn ex4_letter_grade_boundaries() {
    use ex04_if_else::get_letter_grade as grade;
    assert_eq!(grade(100), 'A');
    assert_eq!(grade(90), 'A');
    assert_eq!(grade(89), 'B');
    assert_eq!(grade(80), 'B');
    assert_eq!(grade(79), 'C');
    assert_eq!(grade(70), 'C');
    assert_eq!(grade(69), 'D');
    assert_eq!(grade(60), 'D');
    assert_eq!(grade(59), 'F');
    assert_eq!(grade(0), 'F');
}

#[test]
fn ex4_if_in_let() {
    assert_eq!(ex04_if_else::if_in_let(true), 5);
    assert_eq!(ex04_if_else::if_in_let(false), 6);
}

#[test]
fn ex4_leap_years() {
    use ex04_if_else::is_leap_year;
    assert!(is_leap_year(2024));
    assert!(is_leap_year(2000), "divisible by 400");
    assert!(!is_leap_year(1900), "divisible by 100 but not 400");
    assert!(!is_leap_year(2023));
}

// ---- Exercise 5: loops -------------------------------------------------------

#[test]
fn ex5_loop_with_break_value() {
    assert_eq!(ex05_loops::find_first_multiple_of_7(), 7);
}

#[test]
fn ex5_countdown() {
    assert_eq!(ex05_loops::countdown(3), vec!["3", "2", "1", "Liftoff!"]);
    assert_eq!(ex05_loops::countdown(0), vec!["Liftoff!"]);
}

#[test]
fn ex5_fibonacci() {
    assert_eq!(ex05_loops::fibonacci(8), vec![0, 1, 1, 2, 3, 5, 8, 13]);
    assert_eq!(ex05_loops::fibonacci(0), Vec::<u64>::new());
    assert_eq!(ex05_loops::fibonacci(1), vec![0]);
}

#[test]
fn ex5_labelled_break_leaves_both_loops() {
    // i * j > 10 first happens at i = 3, j = 4. Without the label the outer
    // loop would continue to i = 4 and the last match would be (4, 3).
    assert_eq!(ex05_loops::nested_loop_example(), Some((3, 4)));
}

// ---- Exercise 6: match -------------------------------------------------------

#[test]
fn ex6_coins() {
    use ex06_match::{Coin, value_in_cents};
    let total: u32 = [Coin::Penny, Coin::Nickel, Coin::Dime, Coin::Quarter]
        .into_iter()
        .map(value_in_cents)
        .sum();
    assert_eq!(total, 41);
}

#[test]
fn ex6_classify_number() {
    use ex06_match::classify_number;
    assert_eq!(classify_number(-3), "negative");
    assert_eq!(classify_number(0), "zero");
    assert_eq!(classify_number(1), "small positive");
    assert_eq!(classify_number(10), "small positive");
    assert_eq!(classify_number(11), "large positive");
}

#[test]
fn ex6_guards_and_options() {
    assert_eq!(ex06_match::describe_number(-4), "-4 is negative");
    assert!(ex06_match::describe_number(4).contains("even"));
    assert!(ex06_match::describe_number(5).contains("odd"));
    assert_eq!(ex06_match::handle_input(Some(3)), "Got 3");
    assert_eq!(ex06_match::handle_input(None), "Got nothing");
}

// ---- Exercise 7: calculator --------------------------------------------------

#[test]
fn ex7_parse_operation() {
    use ex07_calculator::parse_operation;
    assert_eq!(parse_operation("5 + 3"), Some((5.0, '+', 3.0)));
    assert_eq!(parse_operation("  2.5   *  4 "), Some((2.5, '*', 4.0)));
    assert_eq!(parse_operation("-1 - -1"), Some((-1.0, '-', -1.0)));
    assert_eq!(parse_operation("5 % 3"), None, "unsupported operator");
    assert_eq!(parse_operation("five + 3"), None);
    assert_eq!(parse_operation("5 +"), None);
    assert_eq!(parse_operation(""), None);
}

#[test]
fn ex7_calculate() {
    use ex07_calculator::calculate;
    assert_eq!(calculate(5.0, '+', 3.0), Some(8.0));
    assert_eq!(calculate(5.0, '-', 3.0), Some(2.0));
    assert_eq!(calculate(5.0, '*', 3.0), Some(15.0));
    assert_eq!(calculate(10.0, '/', 4.0), Some(2.5));
    assert_eq!(calculate(10.0, '/', 0.0), None, "division by zero");
}

#[test]
fn ex7_session_matches_expected_output() {
    let out = transcript("5 + 3\n10 / 2\n10 / 0\nquit\n", |i, o| {
        ex07_calculator::run(i, o).unwrap()
    });
    assert!(out.contains("Result: 8\n"), "got: {out}");
    assert!(out.contains("Result: 5\n"), "got: {out}");
    assert!(out.contains("Error: Division by zero"), "got: {out}");
    assert!(out.trim_end().ends_with("Goodbye!"), "got: {out}");
}

#[test]
fn ex7_session_ends_at_end_of_input_without_quit() {
    let out = transcript("1 + 1\n", |i, o| ex07_calculator::run(i, o).unwrap());
    assert!(out.contains("Result: 2"), "got: {out}");
}

// ---- Exercise 8: FizzBuzz ----------------------------------------------------

#[test]
fn ex8_fizzbuzz_values() {
    use ex08_fizzbuzz::fizzbuzz_match as fb;
    assert_eq!(fb(1), "1");
    assert_eq!(fb(3), "Fizz");
    assert_eq!(fb(5), "Buzz");
    assert_eq!(fb(15), "FizzBuzz");
    assert_eq!(fb(98), "98");
}

#[test]
fn ex8_all_three_versions_agree() {
    use ex08_fizzbuzz::*;
    let a = fizzbuzz_up_to(100, fizzbuzz_if);
    let b = fizzbuzz_up_to(100, fizzbuzz_match);
    let c = fizzbuzz_up_to(100, fizzbuzz_guard);
    assert_eq!(a.len(), 100);
    assert_eq!(a, b);
    assert_eq!(b, c);
}

// ---- Exercise 9: temperature -------------------------------------------------

#[test]
fn ex9_test_cases_from_the_exercise() {
    use ex09_temperature::*;
    assert_eq!(celsius_to_fahrenheit(0.0), 32.0);
    assert_eq!(celsius_to_fahrenheit(100.0), 212.0);
    assert_eq!(fahrenheit_to_celsius(32.0), 0.0);
    assert_eq!(fahrenheit_to_celsius(-40.0), -40.0);
    assert_eq!(celsius_to_fahrenheit(-40.0), -40.0);
}

#[test]
fn ex9_convert_picks_the_direction_from_the_unit() {
    use ex09_temperature::convert;
    assert_eq!(convert(100.0, 'C'), Some((212.0, 'F')));
    assert_eq!(convert(212.0, 'f'), Some((100.0, 'C')));
    assert_eq!(convert(1.0, 'K'), None);
}

#[test]
fn ex9_interactive() {
    let out = transcript("100\nC\n", |i, o| ex09_temperature::run(i, o).unwrap());
    assert!(out.contains("212"), "got: {out}");
}

// ---- Bonus: primes -----------------------------------------------------------

#[test]
fn bonus_is_prime() {
    use bonus_primes::is_prime;
    assert!(is_prime(2));
    assert!(is_prime(17));
    assert!(!is_prime(0));
    assert!(!is_prime(1));
    assert!(!is_prime(4));
    assert!(!is_prime(91), "7 * 13");
    assert!(is_prime(1_000_000_007));
}

#[test]
fn bonus_nth_prime() {
    use bonus_primes::nth_prime;
    assert_eq!(nth_prime(1), 2);
    assert_eq!(nth_prime(10), 29);
    assert_eq!(nth_prime(100), 541);
    assert_eq!(nth_prime(10_000), 104_729);
}

#[test]
fn bonus_primes_up_to() {
    use bonus_primes::*;
    assert_eq!(primes_up_to(10), vec![2, 3, 5, 7]);
    assert_eq!(primes_up_to(1), Vec::<u64>::new());
    assert_eq!(primes_up_to(2), vec![2]);
    // the sieve and trial division must agree
    let sieve = primes_up_to(2_000);
    let trial: Vec<u64> = (0..=2_000).filter(|&n| is_prime(n)).collect();
    assert_eq!(sieve, trial);
}
