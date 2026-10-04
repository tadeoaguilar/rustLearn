// 08-modules-crates -- YOUR WORKSPACE.
//
// This runner is ready to use: it calls the library you build in src/.
// Until you implement a part it stops with "not yet implemented".
//
//     cargo run -p m08-modules-crates -- all
//     cargo run -p m08-modules-crates --all-features -- all
//     cargo run -p m08-modules-crates --no-default-features -- all
//
// The output changes with the features you enable -- that's Exercise 4.

use m08_modules_crates::prelude::*;
use m08_modules_crates::sample_catalog;

fn main() {
    let choice = std::env::args().nth(1).unwrap_or_default();
    let parts: [(&str, &str, fn()); 5] = [
        ("1", "Modules: using the catalog and cart", part1),
        ("2", "Visibility: validated constructors", part2),
        ("4", "Features compiled into this binary", part4),
        ("5", "The money crate", part5),
        ("bonus", "Inventory report", bonus),
    ];
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
            println!("08-modules-crates -- your workspace\n");
            for (key, title, _) in parts {
                println!("  cargo run -p m08-modules-crates -- {key:<6} {title}");
            }
            println!("  cargo run -p m08-modules-crates -- all    Everything");
            println!("\nExercises 3 and 6 are about the API and docs: see src/prelude.rs and");
            println!("  cargo doc -p m08-modules-crates --all-features --open");
        }
    }
}

fn part1() {
    let catalog = sample_catalog();
    for book in catalog.iter() {
        println!(
            "{}  {:<32} {:<14} {}",
            book.isbn(),
            book.title(),
            book.author(),
            book.price()
        );
    }
    let found: Vec<&str> = catalog
        .search("rust in")
        .iter()
        .map(|b| b.title())
        .collect();
    println!("search(\"rust in\") -> {found:?}");

    let mut cart = Cart::new();
    let first = catalog.iter().next().unwrap().isbn().clone();
    cart.add(first.clone(), 1);
    cart.add(first, 2);
    println!(
        "cart: {} copies, total {:?}",
        cart.item_count(),
        cart.total(&catalog).map(|m| m.to_string())
    );
}

fn part2() {
    for s in [
        "978-0-306-40615-7",
        "978-0-306-40615-8",
        "12345",
        "978-0-306-4061X-7",
    ] {
        match Isbn::parse(s) {
            Ok(isbn) => println!("{s:<20} -> ok: {isbn}"),
            Err(e) => println!("{s:<20} -> {e}"),
        }
    }
    let isbn = Isbn::parse("9780306406157").unwrap();
    println!(
        "{:?}",
        Book::new(isbn.clone(), "  ", "Nobody", Money::ZERO).map_err(|e| e.to_string())
    );
    println!(
        "{:?}",
        Book::new(isbn, "Free Book", "Nobody", Money::from_cents(-1)).map_err(|e| e.to_string())
    );
}

fn part4() {
    println!("discounts: {}", cfg!(feature = "discounts"));
    println!("json:      {}", cfg!(feature = "json"));
    println!("report:    {}", cfg!(feature = "report"));

    #[cfg(feature = "discounts")]
    {
        let catalog = sample_catalog();
        let mut cart = Cart::new();
        for book in catalog.iter() {
            cart.add(book.isbn().clone(), 2);
        }
        let total = cart.total(&catalog).unwrap();
        for d in [
            Discount::Percent(10),
            Discount::Fixed(Money::new(20, 0)),
            Discount::Bulk {
                min_items: 10,
                percent: 25,
            },
        ] {
            println!(
                "{total} with {d:?} = {}",
                cart.total_with_discount(&catalog, &d).unwrap()
            );
        }
    }
    #[cfg(feature = "json")]
    println!("{}", sample_catalog().to_json());
}

fn part5() {
    let price: Money = "$12.34".parse().unwrap();
    println!(
        "parsed {price}, x3 = {}, minus $20 = {}",
        price * 3,
        price - Money::new(20, 0)
    );
    println!(
        "\"12.3\" -> {:?}",
        "12.3".parse::<Money>().map(|m| m.to_string())
    );
    println!(
        "\"abc\"  -> {:?}",
        "abc".parse::<Money>().map_err(|e| e.to_string())
    );
}

fn bonus() {
    #[cfg(feature = "report")]
    println!("{}", sample_catalog().report());
    #[cfg(not(feature = "report"))]
    println!(
        "Built without the `report` feature. Try:\n  cargo run -p m08-modules-crates --features report -- bonus"
    );
}
