// 05-error-handling -- YOUR WORKSPACE.
//
// This runner is ready to use: it calls the functions in src/ that you
// fill in. Until you implement a part it stops with "not yet implemented".
//
//     cargo run -p m05-error-handling -- 8       # one exercise
//     cargo run -p m05-error-handling -- all     # everything

use m05_error_handling::*;

fn main() {
    let parts: [(&str, &str, fn()); 10] = [
        ("1", "Panic basics", ex01_panic::run),
        ("2", "Result basics", ex02_result::run),
        ("3", "The ? operator", ex03_question_mark::run),
        ("4", "Custom error types", ex04_custom_errors::run),
        (
            "5",
            "Converting between error types",
            ex05_converting_errors::run,
        ),
        ("6", "Option to Result", ex06_option_to_result::run),
        ("7", "Error recovery", ex07_recovery::run),
        ("8", "Config file parser", ex08_config_parser::run),
        ("9", "thiserror (and anyhow)", ex09_thiserror::run),
        ("bonus", "Result combinators", bonus_combinators::run),
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
            println!("05-error-handling -- your workspace\n");
            for (key, title, _) in parts {
                println!("  cargo run -p m05-error-handling -- {key:<6} {title}");
            }
            println!("  cargo run -p m05-error-handling -- all    Everything");
        }
    }
}
