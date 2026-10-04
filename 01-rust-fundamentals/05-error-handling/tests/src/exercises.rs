use crate::sut::*;
use std::time::Duration;

// ---- Exercise 1 --------------------------------------------------------------

#[test]
#[should_panic(expected = "Age -5 is unrealistic")]
fn ex1_set_age_panics_on_bad_input() {
    ex01_panic::set_age(-5);
}

#[test]
fn ex1_set_age_and_try_set_age() {
    assert_eq!(ex01_panic::set_age(25), 25);
    assert_eq!(ex01_panic::try_set_age(25), Ok(25));
    assert!(ex01_panic::try_set_age(151).is_err());
    assert!(ex01_panic::try_set_age(-1).is_err());
}

#[test]
fn ex1_get_instead_of_index() {
    assert_eq!(ex01_panic::element_at(&[1, 2, 3], 1), Some(2));
    assert_eq!(ex01_panic::element_at(&[1, 2, 3], 99), None);
}

// ---- Exercise 2 --------------------------------------------------------------

#[test]
fn ex2_divide() {
    assert_eq!(ex02_result::divide(10.0, 2.0), Ok(5.0));
    assert_eq!(
        ex02_result::divide(10.0, 0.0),
        Err("Division by zero".to_string())
    );
}

#[test]
fn ex2_safe_divide_covers_both_errors() {
    use ex02_result::{DivisionError, safe_divide};
    assert_eq!(safe_divide(10, 3), Ok(3));
    assert_eq!(safe_divide(1, 0), Err(DivisionError::DivideByZero));
    assert_eq!(
        safe_divide(i32::MIN, -1),
        Err(DivisionError::Overflow),
        "the one overflowing division"
    );
}

// ---- Exercise 3 --------------------------------------------------------------

#[test]
fn ex3_all_four_readers_agree() {
    use ex03_question_mark::*;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("username.txt");
    std::fs::write(&path, "ferris").unwrap();
    assert_eq!(read_username_v1(&path).unwrap(), "ferris");
    assert_eq!(read_username_v2(&path).unwrap(), "ferris");
    assert_eq!(read_username_v3(&path).unwrap(), "ferris");
    assert_eq!(read_username_v4(&path).unwrap(), "ferris");
}

#[test]
fn ex3_missing_file_is_an_error_not_a_panic() {
    use ex03_question_mark::*;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nope.txt");
    for result in [
        read_username_v1(&path),
        read_username_v2(&path),
        read_username_v3(&path),
        read_username_v4(&path),
    ] {
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }
}

#[test]
fn ex3_chaining_with_question_mark() {
    use ex03_question_mark::*;
    assert_eq!(parse_and_double("21"), Ok(42));
    assert!(parse_and_double("x").is_err());
    assert_eq!(sum_from_strings("10", "20"), Ok(30));
    assert!(sum_from_strings("10", "twenty").is_err());
}

// ---- Exercise 4 --------------------------------------------------------------

#[test]
fn ex4_validation_error() {
    use ex04_custom_errors::validate_age;
    assert_eq!(validate_age(30), Ok(30));
    let err = validate_age(-5).unwrap_err();
    assert_eq!(err.field, "age");
    assert_eq!(
        err.to_string(),
        "Validation error in age: Age cannot be negative"
    );
    assert!(validate_age(151).unwrap_err().message.contains("high"));
}

#[test]
fn ex4_errors_implement_std_error() {
    // Compiles only if the type implements std::error::Error.
    fn is_error<E: std::error::Error>(_: &E) {}
    is_error(&ex04_custom_errors::validate_age(-1).unwrap_err());
    is_error(&ex04_custom_errors::UserError::InvalidAge(-1));
}

#[test]
fn ex4_validate_user() {
    use ex04_custom_errors::{UserError, validate_user};
    assert_eq!(validate_user(30, "a@b.c", "alice"), Ok(()));
    assert_eq!(
        validate_user(-1, "a@b.c", "alice"),
        Err(UserError::InvalidAge(-1))
    );
    assert_eq!(
        validate_user(30, "nope", "alice"),
        Err(UserError::InvalidEmail("nope".into()))
    );
    assert_eq!(
        validate_user(30, "a@b.c", "al"),
        Err(UserError::UsernameTooShort(2))
    );
    assert_eq!(
        UserError::UsernameTooShort(2).to_string(),
        "Username too short: 2 characters"
    );
}

// ---- Exercise 5 --------------------------------------------------------------

