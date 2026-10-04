// Reference solution for 06-traits-generics.
//
//     cargo run -p m06-traits-generics-solution -- 4       # one exercise
//     cargo run -p m06-traits-generics-solution -- all     # everything

use m06_traits_generics_solution::*;

fn main() {
    let parts: [(&str, &str, fn()); 8] = [
        ("1", "Basic traits", ex01_basic_traits::run),
        ("2", "Generic functions", ex02_generic_functions::run),
        ("3", "Trait bounds", ex03_trait_bounds::run),
        ("4", "Operator overloading", ex04_operator_overloading::run),
        ("5", "Associated types", ex05_associated_types::run),
        ("6", "Trait objects (plugins)", ex06_trait_objects::run),
        ("7", "Generic stack", ex07_generic_stack::run),
        ("bonus", "Shape system", bonus_shapes::run),
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
            println!("06-traits-generics -- reference solution\n");
            for (key, title, _) in parts {
                println!("  cargo run -p m06-traits-generics-solution -- {key:<6} {title}");
            }
            println!("  cargo run -p m06-traits-generics-solution -- all    Everything");
        }
    }
}
