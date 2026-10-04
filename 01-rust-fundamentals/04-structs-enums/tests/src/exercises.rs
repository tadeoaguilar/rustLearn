use crate::sut::*;

const EPS: f64 = 1e-9;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < EPS
}

// ---- Exercise 1 --------------------------------------------------------------

#[test]
fn ex1_build_and_modify_user() {
    let user = ex01_structs::build_user("a@b.c".into(), "ab".into());
    assert!(user.active);
    assert_eq!(user.sign_in_count, 1);
    let user = ex01_structs::change_email(user, "new@b.c");
    assert_eq!(user.email, "new@b.c");
}

#[test]
fn ex1_struct_update_moves_only_some_fields() {
    let user1 = ex01_structs::build_user("user@example.com".into(), "user123".into());
    let (user2, old_email, active) = ex01_structs::struct_update(user1);
    assert_eq!(user2.email, "another@example.com");
    assert_eq!(user2.username, "user123", "taken from user1");
    assert_eq!(old_email, "user@example.com", "user1.email was never moved");
    assert!(active);
}

// ---- Exercise 2 --------------------------------------------------------------

#[test]
fn ex2_distance_from_origin() {
    use ex02_tuple_unit_structs::*;
    assert_eq!(distance_from_origin(Point2D(3.0, 4.0)), 5.0);
    assert_eq!(distance_from_origin(Point2D(0.0, 0.0)), 0.0);
    assert_eq!(distance_from_origin_3d(Point3D(1.0, 2.0, 2.0)), 3.0);
}

#[test]
fn ex2_unit_struct_is_zero_sized() {
    assert_eq!(
        std::mem::size_of::<ex02_tuple_unit_structs::AlwaysEqual>(),
        0
    );
}

// ---- Exercise 3 --------------------------------------------------------------

#[test]
fn ex3_rectangle_scenario_from_the_exercise() {
    use ex03_methods::Rectangle;
    let rect1 = Rectangle::new(30, 50);
    let rect2 = Rectangle::square(25);
    assert_eq!(rect1.area(), 1500);
    assert!(rect1.can_hold(&rect2));
    assert!(!rect2.can_hold(&rect1));

    let mut rect3 = Rectangle::new(10, 20);
    rect3.scale(2);
    // NOT 400 as exercises.md says: 10x20 scaled by 2 is 20x40.
    assert_eq!(rect3.area(), 800);

    let square = rect3.to_square();
    assert_eq!(square, Rectangle::square(40));
    assert!(square.is_square());
}

#[test]
fn ex3_can_hold_is_strict() {
    use ex03_methods::Rectangle;
    assert!(!Rectangle::square(10).can_hold(&Rectangle::square(10)));
}

// ---- Exercise 4 --------------------------------------------------------------

#[test]
fn ex4_directions() {
    use ex04_enums::{Direction::*, move_player, walk};
    assert_eq!(move_player(North), (0, 1));
    assert_eq!(move_player(South), (0, -1));
    assert_eq!(move_player(East), (1, 0));
    assert_eq!(move_player(West), (-1, 0));
    assert_eq!(walk(&[North, North, East, South, West, West]), (-1, 1));
}

#[test]
fn ex4_messages() {
    use ex04_enums::Message;
    assert_eq!(Message::Quit.describe(), "Quit");
    assert_eq!(
        Message::Move { x: 10, y: 20 }.describe(),
        "Move to (10, 20)"
    );
    assert_eq!(Message::Write("hello".into()).describe(), "Write: hello");
    assert_eq!(
        Message::ChangeColor(255, 0, 0).describe(),
        "Change color to (255, 0, 0)"
    );
}

// ---- Exercise 5 --------------------------------------------------------------

#[test]
fn ex5_divide() {
    assert_eq!(ex05_option::divide(10.0, 2.0), Some(5.0));
    assert_eq!(ex05_option::divide(1.0, 0.0), None);
}

#[test]
fn ex5_option_methods() {
    // [unwrap, unwrap_or, unwrap_or_else, map, and_then, filter]
    assert_eq!(
        ex05_option::option_methods(),
        [Some(5), Some(0), Some(42), Some(10), Some(10), None]
    );
}

