// 13-macros -- YOUR WORKSPACE.
//
// Ready-made runner; until you implement a part it stops with "not yet implemented".
//
//     cargo run -p m13-macros -- 4       # one exercise
//     cargo run -p m13-macros -- all     # everything

use m13_macros::*;

fn main() {
    let parts: [(&str, &str, fn()); 8] = [
        ("1", "First macro_rules!", ex01_basics::run),
        ("2", "Generating items", ex02_items::run),
        ("3", "Compile-time validation", ex03_validation::run),
        ("4", "A state-machine DSL", ex04_dsl::run),
        ("5", "Hygiene and debugging", ex05_hygiene::run),
        ("6", "#[derive(Describe)]", ex06_derive::run),
        ("7", "#[derive(Builder)]", ex07_builder::run),
        ("bonus", "#[timed] attribute", bonus_attribute::run),
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
            println!("13-macros -- your workspace\n");
            for (key, title, _) in parts {
                println!("  cargo run -p m13-macros -- {key:<6} {title}");
            }
            println!("  cargo run -p m13-macros -- all    Everything");
        }
    }
}