#[test]
fn ex5_my_error_wraps_each_source() {
    use ex05_converting_errors::{MyError, read_number, read_number_boxed};
    let dir = tempfile::tempdir().unwrap();
    let good = dir.path().join("good.txt");
    let bad = dir.path().join("bad.txt");
    std::fs::write(&good, " 42\n").unwrap();
    std::fs::write(&bad, "forty-two").unwrap();

    assert_eq!(read_number(&good).unwrap(), 42);
    assert!(matches!(read_number(&bad), Err(MyError::Parse(_))));
    assert!(matches!(
        read_number(&dir.path().join("missing")),
        Err(MyError::Io(_))
    ));

    assert_eq!(read_number_boxed(&good).unwrap(), 42);
    let boxed = read_number_boxed(&bad).unwrap_err();
    assert!(
        boxed.downcast_ref::<std::num::ParseIntError>().is_some(),
        "Box<dyn Error> keeps the original type"
    );
}

#[test]
fn ex5_from_impls_power_the_question_mark() {
    use ex05_converting_errors::MyError;
    let parse_err = "x".parse::<i32>().unwrap_err();
    assert!(matches!(MyError::from(parse_err), MyError::Parse(_)));
    let io_err = std::io::Error::other("boom");
    let e = MyError::from(io_err);
    assert_eq!(e.to_string(), "IO error: boom");
}

// ---- Exercise 6 --------------------------------------------------------------

#[test]
fn ex6_ok_or_and_ok_or_else() {
    use ex06_option_to_result::*;
    assert_eq!(get_user_id_result("alice"), Ok(1));
    assert_eq!(
        get_user_id_result("bob"),
        Err("User 'bob' not found".to_string())
    );
    assert_eq!(get_user_id_lazy("alice"), Ok(1));
    assert_eq!(
        get_user_id_lazy("bob"),
        Err("User 'bob' not found".to_string())
    );
}

#[test]
fn ex6_transpose() {
    use ex06_option_to_result::*;
    for f in [parse_optional, parse_optional_v2] {
        assert_eq!(f(Some("5")), Ok(Some(5)));
        assert_eq!(f(None), Ok(None));
        assert!(f(Some("x")).is_err());
    }
}

// ---- Exercise 7 --------------------------------------------------------------

#[test]
fn ex7_retry_succeeds_once_the_operation_does() {
    let mut calls = 0;
    let result: Result<u32, String> = ex07_recovery::retry_with_delay(
        || {
            calls += 1;
            if calls < 3 {
                Err(format!("fail {calls}"))
            } else {
                Ok(calls)
            }
        },
        5,
        Duration::ZERO,
    );
    assert_eq!(result, Ok(3));
    assert_eq!(calls, 3, "stops as soon as it succeeds");
}

#[test]
fn ex7_retry_gives_up_with_the_last_error() {
    let mut calls = 0;
    let result: Result<(), String> = ex07_recovery::retry_with_delay(
        || {
            calls += 1;
            Err(format!("fail {calls}"))
        },
        4,
        Duration::ZERO,
    );
    assert_eq!(result, Err("fail 4".to_string()));
    assert_eq!(calls, 4, "max_retries is the maximum number of attempts");
}

#[test]
fn ex7_backoff_and_fallback() {
    use ex07_recovery::*;
    let ms = Duration::from_millis;
    assert_eq!(backoff_delays(ms(10), 4), vec![ms(10), ms(20), ms(40)]);
    assert_eq!(backoff_delays(ms(10), 1), vec![]);
    assert_eq!(get_config_with_default("timeout", "30"), "30");
}

// ---- Exercise 8 --------------------------------------------------------------

#[test]
fn ex8_parses_the_sample() {
    use ex08_config_parser::{Config, SAMPLE};
    let config = Config::parse(SAMPLE).unwrap();
    assert_eq!(config.get_int("port").unwrap(), 8080);
    assert_eq!(config.get("host"), Some("localhost"));
    assert!(config.get_bool("debug").unwrap());
    assert_eq!(config.get_or("missing", "fallback"), "fallback");
}

#[test]
fn ex8_values_may_contain_equals_signs() {
    let config = ex08_config_parser::Config::parse("url = http://x/?a=b").unwrap();
    assert_eq!(config.get("url"), Some("http://x/?a=b"));
}

#[test]
fn ex8_reports_the_line_number_of_bad_lines() {
    use ex08_config_parser::{Config, ConfigError};
    let err = Config::parse("# comment\n\nport = 1\nnonsense\n").unwrap_err();
    match err {
        ConfigError::InvalidFormat { line_num, line } => {
            assert_eq!(
                line_num, 4,
                "line numbers are 1-based and count comments and blanks"
            );
            assert_eq!(line, "nonsense");
        }
        other => panic!("expected InvalidFormat, got {other:?}"),
    }
}

