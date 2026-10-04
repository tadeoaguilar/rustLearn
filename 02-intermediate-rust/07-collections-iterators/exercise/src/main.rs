// 07-collections-iterators -- YOUR WORKSPACE.
//
// This runner is ready to use: it calls the functions in src/ that you
// fill in. Until you implement a part it stops with "not yet implemented".
//
//     cargo run -p m07-collections-iterators -- 5       # one exercise
//     cargo run -p m07-collections-iterators -- all     # everything

use m07_collections_iterators::*;

fn main() {
    let parts: [(&str, &str, fn()); 7] = [
        ("1", "Vector operations", ex01_vectors::run),
        ("2", "HashMap basics", ex02_hashmaps::run),
        ("3", "Iterator basics", ex03_iterator_basics::run),
        ("4", "Advanced iterators", ex04_advanced_iterators::run),
        ("5", "Custom iterators", ex05_custom_iterators::run),
        ("6", "Closures", ex06_closures::run),
        ("bonus", "Data pipeline", bonus_pipeline::run),
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
            println!("07-collections-iterators -- your workspace\n");
            for (key, title, _) in parts {
                println!("  cargo run -p m07-collections-iterators -- {key:<6} {title}");
            }
            println!("  cargo run -p m07-collections-iterators -- all    Everything");
        }
    }
}
