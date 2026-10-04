// Reference solution for 04-structs-enums.
//
//     cargo run -p m04-structs-enums-solution -- 8       # one exercise
//     cargo run -p m04-structs-enums-solution -- all     # everything

use m04_structs_enums_solution::*;

fn main() {
    let parts: [(&str, &str, fn()); 10] = [
        ("1", "Basic structs", ex01_structs::run),
        ("2", "Tuple and unit structs", ex02_tuple_unit_structs::run),
        ("3", "Methods and associated functions", ex03_methods::run),
        ("4", "Basic enums", ex04_enums::run),
        ("5", "Option<T>", ex05_option::run),
        ("6", "Pattern matching", ex06_patterns::run),
        ("7", "Result<T, E>", ex07_result::run),
        ("8", "Game state machine", ex08_game_state::run),
        ("9", "Shape calculator", ex09_shapes::run),
        ("bonus", "JSON-like data structure", bonus_json::run),
    ];

    let choice = std::env::args().nth(1).unwrap_or_default();
    match parts.iter().find(|(key, _, _)| *key == choice) {
        Some((key, title, run)) => {
            println!("=== EXERCISE {key}: {title} ===\n");
            run();
        }
        None if choice == "all" => {
            for (key, title, run) in parts {
                println!("\n=== EXERCISE {key}: {title} ===\n");
                run();
            }
        }
        None => {
            println!("04-structs-enums -- reference solution\n");
            for (key, title, _) in parts {
                println!("  cargo run -p m04-structs-enums-solution -- {key:<6} {title}");
            }
            println!("  cargo run -p m04-structs-enums-solution -- all    Everything");
        }
    }
}
