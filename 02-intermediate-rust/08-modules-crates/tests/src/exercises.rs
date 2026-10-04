use crate::money::Money as RawMoney;
use crate::sut;

fn isbn(s: &str) -> sut::catalog::Isbn {
    sut::catalog::Isbn::parse(s).expect("valid ISBN in test")
}

// ---- Exercise 1: the module paths exist -------------------------------------

#[test]
fn ex1_module_paths_resolve() {
    // If this compiles, the module tree has the right shape.
    use sut::cart::Cart;
    use sut::catalog::{Book, Catalog, Isbn};
    use sut::pricing::Discount;
    let _ = (Cart::new(), Catalog::new(), Discount::Percent(1));
    let _ = Isbn::parse;
    let _ = Book::new;
}

#[test]
fn ex1_sample_catalog_and_search() {
    let catalog = sut::sample_catalog();
    assert_eq!(catalog.len(), 3);
    let titles: Vec<&str> = catalog
        .search("RUST IN")
        .iter()
        .map(|b| b.title())
        .collect();
    assert_eq!(titles, vec!["Rust in Action"]);
    assert_eq!(
        catalog.search("klabnik").len(),
        1,
        "search covers authors too"
    );
    assert!(catalog.search("python").is_empty());
}

#[test]
fn ex1_cart_merges_lines_and_totals() {
    let catalog = sut::sample_catalog();
    let mut cart = sut::cart::Cart::new();
    cart.add(isbn("978-1-4920-5259-3"), 1);
    cart.add(isbn("9781492052593"), 2); // same book, no hyphens
    cart.add(isbn("978-1-7185-0044-0"), 0); // ignored
    assert_eq!(cart.lines().len(), 1);
    assert_eq!(cart.item_count(), 3);
    assert_eq!(cart.total(&catalog).unwrap().to_string(), "$149.97");
    assert_eq!(cart.remove(&isbn("978-1-4920-5259-3")), 3);
    assert_eq!(cart.total(&catalog).unwrap().to_string(), "$0.00");
}

#[test]
fn ex1_unknown_books_are_an_error() {
    let mut cart = sut::cart::Cart::new();
    let unknown = isbn("978-0-306-40615-7");
    cart.add(unknown.clone(), 1);
    assert_eq!(
        cart.total(&sut::sample_catalog()),
        Err(sut::cart::CartError::UnknownBook(unknown))
    );
}

// ---- Exercise 2: validation and encapsulation -------------------------------

#[test]
fn ex2_isbn_validation() {
    use sut::catalog::{Isbn, IsbnError};
    assert!(Isbn::parse("978-0-306-40615-7").is_ok());
    assert_eq!(
        Isbn::parse("978-0-306-40615-7").unwrap().as_str(),
        "9780306406157"
    );
    assert_eq!(
        Isbn::parse("978-0-306-40615-8"),
        Err(IsbnError::BadChecksum)
    );
    assert_eq!(Isbn::parse("12345"), Err(IsbnError::WrongLength(5)));
    assert_eq!(
        Isbn::parse("978-0-306-4061X-7"),
        Err(IsbnError::InvalidCharacter('X'))
    );
    assert_eq!(
        Isbn::parse("978 0 306 40615 7").map(|i| i.to_string()),
        Ok("9780306406157".to_string())
    );
}

#[test]
fn ex2_book_constructor_enforces_invariants() {
    use sut::catalog::{Book, BookError};
    let i = isbn("9780306406157");
    let price = sut::Money::from_cents(1000);
    assert_eq!(
        Book::new(i.clone(), "   ", "A", price),
        Err(BookError::EmptyTitle)
    );
    let negative = sut::Money::from_cents(-1);
    assert_eq!(
        Book::new(i.clone(), "T", "A", negative),
        Err(BookError::NegativePrice(negative))
    );
    let book = Book::new(i.clone(), "  Title  ", " Author ", price).unwrap();
    assert_eq!(
        (book.title(), book.author(), book.price()),
        ("Title", "Author", price),
        "trimmed"
    );
    assert_eq!(book.isbn(), &i);
}

// ---- Exercise 3: re-exports and prelude -------------------------------------

#[test]
fn ex3_root_reexports_and_prelude() {
    // Same types, reachable three ways.
    let a: sut::Book = sut::sample_catalog().iter().next().unwrap().clone();
    let b: sut::catalog::Book = a.clone();
    {
        use sut::prelude::*;
        let c: Book = b.clone();
        let _: (Cart, Catalog, Discount, Money) = (
            Cart::new(),
            Catalog::new(),
            Discount::Percent(0),
            Money::ZERO,
        );
        assert_eq!(a, c);
    }
}

