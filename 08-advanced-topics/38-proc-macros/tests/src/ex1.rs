//! Exercise 1: #[derive(ToJson)] on real types.

use crate::sut::ToJson;
use crate::sut::json::ToJson as _;

#[derive(ToJson)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(ToJson)]
struct User {
    id: u64,
    #[json(rename = "displayName")]
    name: String,
    email: Option<String>,
    tags: Vec<String>,
    location: Point,
    #[json(skip)]
    #[allow(dead_code)]
    password: String,
}

#[derive(ToJson)]
enum Shape {
    Empty,
    #[json(rename = "dot")]
    Dot,
    Circle {
        r: f64,
    },
    Line(Point, Point),
}

#[derive(ToJson)]
struct Wrapper<T> {
    value: T,
    count: usize,
}

#[derive(ToJson)]
struct Pair(i32, String);

#[derive(ToJson)]
struct Nothing;

#[test]
fn ex1_structs() {
    assert_eq!(Point { x: 1, y: -2 }.to_json(), r#"{"x":1,"y":-2}"#);
    let user = User {
        id: 7,
        name: "Ana \"A\"".into(),
        email: None,
        tags: vec!["a".into(), "b".into()],
        location: Point { x: 0, y: 0 },
        password: "hunter2".into(),
    };
    assert_eq!(
        user.to_json(),
        r#"{"id":7,"displayName":"Ana \"A\"","email":null,"tags":["a","b"],"location":{"x":0,"y":0}}"#
    );
    assert!(!user.to_json().contains("hunter2"), "skipped");
}

#[test]
fn ex1_enums_generics_tuples() {
    assert_eq!(Shape::Empty.to_json(), r#""Empty""#);
    assert_eq!(Shape::Dot.to_json(), r#""dot""#);
    assert_eq!(
        Shape::Circle { r: 1.5 }.to_json(),
        r#"{"Circle":{"r":1.5}}"#
    );
    assert_eq!(
        Shape::Line(Point { x: 0, y: 0 }, Point { x: 1, y: 1 }).to_json(),
        r#"{"Line":[{"x":0,"y":0},{"x":1,"y":1}]}"#
    );
    assert_eq!(
        Wrapper {
            value: vec![true, false],
            count: 2
        }
        .to_json(),
        r#"{"value":[true,false],"count":2}"#
    );
    assert_eq!(
        Wrapper {
            value: Point { x: 3, y: 4 },
            count: 1
        }
        .to_json(),
        r#"{"value":{"x":3,"y":4},"count":1}"#
    );
    assert_eq!(Pair(1, "one".into()).to_json(), r#"[1,"one"]"#);
    assert_eq!(Nothing.to_json(), "null");
}
