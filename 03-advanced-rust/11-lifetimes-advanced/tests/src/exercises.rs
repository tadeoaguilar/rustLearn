//! Behaviour tests. The lifetime-signature checks are in signatures.rs.

use crate::sut::*;

// ---- Exercise 1 --------------------------------------------------------------

#[test]
fn ex1_functions() {
    use ex01_annotations::*;
    assert_eq!(first_word("hello world"), "hello");
    assert_eq!(first_word(""), "");
    assert_eq!(longest("ab", "abc"), "abc");
    assert_eq!(longest("abc", "xyz"), "abc", "ties go to the first");
    assert_eq!(longest_with_announcement("a", "bb", 42), "bb");
    assert_eq!(make_greeting("Ferris"), "Hello, Ferris!");
}

// ---- Exercise 2 --------------------------------------------------------------

#[test]
fn ex2_excerpt() {
    use ex02_structs::Excerpt;
    let novel = String::from("Call me Ishmael. Some years ago...");
    let e = Excerpt::first_sentence(&novel);
    assert_eq!(e.text(), "Call me Ishmael.");
    assert_eq!(e.word_count(), 3);
    assert_eq!(
        Excerpt::first_sentence("No terminator").text(),
        "No terminator"
    );
    assert_eq!(Excerpt::first_sentence("Wow! Really?").text(), "Wow!");
}

// ---- Exercise 3 --------------------------------------------------------------

#[test]
fn ex3_parses_a_get_request() {
    use ex03_parser::parse_request;
    let raw = "GET /search?q=rust&page=2 HTTP/1.1\r\nHost: example.com\r\nAccept: */*\r\n\r\n";
    let req = parse_request(raw).unwrap();
    assert_eq!(
        (req.method, req.path, req.version),
        ("GET", "/search?q=rust&page=2", "HTTP/1.1")
    );
    assert_eq!(req.header("host"), Some("example.com"));
    assert_eq!(req.header("ACCEPT"), Some("*/*"));
    assert_eq!(req.header("missing"), None);
    assert_eq!(req.query_params(), vec![("q", "rust"), ("page", "2")]);
    assert_eq!(req.body, "");
    assert_eq!(
        req.method.as_ptr(),
        raw.as_ptr(),
        "zero-copy: method points into the input"
    );
}

#[test]
fn ex3_body_lf_only_and_flags() {
    use ex03_parser::{SAMPLE, parse_request};
    let req = parse_request(SAMPLE).unwrap();
    assert_eq!(req.body, "hello body");
    assert_eq!(req.headers().len(), 2);
    let req = parse_request("GET /?debug&x=1 HTTP/1.0\nHost: a\n\nbody").unwrap();
    assert_eq!(req.body, "body", "bare LF line endings are accepted");
    assert_eq!(req.query_params(), vec![("debug", ""), ("x", "1")]);
    assert_eq!(
        parse_request("GET / HTTP/1.1\r\n\r\n")
            .unwrap()
            .query_params(),
        vec![]
    );
}

#[test]
fn ex3_errors_quote_the_input() {
    use ex03_parser::{ParseError, parse_request};
    assert_eq!(parse_request(""), Err(ParseError::Empty));
    assert_eq!(
        parse_request("GET\r\n\r\n"),
        Err(ParseError::BadRequestLine("GET"))
    );
    assert_eq!(
        parse_request("GET / FTP/1\r\n\r\n"),
        Err(ParseError::BadRequestLine("GET / FTP/1"))
    );
    assert_eq!(
        parse_request("GET / HTTP/1.1\r\nno colon\r\n\r\n"),
        Err(ParseError::BadHeader("no colon"))
    );
}

// ---- Exercise 4 --------------------------------------------------------------

#[test]
fn ex4_pairs() {
    use ex04_iterators::Pairs;
    assert_eq!(
        Pairs::new(&[1, 2, 3]).collect::<Vec<_>>(),
        vec![(&1, &2), (&2, &3)]
    );
    assert_eq!(Pairs::new(&[1]).count(), 0);
    assert_eq!(Pairs::<i32>::new(&[]).count(), 0);
}

#[test]
fn ex4_words() {
    use ex04_iterators::Words;
    assert_eq!(
        Words::new("  a  bb\tccc\n").collect::<Vec<_>>(),
        vec!["a", "bb", "ccc"]
    );
    assert_eq!(Words::new("   ").count(), 0);
}

#[test]
fn ex4_every_other_mut() {
    use ex04_iterators::EveryOtherMut;
    let mut v = vec![1, 2, 3, 4, 5];
    for x in EveryOtherMut::new(&mut v) {
        *x *= 10;
    }
    assert_eq!(v, vec![10, 2, 30, 4, 50]);

    let mut even = vec![1, 2, 3, 4];
    let refs: Vec<&mut i32> = EveryOtherMut::new(&mut even).collect(); // all alive at once
    assert_eq!(refs.len(), 2);
    let mut empty: Vec<i32> = vec![];
    assert_eq!(EveryOtherMut::new(&mut empty).count(), 0);
}