// ---- Exercise 4: features ---------------------------------------------------

#[test]
fn ex4_discounts() {
    use sut::{Discount, Money};
    let catalog = sut::sample_catalog();
    let mut cart = sut::Cart::new();
    for book in catalog.iter() {
        cart.add(book.isbn().clone(), 2);
    }
    let total = cart.total(&catalog).unwrap();
    assert_eq!(total, Money::from_cents(26994));
    let with = |d: Discount| cart.total_with_discount(&catalog, &d).unwrap();
    assert_eq!(
        with(Discount::Percent(10)),
        Money::from_cents(24295),
        "2699.4 rounds to 2699 off"
    );
    assert_eq!(
        with(Discount::Fixed(Money::new(20, 0))),
        Money::from_cents(24994)
    );
    assert_eq!(
        with(Discount::Fixed(Money::new(1_000, 0))),
        Money::ZERO,
        "never below zero"
    );
    assert_eq!(
        with(Discount::Bulk {
            min_items: 6,
            percent: 50
        }),
        Money::from_cents(13497)
    );
    assert_eq!(
        with(Discount::Bulk {
            min_items: 7,
            percent: 50
        }),
        total,
        "not enough items"
    );
    assert_eq!(
        with(Discount::Percent(150)),
        Money::ZERO,
        "percent is capped at 100"
    );
}

#[test]
fn ex4_json_export() {
    let json = sut::sample_catalog().to_json();
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    let books = parsed.as_array().expect("an array of books");
    assert_eq!(books.len(), 3);
    assert_eq!(books[0]["title"], "Programming Rust", "sorted by ISBN");
    assert_eq!(books[0]["isbn"], "9781492052593");
    assert_eq!(books[0]["price"]["cents"], 4999);
}

// ---- Exercise 5: the money crate --------------------------------------------

#[test]
fn ex5_money_display_and_parse() {
    assert_eq!(RawMoney::from_cents(1234).to_string(), "$12.34");
    assert_eq!(RawMoney::from_cents(-5).to_string(), "-$0.05");
    assert_eq!(RawMoney::from_cents(7).to_string(), "$0.07");
    assert_eq!(RawMoney::new(3, 5).cents(), 305);
    assert_eq!("$12.34".parse::<RawMoney>(), Ok(RawMoney::from_cents(1234)));
    assert_eq!("12".parse::<RawMoney>(), Ok(RawMoney::from_cents(1200)));
    assert_eq!(
        "12.3".parse::<RawMoney>(),
        Ok(RawMoney::from_cents(1230)),
        "one decimal means tens of cents"
    );
    assert_eq!("-$1.50".parse::<RawMoney>(), Ok(RawMoney::from_cents(-150)));
    assert!("abc".parse::<RawMoney>().is_err());
    assert!("1.234".parse::<RawMoney>().is_err());
    assert!("$".parse::<RawMoney>().is_err());
}

#[test]
fn ex5_money_arithmetic() {
    let m = RawMoney::from_cents;
    assert_eq!(m(250) * 3, m(750));
    assert_eq!(m(250) + m(1), m(251));
    assert_eq!(m(250) - m(300), m(-50));
    assert_eq!(vec![m(1), m(2), m(3)].into_iter().sum::<RawMoney>(), m(6));
    assert_eq!([m(1), m(2)].iter().sum::<RawMoney>(), m(3));
}

#[test]
fn ex5_bookstore_reexports_the_money_type() {
    // The bookstore's Money *is* the money crate's Money, not a copy of it.
    let from_crate: RawMoney = RawMoney::from_cents(1);
    let from_bookstore: sut::Money = from_crate;
    assert_eq!(from_bookstore, sut::Money::from_cents(1));
}

// ---- Bonus ------------------------------------------------------------------

#[test]
fn bonus_report() {
    let report = sut::sample_catalog().report();
    assert_eq!(report.books, 3);
    assert_eq!(report.authors, 3);
    assert_eq!(report.total_value.to_string(), "$134.97");
    assert_eq!(
        report.to_string(),
        "3 books by 3 authors, total list value $134.97\nMost expensive: Programming Rust ($49.99)"
    );
    assert_eq!(
        sut::Catalog::new().report().to_string().lines().last(),
        Some("The catalog is empty")
    );
}
