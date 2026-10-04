use crate::sut;
use crate::sut::ex06_derive::*;

#[test]
fn describes_structs() {
    assert_eq!(Point::describe(), "struct Point { x: i32, y: i32 }");
    assert_eq!(Point::field_names(), &["x", "y"]);
    assert_eq!(Meters::describe(), "struct Meters(f64)");
    assert_eq!(Marker::describe(), "struct Marker");
}

#[test]
fn describes_enums() {
    assert_eq!(
        Shape::describe(),
        "enum Shape { Circle(f64), Square { side: f64 }, Empty }"
    );
}

#[test]
fn generic_and_complex_types() {
    assert_eq!(
        Wrapper::<i32>::describe(),
        "struct Wrapper { inner: T, history: Vec<T> }"
    );
    assert_eq!(
        Profile::describe(),
        "struct Profile { name: String, tags: Vec<String>, age: Option<u8>, motto: &'static str }"
    );
}

#[test]
fn derive_works_in_another_crate() {
    #[derive(sut::Describe)]
    #[allow(dead_code)]
    struct Local {
        id: u64,
    }
    assert_eq!(Local::describe(), "struct Local { id: u64 }");
}
