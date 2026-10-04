//! Exercise 3: Documentation Explorer.
//!
//! "Find 3 different ways to create a String" -- here are five, all of them
//! found on the `String` page of `cargo doc --open` / doc.rust-lang.org/std.

/// Returns the same text built five different ways.
pub fn strings_five_ways() -> [String; 5] {
    [
        // 1. From a string literal (a &'static str) through the From trait.
        String::from("hello"),
        // 2. `to_string` comes from the Display trait -- works for anything printable.
        "hello".to_string(),
        // 3. `to_owned` turns a borrowed &str into an owned String.
        "hello".to_owned(),
        // 4. The format! macro, same syntax as println!.
        format!("{}{}", "hel", "lo"),
        // 5. Start empty and grow it.
        {
            let mut s = String::with_capacity(5);
            s.push_str("hel");
            s.push('l');
            s.push('o');
            s
        },
    ]
}

/// `String` owns its bytes; `&str` borrows someone else's. A function that
/// only *reads* text should take `&str` -- callers can pass both a `&String`
/// (it derefs to `&str`) and a literal.
pub fn shout(text: &str) -> String {
    text.to_uppercase()
}