#[test]
fn ex8_typed_getters_report_bad_values() {
    use ex08_config_parser::{Config, ConfigError};
    let config = Config::parse("port = eighty\ndebug = maybe\nyes = YES").unwrap();
    assert!(matches!(
        config.get_int("port"),
        Err(ConfigError::InvalidValue { .. })
    ));
    assert!(matches!(
        config.get_bool("debug"),
        Err(ConfigError::InvalidValue { .. })
    ));
    assert!(
        config.get_bool("yes").unwrap(),
        "booleans are case-insensitive"
    );
    assert!(matches!(
        config.get_int("nope"),
        Err(ConfigError::MissingKey { .. })
    ));
}

#[test]
fn ex8_from_file_and_start_server() {
    use ex08_config_parser::{ConfigError, SAMPLE, start_server};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.txt");
    std::fs::write(&path, SAMPLE).unwrap();
    let msg = start_server(&path).unwrap();
    assert!(msg.contains("localhost:8080"), "got: {msg}");
    assert!(msg.contains("Debug mode: true"), "got: {msg}");
    assert!(matches!(
        start_server(dir.path().join("missing.txt")),
        Err(ConfigError::Io(_))
    ));
}

// ---- Exercise 9 --------------------------------------------------------------

#[test]
fn ex9_thiserror_messages() {
    use ex09_thiserror::*;
    assert_eq!(validate_percentage(50).unwrap(), 50);
    assert_eq!(
        validate_percentage(150).unwrap_err().to_string(),
        "Invalid value: 150 (must be between 0 and 100)"
    );
    assert_eq!(DataError::Custom("x".into()).to_string(), "Custom error: x");
}

#[test]
fn ex9_from_conversions() {
    use ex09_thiserror::*;
    assert_eq!(parse_percentage(" 42 ").unwrap(), 42);
    assert!(matches!(parse_percentage("abc"), Err(DataError::Parse(_))));
    assert!(matches!(
        parse_percentage("101"),
        Err(DataError::OutOfRange { value: 101, .. })
    ));
}

#[test]
fn ex9_anyhow_adds_context() {
    let err =
        ex09_thiserror::read_percentage_file(std::path::Path::new("/no/such/file")).unwrap_err();
    let full = format!("{err:#}");
    assert!(
        full.starts_with("reading percentage from /no/such/file"),
        "got: {full}"
    );
}

// ---- Bonus -------------------------------------------------------------------

#[test]
fn bonus_collects_all_errors() {
    use bonus_combinators::register_user;
    let errors = register_user("ab", "invalid", 10).unwrap_err();
    assert_eq!(
        errors,
        vec![
            "Username too short",
            "Invalid email format",
            "Must be at least 13 years old"
        ]
    );
    let errors = register_user("ferris", "ferris@rust-lang.org", 200).unwrap_err();
    assert_eq!(errors, vec!["Age is unrealistic"]);
}

#[test]
fn bonus_success() {
    use bonus_combinators::{User, register_user, register_user_fail_fast};
    let expected = User {
        username: "ferris".into(),
        email: "ferris@rust-lang.org".into(),
        age: 30,
    };
    assert_eq!(
        register_user("ferris", "ferris@rust-lang.org", 30),
        Ok(expected.clone())
    );
    assert_eq!(
        register_user_fail_fast("ferris", "ferris@rust-lang.org", 30),
        Ok(expected)
    );
    assert_eq!(
        register_user_fail_fast("ab", "invalid", 10),
        Err("Username too short".into())
    );
}

#[test]
fn bonus_validators_edge_cases() {
    use bonus_combinators::*;
    assert!(validate_username(&"x".repeat(20)).is_ok());
    assert!(validate_username(&"x".repeat(21)).is_err());
    assert!(validate_email("@nobody.com").is_err());
    assert!(validate_email("a@b").is_err());
    assert!(validate_age(13).is_ok());
    assert!(validate_age(120).is_ok());
}

#[test]
fn bonus_combinator_tour() {
    use bonus_combinators::combinator_tour;
    assert_eq!(combinator_tour("30"), Ok(31));
    assert_eq!(combinator_tour("unknown"), Ok(0));
    assert!(combinator_tour("5").is_err());
    assert!(combinator_tour("abc").unwrap_err().contains("not a number"));
}
