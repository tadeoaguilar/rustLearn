// 01-getting-started -- YOUR WORKSPACE.
//
// This runner is ready to go; it calls the functions in src/ that you fill
// in. Until you implement a part, running it stops with "not yet implemented".
//
//     cargo run -p m01-getting-started -- 1      # Hello, Rust! (asks your name)
//     cargo run -p m01-getting-started -- 4      # the guessing game
//     cargo run -p m01-getting-started -- all    # every non-interactive part
//
// Interactive parts read from stdin, so you can also pipe answers in:
//
//     echo Ferris | cargo run -p m01-getting-started -- 1

use m01_getting_started::*;
use std::io;

fn main() -> io::Result<()> {
    let choice = std::env::args().nth(1).unwrap_or_default();

    match choice.as_str() {
        "1" => ex01_hello::run(io::stdin().lock(), io::stdout())?,
        "2" => part2(),
        "3" => part3(),
        "4" => {
            let secret = ex04_guessing_game::secret_number();
            ex04_guessing_game::play(secret, io::stdin().lock(), io::stdout())?;
        }
        "5" => part5(),
        "6" => part6(),
        "bonus" => println!("{}", bonus_build_script::build_info()),
        "all" => {
            part2();
            separator();
            part3();
            separator();
            part5();
            separator();
            part6();
            separator();
            println!("{}", bonus_build_script::build_info());
        }
        _ => menu(),
    }
    Ok(())
}

fn part2() {
    println!("=== EXERCISE 2: CARGO BASICS ===\n");
    // This binary is "a binary that uses the library".
    println!("my_math::add(2, 3) = {}", ex02_cargo_basics::add(2, 3));
    println!("Run the unit tests with: cargo test -p m01-getting-started");
}

fn part3() {
    println!("=== EXERCISE 3: DOCUMENTATION EXPLORER ===\n");
    for (i, s) in ex03_docs_explorer::strings_five_ways().iter().enumerate() {
        println!("way {}: {s:?}", i + 1);
    }
    let owned = String::from("works with String");
    println!("{}", ex03_docs_explorer::shout(&owned));
    println!("{}", ex03_docs_explorer::shout("and with &str"));
}

fn part5() {
    use ex05_project_setup::{Tool, command_for};
    println!("=== EXERCISE 5: PROJECT SETUP ===\n");
    for tool in [Tool::Fmt, Tool::Clippy, Tool::Test] {
        println!("{tool:?}: {}", command_for(tool));
    }
}

fn part6() {
    println!("=== EXERCISE 6: DEBUGGING ===\n");
    println!("fixed_sum() = {:?}", ex06_debugging::fixed_sum());
    println!("double(21) = {}", ex06_debugging::double(21));
    println!("dbg! output goes to stderr:");
    let total = ex06_debugging::sum_of_squares(&[1, 2, 3]);
    println!("sum_of_squares([1, 2, 3]) = {total}");
}

fn menu() {
    println!("01-getting-started -- your workspace\n");
    println!("  cargo run -p m01-getting-started -- 1       Hello, Rust! (interactive)");
    println!("  cargo run -p m01-getting-started -- 2       Cargo basics");
    println!("  cargo run -p m01-getting-started -- 3       Documentation explorer");
    println!("  cargo run -p m01-getting-started -- 4       Guessing game (interactive)");
    println!("  cargo run -p m01-getting-started -- 5       Project setup");
    println!("  cargo run -p m01-getting-started -- 6       Debugging");
    println!("  cargo run -p m01-getting-started -- bonus   Build script info");
    println!("  cargo run -p m01-getting-started -- all     Every non-interactive part");
}

fn separator() {
    println!("\n{}\n", "-".repeat(60));
}
