// 03-ownership-borrowing -- YOUR WORKSPACE.
//
// This runner is ready to use: it calls the functions in src/ that you
// fill in. Until you implement a part it stops with "not yet implemented".
//
//     cargo run -p m03-ownership-borrowing -- 7       # one exercise
//     cargo run -p m03-ownership-borrowing -- all     # everything

use m03_ownership_borrowing::*;

fn main() {
    let parts: [(&str, &str, fn()); 9] = [
        ("1", "Understanding ownership", ex01_ownership::run),
        ("2", "References and borrowing", ex02_references::run),
        ("3", "String vs &str", ex03_strings::run),
        ("4", "Dangling references", ex04_dangling::run),
        ("5", "Clone vs Copy", ex05_clone_copy::run),
        ("6", "Mutable vs immutable borrow", ex06_borrow_rules::run),
        ("7", "Text buffer", ex07_text_buffer::run),
        ("8", "Ownership in collections", ex08_collections::run),
        ("bonus", "SimpleRc", bonus_simple_rc::run),
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
            println!("03-ownership-borrowing -- your workspace\n");
            for (key, title, _) in parts {
                println!("  cargo run -p m03-ownership-borrowing -- {key:<6} {title}");
            }
            println!("  cargo run -p m03-ownership-borrowing -- all    Everything");
        }
    }
}