#[test]
fn ex5_find() {
    use ex05_option::*;
    let numbers = vec![1, 3, 5, 8, 9, 10];
    assert_eq!(find_first_even(&numbers), Some(8));
    assert_eq!(find_first_even(&[1, 3]), None);
    assert_eq!(find_position(&numbers, &5), Some(2));
    assert_eq!(find_position(&numbers, &100), None);
    assert_eq!(find_position(&["a", "b"], &"b"), Some(1), "generic over T");
    assert_eq!(find_position_loop(&numbers, &9), Some(4));
}

// ---- Exercise 6 --------------------------------------------------------------

#[test]
fn ex6_describe_number() {
    use ex06_patterns::describe_number as d;
    assert_eq!(d(0), "zero");
    assert_eq!(d(2), "one or two");
    assert_eq!(d(3), "three through nine");
    assert_eq!(d(9), "three through nine");
    assert_eq!(d(10), "ten");
    assert_eq!(d(-1), "something else");
}

#[test]
fn ex6_destructuring() {
    use ex06_patterns::{Point, locate};
    assert_eq!(locate(Point { x: 0, y: 7 }), "On y-axis at 7");
    assert_eq!(locate(Point { x: 3, y: 0 }), "On x-axis at 3");
    assert_eq!(locate(Point { x: 2, y: 5 }), "At (2, 5)");
}

#[test]
fn ex6_guards() {
    use ex06_patterns::{Temperature::*, describe_temperature as d};
    assert_eq!(d(Celsius(-1)), "freezing in Celsius");
    assert_eq!(d(Celsius(31)), "hot in Celsius");
    assert_eq!(d(Celsius(20)), "moderate temperature");
    assert_eq!(d(Fahrenheit(0)), "freezing in Fahrenheit");
    assert_eq!(d(Fahrenheit(90)), "hot in Fahrenheit");
    assert_eq!(d(Fahrenheit(70)), "moderate temperature");
}

#[test]
fn ex6_if_let_while_let_let_else() {
    use ex06_patterns::*;
    assert!(is_three(Some(3)));
    assert!(!is_three(Some(4)));
    assert!(!is_three(None));
    assert_eq!(drain_stack(vec![1, 2, 3]), vec![3, 2, 1]);
    assert_eq!(describe_pair("salt, pepper"), "salt and pepper");
    assert_eq!(describe_pair("salt"), "\"salt\" has no comma");
    assert_eq!(parse_pair("3, 4"), Some((3, 4)));
    assert_eq!(parse_pair("3 4"), None);
    assert_eq!(parse_pair("x,4"), None);
}

// ---- Exercise 7 --------------------------------------------------------------

#[test]
fn ex7_parse_int() {
    assert_eq!(ex07_result::parse_int("42"), Ok(42));
    assert!(ex07_result::parse_int("forty-two").is_err());
}

#[test]
fn ex7_math_errors() {
    use ex07_result::*;
    assert_eq!(divide(10.0, 2.0), Ok(5.0));
    assert_eq!(divide(1.0, 0.0), Err(MathError::DivisionByZero));
    assert_eq!(sqrt(9.0), Ok(3.0));
    assert_eq!(sqrt(-1.0), Err(MathError::NegativeSquareRoot));
    assert_eq!(sqrt_of_quotient(8.0, 2.0), Ok(2.0));
    assert_eq!(sqrt_of_quotient(8.0, 0.0), Err(MathError::DivisionByZero));
    assert_eq!(
        sqrt_of_quotient(-8.0, 2.0),
        Err(MathError::NegativeSquareRoot)
    );
    assert_eq!(
        MathError::DivisionByZero.to_string(),
        "cannot divide by zero"
    );
}

// ---- Exercise 8 --------------------------------------------------------------

#[test]
fn ex8_full_game_from_the_exercise() {
    use ex08_game_state::{GameState, LEVEL_BONUS};
    let mut game = GameState::new();
    assert_eq!(game, GameState::Menu);

    game.start_game();
    assert_eq!(
        game,
        GameState::Playing {
            level: 1,
            score: 0,
            lives: 3
        }
    );

    game.add_score(100);
    game.pause();
    assert_eq!(
        game,
        GameState::Paused {
            level: 1,
            score: 100,
            lives: 3
        }
    );

    game.add_score(1_000);
    assert_eq!(
        game,
        GameState::Paused {
            level: 1,
            score: 100,
            lives: 3
        },
        "no scoring while paused"
    );

    game.resume();
    game.add_score(50);
    assert_eq!(
        game,
        GameState::Playing {
            level: 1,
            score: 150,
            lives: 3
        }
    );

    game.next_level();
    assert_eq!(
        game,
        GameState::Playing {
            level: 2,
            score: 150 + LEVEL_BONUS,
            lives: 3
        }
    );

    game.lose_life();
    game.lose_life();
    assert!(!game.is_game_over());
    game.lose_life();
    assert!(game.is_game_over());
    assert_eq!(
        game,
        GameState::GameOver {
            final_score: 150 + LEVEL_BONUS
        }
    );
    assert!(game.display().contains("GAME OVER"));
}