// ---- Exercise 5 --------------------------------------------------------------

#[test]
fn ex5_spawn_and_static() {
    use ex05_static::*;
    assert_eq!(spawn_and_join(String::from("x")), "from another thread: x");
    assert_eq!(spawn_and_join(7), "from another thread: 7");
    is_static(&String::from("owned data is 'static"));
    is_static(&"a literal");
}

#[test]
fn ex5_callbacks_can_borrow() {
    let prefix = String::from(">");
    let mut cb = ex05_static::Callbacks::new();
    cb.register(|e| format!("{prefix}{e}"));
    cb.register(str::to_uppercase);
    assert_eq!(cb.fire("go"), vec![">go", "GO"]);
}

#[test]
fn ex5_config_and_leak() {
    use ex05_static::*;
    let a: &'static Config = config();
    let b = config();
    assert!(
        std::ptr::eq(a, b),
        "initialised once, same instance every time"
    );
    assert_eq!(a.max_connections, 16);
    let s: &'static str = leak_str(String::from("forever"));
    assert_eq!(s, "forever");
}

// ---- Exercise 6 --------------------------------------------------------------

#[test]
fn ex6_apply_to_all() {
    use ex06_hrtb::apply_to_all;
    let names = vec!["  Ann ".to_string(), "bob@example.com".to_string()];
    assert_eq!(
        apply_to_all(&names, str::trim),
        vec!["Ann", "bob@example.com"]
    );
    assert_eq!(
        apply_to_all(&names, |s| s.split('@').next().unwrap_or(s)),
        vec!["  Ann ", "bob"]
    );
}

#[test]
fn ex6_slice_pipeline() {
    use ex06_hrtb::SlicePipeline;
    let p = SlicePipeline::new()
        .then(str::trim)
        .then(|s| s.strip_prefix("re: ").unwrap_or(s));
    assert_eq!(p.run("  re: hello "), "hello");
    assert_eq!(SlicePipeline::new().run("same"), "same");
    let input = String::from("  abc  ");
    let out = p.run(&input);
    assert!(
        input.contains(out) && out.as_ptr() > input.as_ptr(),
        "the output is a slice of the input"
    );
}

// ---- Exercise 7 --------------------------------------------------------------

#[test]
fn ex7a_and_7b() {
    use ex07_fix_errors::*;
    let text = "short\nthe longest line\nmid";
    assert_eq!(longest_line(text), "the longest line");
    assert_eq!(longest_line_upper(text), "THE LONGEST LINE");
    assert_eq!(parse_config("  app \n"), Config { name: "app" });
    let dir = std::env::temp_dir().join(format!("m11-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("app.conf");
    std::fs::write(&path, "  my-app\n").unwrap();
    assert_eq!(
        load_owned(&path).unwrap(),
        OwnedConfig {
            name: "my-app".into()
        }
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn ex7c_get_or_default() {
    use ex07_fix_errors::*;
    use std::collections::HashMap;
    let mut map = HashMap::from([("a".to_string(), "x".to_string())]);
    assert_eq!(get_or_default(&mut map, "a"), "x");
    assert_eq!(get_or_default(&mut map, "b"), "");
    assert_eq!(get_or_default_two_lookups(&mut map, "c"), "");
    assert_eq!(get_or_default_two_lookups(&mut map, "a"), "x");
    assert_eq!(map.len(), 3);
}

#[test]
fn ex7d_tokens() {
    let mut p = ex07_fix_errors::Parser::new("let x = 5");
    let tokens: Vec<String> = (0..5).map(|_| p.next_token().to_string()).collect();
    assert_eq!(tokens, vec!["let", "x", "=", "5", ""]);
}

#[test]
fn ex7e_adders() {
    use ex07_fix_errors::*;
    let adder = {
        let n = 10;
        make_adder(&n) // the copy lets the closure outlive n
    };
    assert_eq!(adder(5), 15);
    let n = 3;
    assert_eq!(make_adder_borrowing(&n)(4), 7);
}

// ---- Bonus -------------------------------------------------------------------

#[test]
fn bonus_str_split() {
    use bonus_str_split::*;
    assert_eq!(
        StrSplit::new("a b c d", " ").collect::<Vec<_>>(),
        vec!["a", "b", "c", "d"]
    );
    assert_eq!(
        StrSplit::new("a,b,,c,", ',').collect::<Vec<_>>(),
        vec!["a", "b", "", "c", ""]
    );
    assert_eq!(StrSplit::new("", ',').collect::<Vec<_>>(), vec![""]);
    assert_eq!(
        StrSplit::new("ééxé", 'x').collect::<Vec<_>>(),
        vec!["éé", "é"],
        "multi-byte safe"
    );
    assert_eq!(until_char("hello world", 'o'), "hell");
    assert_eq!(until_str("key=value", "="), "key");
}
