// Reference solution for 09-testing.
//
// This module is about the tests, not the program. The binary just shows the
// library working; run the test suite to see the point:
//
//     cargo test -p m09-testing-solution            # unit + integration + property + doc tests
//     cargo bench -p m09-testing-solution           # criterion benchmarks

use m09_testing_solution::*;

fn main() {
    println!(
        "stats:  mean {:?}, median {:?}, mode {:?}, std_dev {:?}",
        stats::mean(&[1.0, 2.0, 3.0, 4.0]),
        stats::median(&[1.0, 2.0, 3.0]),
        stats::mode(&[1, 2, 2, 3]),
        stats::std_dev(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0])
    );

    let mut a = account::Account::new("Ann", 100);
    let mut b = account::Account::new("Bob", 0);
    println!(
        "bank:   transfer -> {:?}; balances {} / {}",
        account::transfer(&mut a, &mut b, 30),
        a.balance(),
        b.balance()
    );

    println!("slug:   {:?}", slug::slugify("Hello World 2024"));
    println!(
        "roman:  2024 -> {:?}, 14 -> {:?}, \"MMXXIV\" -> {:?}",
        roman::to_roman(2024),
        roman::to_roman(14),
        roman::from_roman("MMXXIV")
    );

    let advisor = weather::Advisor::new(weather::DemoWeather);
    for city in ["Oslo", "London", "Madrid", "Atlantis", "Gotham"] {
        println!("advice: {}", advisor.sentence(city));
    }
    println!("bowling: perfect game = {}", bowling::score(&[10; 12]));
}