#[test]
fn ex8_invalid_transitions_are_ignored() {
    use ex08_game_state::GameState;
    let mut game = GameState::new();
    game.pause();
    game.resume();
    game.add_score(10);
    game.lose_life();
    game.next_level();
    assert_eq!(game, GameState::Menu);
}

// ---- Exercise 9 --------------------------------------------------------------

#[test]
fn ex9_areas() {
    use ex09_shapes::{Shape, total_area};
    use std::f64::consts::PI;
    let shapes = vec![
        Shape::Circle { radius: 5.0 },
        Shape::Rectangle {
            width: 10.0,
            height: 20.0,
        },
        Shape::Triangle {
            base: 6.0,
            height: 8.0,
        },
    ];
    assert!(close(shapes[0].area(), 25.0 * PI));
    assert_eq!(shapes[1].area(), 200.0);
    assert_eq!(shapes[2].area(), 24.0);
    assert!(close(total_area(&shapes), 25.0 * PI + 224.0));
}

#[test]
fn ex9_perimeters() {
    use ex09_shapes::Shape;
    use std::f64::consts::PI;
    assert!(close(Shape::Circle { radius: 1.0 }.perimeter(), 2.0 * PI));
    assert_eq!(
        Shape::Rectangle {
            width: 10.0,
            height: 20.0
        }
        .perimeter(),
        60.0
    );
    // isosceles: base 6, height 4 -> two sides of 5
    assert_eq!(
        Shape::Triangle {
            base: 6.0,
            height: 4.0
        }
        .perimeter(),
        16.0
    );
}

#[test]
fn ex9_scale() {
    use ex09_shapes::Shape;
    let mut r = Shape::Rectangle {
        width: 2.0,
        height: 3.0,
    };
    r.scale(2.0);
    assert_eq!(
        r,
        Shape::Rectangle {
            width: 4.0,
            height: 6.0
        }
    );
    let mut t = Shape::Triangle {
        base: 6.0,
        height: 8.0,
    };
    let before = t.area();
    t.scale(3.0);
    assert!(close(t.area(), before * 9.0), "area scales with factor²");
    let mut c = Shape::Circle { radius: 1.0 };
    c.scale(0.5);
    assert_eq!(c, Shape::Circle { radius: 0.5 });
}

// ---- Bonus -------------------------------------------------------------------

#[test]
fn bonus_accessors() {
    use bonus_json::{JsonValue, sample_user};
    let json = sample_user();
    assert_eq!(json.get("name").and_then(JsonValue::as_str), Some("Alice"));
    assert_eq!(json.get("age").and_then(|v| v.as_number()), Some(30.0));
    assert_eq!(json.get("active").and_then(JsonValue::as_bool), Some(true));
    assert!(json.get("manager").is_some_and(JsonValue::is_null));
    assert_eq!(json.get("missing"), None);
    assert_eq!(
        json.get("name").and_then(JsonValue::as_number),
        None,
        "wrong type -> None"
    );
    let tags = json.get("tags").unwrap();
    assert_eq!(tags.get_index(0).and_then(JsonValue::as_str), Some("admin"));
    assert_eq!(tags.get_index(9), None);
    assert_eq!(JsonValue::Null.get("x"), None);
}

#[test]
fn bonus_pretty_printing_is_deterministic() {
    use bonus_json::sample_user;
    let expected = r#"{
  "active": true,
  "age": 30,
  "manager": null,
  "name": "Alice",
  "tags": [
    "admin",
    "dev"
  ]
}"#;
    assert_eq!(sample_user().to_pretty_string(2), expected);
}

#[test]
fn bonus_strings_are_escaped() {
    use bonus_json::JsonValue;
    let v = JsonValue::String("say \"hi\"\n".into());
    assert_eq!(v.to_pretty_string(2), r#""say \"hi\"\n""#);
    assert_eq!(JsonValue::Array(vec![]).to_pretty_string(2), "[]");
}
