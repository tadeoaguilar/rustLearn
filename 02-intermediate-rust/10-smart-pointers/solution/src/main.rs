// Reference solution for 10-smart-pointers.
//
//     cargo run -p m10-smart-pointers-solution -- 4       # one exercise
//     cargo run -p m10-smart-pointers-solution -- all     # everything

use m10_smart_pointers_solution::*;

fn main() {
    let parts: [(&str, &str, fn()); 8] = [
        ("1", "Box and recursive types", ex01_box::run),
        ("2", "Linked list with Box", ex02_linked_list::run),
        ("3", "Deref and Drop", ex03_deref_drop::run),
        ("4", "Rc and Weak: a tree", ex04_rc_weak_tree::run),
        ("5", "RefCell and Cell", ex05_refcell::run),
        ("6", "Rc<RefCell> graph, and a leak", ex06_graph::run),
        ("7", "Arc<Mutex> shared cache", ex07_shared_cache::run),
        ("bonus", "Cow", bonus_cow::run),
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
            println!("10-smart-pointers -- reference solution\n");
            for (key, title, _) in parts {
                println!("  cargo run -p m10-smart-pointers-solution -- {key:<6} {title}");
            }
            println!("  cargo run -p m10-smart-pointers-solution -- all    Everything");
        }
    }
}
