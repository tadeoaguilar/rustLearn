// 09-testing -- YOUR WORKSPACE.
//
// The output below looks right. It isn't -- the library has six bugs, and
// none of them shows up here. That's the point of this module: write tests.
//
//     cargo test -p m09-testing                          # your tests
//     cargo test -p m09-testing-tests --features mine    # the answer key
//     cargo bench -p m09-testing                         # your benchmarks

use m09_testing::*;

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
    // Uncomment when you have done the bowling bonus:
    // println!("bowling: perfect game = {}", bowling::score(&[10; 12]));
}
