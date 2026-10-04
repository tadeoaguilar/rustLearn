// Reference solution for 02-basic-syntax.
//
//     cargo run -p m02-basic-syntax-solution -- 5        # one exercise
//     cargo run -p m02-basic-syntax-solution -- all      # every non-interactive one
//
// Exercises 7 and 9 are interactive; pipe input in to script them:
//
//     printf '5 + 3\n10 / 0\nquit\n' | cargo run -p m02-basic-syntax-solution -- 7

use m02_basic_syntax_solution::*;
use std::io;

fn main() -> io::Result<()> {
    let choice = std::env::args().nth(1).unwrap_or_default();
    match choice.as_str() {
        "1" => ex01_variables::run(),
        "2" => ex02_data_types::run(),
        "3" => ex03_functions::run(),
        "4" => ex04_if_else::run(),
        "5" => ex05_loops::run(),
        "6" => ex06_match::run(),
        "7" => ex07_calculator::run(io::stdin().lock(), io::stdout())?,
        "8" => ex08_fizzbuzz::run(),
        "9" => ex09_temperature::run(io::stdin().lock(), io::stdout())?,
        "bonus" => bonus_primes::run(),
        "all" => {
            let parts: [(&str, fn()); 8] = [
                ("1: Variables", ex01_variables::run),
                ("2: Data types", ex02_data_types::run),
                ("3: Functions", ex03_functions::run),
                ("4: If/else", ex04_if_else::run),
                ("5: Loops", ex05_loops::run),
                ("6: Match", ex06_match::run),
                ("8: FizzBuzz", ex08_fizzbuzz::run),
                ("Bonus: Primes", bonus_primes::run),
            ];
            for (title, part) in parts {
                println!("\n=== EXERCISE {title} ===\n");
                part();
            }
        }
        _ => menu(),
    }
    Ok(())
}

fn menu() {
    let me = "cargo run -p m02-basic-syntax-solution --";
    println!("02-basic-syntax -- reference solution\n");
    println!("  {me} 1       Variables and mutability");
    println!("  {me} 2       Data types");
    println!("  {me} 3       Functions");
    println!("  {me} 4       If/else");
    println!("  {me} 5       Loops");
    println!("  {me} 6       Match");
    println!("  {me} 7       Calculator (interactive)");
    println!("  {me} 8       FizzBuzz");
    println!("  {me} 9       Temperature converter (interactive)");
    println!("  {me} bonus   Prime numbers");
    println!("  {me} all     Every non-interactive exercise");
}
