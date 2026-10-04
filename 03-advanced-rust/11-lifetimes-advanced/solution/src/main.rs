// Reference solution for 11-lifetimes-advanced.
//
//     cargo run -p m11-lifetimes-advanced-solution -- 3       # one exercise
//     cargo run -p m11-lifetimes-advanced-solution -- all     # everything

use m11_lifetimes_advanced_solution::*;

fn main() {
    let parts: [(&str, &str, fn()); 8] = [
        ("1", "Annotations and elision", ex01_annotations::run),
        ("2", "Structs that borrow", ex02_structs::run),
        ("3", "Zero-copy HTTP parser", ex03_parser::run),
        ("4", "Iterators that borrow", ex04_iterators::run),
        ("5", "'static and lifetime bounds", ex05_static::run),
        ("6", "Higher-ranked trait bounds", ex06_hrtb::run),
        ("7", "Fixing lifetime errors", ex07_fix_errors::run),
        ("bonus", "StrSplit with two lifetimes", bonus_str_split::run),
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
            println!("11-lifetimes-advanced -- reference solution\n");
            for (key, title, _) in parts {
                println!("  cargo run -p m11-lifetimes-advanced-solution -- {key:<6} {title}");
            }
            println!("  cargo run -p m11-lifetimes-advanced-solution -- all    Everything");
        }
    }
}
