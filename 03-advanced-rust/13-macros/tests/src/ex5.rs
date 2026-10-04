use crate::sut;
use crate::sut::ex05_hygiene::{hygiene_demo, take_trace_log};

#[test]
fn make_var_and_hygiene() {
    sut::make_var!(answer, 41);
    assert_eq!(answer + 1, 42);
    let value = 100;
    assert_eq!(sut::double!(value + 1), 202);
    assert_eq!(value, 100, "the macro's own `value` didn't touch ours");
    assert_eq!(hygiene_demo(), (42, 32));
}

#[test]
fn my_assert_eq_passes_and_fails_with_source_text() {
    sut::my_assert_eq!(1 + 1, 2);
    let err = std::panic::catch_unwind(|| sut::my_assert_eq!(1 + 2, 4)).unwrap_err();
    let msg = err.downcast_ref::<String>().cloned().unwrap_or_default();
    assert!(
        msg.starts_with("assertion failed: `1 + 2 == 4` (left: 3, right: 4) at "),
        "got: {msg}"
    );
    assert!(
        msg.contains("ex5.rs"),
        "file!() must name the caller's file: {msg}"
    );
}

#[test]
fn traced_records_and_returns() {
    take_trace_log();
    let v = sut::traced!(2 + 3);
    assert_eq!(v, 5);
    let log = take_trace_log();
    assert_eq!(log.len(), 1);
    assert!(log[0].ends_with(": 2 + 3 = 5"), "got: {}", log[0]);
    assert!(log[0].contains("ex5.rs:"), "got: {}", log[0]);
}
