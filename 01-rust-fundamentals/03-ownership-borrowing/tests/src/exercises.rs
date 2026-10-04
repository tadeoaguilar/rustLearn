use crate::sut::*;

// ---- Exercise 1 --------------------------------------------------------------

#[test]
fn ex1_three_fixes() {
    let (a, b) = ex01_ownership::fix_with_clone();
    assert_eq!(a, "hello");
    assert_eq!(b, "hello");
    assert_eq!(ex01_ownership::fix_with_reference(), "hello hello");
    assert!(ex01_ownership::fix_by_restructuring().contains("hello"));
}

#[test]
fn ex1_functions_and_ownership() {
    let s = String::from("hello");
    let s = ex01_ownership::takes_and_gives_back(s);
    assert_eq!(ex01_ownership::borrows(&s), 5);
    assert_eq!(s, "hello", "borrows() must not consume s");
    assert_eq!(ex01_ownership::takes_ownership(s), 5);
    let n = 21;
    assert_eq!(ex01_ownership::makes_copy(n), 42);
    assert_eq!(n, 21);
}

// ---- Exercise 2 --------------------------------------------------------------

#[test]
fn ex2_references() {
    let s1 = String::from("hello");
    assert_eq!(ex02_references::calculate_length(&s1), 5);
    assert_eq!(ex02_references::calculate_length("a literal works too"), 19);

    let mut s = String::from("hello");
    ex02_references::append_world(&mut s);
    assert_eq!(s, "hello, world");
}

#[test]
fn ex2_borrowing_rules_fixed() {
    let out = ex02_references::borrowing_rules_fixed();
    assert!(out.contains("hello world"), "got: {out}");
}

// ---- Exercise 3 --------------------------------------------------------------

#[test]
fn ex3_words() {
    use ex03_strings::*;
    let sentence = String::from("Hello Rust World");
    assert_eq!(first_word(&sentence), "Hello");
    assert_eq!(last_word(&sentence), "World");
    assert_eq!(first_word("single"), "single");
    assert_eq!(last_word("single"), "single");
    assert_eq!(first_word(""), "");
}

#[test]
fn ex3_first_word_borrows_rather_than_copies() {
    let sentence = String::from("Hello Rust World");
    let word = ex03_strings::first_word(&sentence);
    // Same address: a view into the original bytes, not a copy.
    assert_eq!(word.as_ptr(), sentence.as_ptr());
}

#[test]
fn ex3_reverse_handles_multibyte_chars() {
    use ex03_strings::reverse_string;
    assert_eq!(reverse_string("hello"), "olleh");
    assert_eq!(reverse_string("héllo"), "olléh");
    assert_eq!(reverse_string(""), "");
}

#[test]
fn ex3_four_concatenations() {
    for s in ex03_strings::concat_four_ways() {
        assert_eq!(s, "Hello, World!");
    }
}

// ---- Exercise 4 --------------------------------------------------------------

#[test]
fn ex4_no_dangling() {
    assert_eq!(ex04_dangling::no_dangle(), "hello");
    let s: &'static str = ex04_dangling::static_str();
    assert_eq!(s, "hello");
}

#[test]
fn ex4_longest() {
    use ex04_dangling::longest;
    assert_eq!(longest("long one", "short"), "long one");
    assert_eq!(longest("ab", "abc"), "abc");
}

// ---- Exercise 5 --------------------------------------------------------------

/// Compiles only for Copy types -- the table in ex05_clone_copy.rs, checked by
/// the compiler. Try adding `assert_copy::<String>()` and read the error.
fn assert_copy<T: Copy>() {}

#[test]
fn ex5_copy_types() {
    assert_copy::<i32>();
    assert_copy::<f64>();
    assert_copy::<bool>();
    assert_copy::<char>();
    assert_copy::<&str>();
    assert_copy::<(i32, char)>();
    assert_copy::<ex05_clone_copy::Point>();
}

#[test]
fn ex5_point_is_copied_not_moved() {
    let p1 = ex05_clone_copy::Point { x: 5, y: 10 };
    let p2 = p1;
    assert_eq!(p1, p2); // p1 is still usable
}

#[test]
fn ex5_person_clone_is_a_deep_copy() {
    let a = ex05_clone_copy::Person {
        name: "Alice".into(),
        age: 30,
    };
    let b = a.clone();
    assert_eq!(a, b);
    assert_ne!(
        a.name.as_ptr(),
        b.name.as_ptr(),
        "clone must allocate a new String"
    );
}

// ---- Exercise 6 --------------------------------------------------------------

#[test]
fn ex6_cache() {
    let mut cache = ex06_borrow_rules::Cache::new();
    assert!(cache.is_empty());
    cache.set("name".into(), "Alice".into());
    assert_eq!(cache.get("name").map(String::as_str), Some("Alice"));
    assert_eq!(cache.get("missing"), None);

    assert!(cache.copy_value("name", "backup"));
    assert!(!cache.copy_value("missing", "x"));
    assert_eq!(cache.get("backup").map(String::as_str), Some("Alice"));
    assert_eq!(cache.len(), 2);
}

// ---- Exercise 7 --------------------------------------------------------------

#[test]
fn ex7_text_buffer_scenario() {
    use ex07_text_buffer::TextBuffer;
    let mut buffer = TextBuffer::new();
    buffer.append("Hello ");
    buffer.append("World");
    assert_eq!(buffer.content(), "Hello World");
    assert_eq!(buffer.len(), 11);

    buffer.prepend("Say: ");
    assert_eq!(buffer.content(), "Say: Hello World");

    // NOT vec![9, 13] as exercises.md says -- see the module README.
    assert_eq!(buffer.search("l"), vec![7, 8, 14]);
    assert_eq!(buffer.search("Hello"), vec![5]);
    assert_eq!(buffer.search("zzz"), Vec::<usize>::new());

    buffer.replace("World", "Rust");
    assert_eq!(buffer.content(), "Say: Hello Rust");

    buffer.clear();
    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
}

#[test]
fn ex7_from_str_and_empty_search() {
    use ex07_text_buffer::TextBuffer;
    let buffer = TextBuffer::from_str("aaa");
    assert_eq!(buffer.content(), "aaa");
    assert_eq!(buffer.search("a"), vec![0, 1, 2]);
    assert_eq!(
        buffer.search(""),
        Vec::<usize>::new(),
        "an empty needle matches nothing"
    );
}

// ---- Exercise 8 --------------------------------------------------------------

#[test]
fn ex8_borrowing_from_a_vec() {
    let v = vec![String::from("hello"), String::from("world")];
    assert_eq!(ex08_collections::first(&v), Some("hello"));
    assert_eq!(ex08_collections::first(&[]), None);
    assert_eq!(ex08_collections::total_len(&v), 10);
    assert_eq!(v.len(), 2, "v is still ours");
    assert_eq!(ex08_collections::into_upper(v), vec!["HELLO", "WORLD"]);
}

#[test]
fn ex8_modify_in_place() {
    let mut numbers = vec![1, 2, 3, 4, 5];
    ex08_collections::double_in_place(&mut numbers);
    assert_eq!(numbers, vec![2, 4, 6, 8, 10]);
}

#[test]
fn ex8_remove_negatives() {
    let mut nums = vec![1, -2, 3, -4, 5];
    ex08_collections::remove_negatives(&mut nums);
    assert_eq!(nums, vec![1, 3, 5]);

    let mut nums = vec![1, -2, 3, -4, 5];
    let removed = ex08_collections::take_negatives(&mut nums);
    assert_eq!(nums, vec![1, 3, 5]);
    assert_eq!(removed, vec![-2, -4]);
}

// ---- Bonus -------------------------------------------------------------------

#[test]
fn bonus_simple_rc_counts_references() {
    use bonus_simple_rc::SimpleRc;
    let rc1 = SimpleRc::new(String::from("hello"));
    assert_eq!(rc1.ref_count(), 1);
    let rc2 = rc1.clone();
    assert_eq!(rc1.ref_count(), 2);
    assert_eq!(rc2.ref_count(), 2, "both handles share one count");
    assert_eq!(rc1.get(), "hello");
    assert_eq!(rc2.get(), "hello");
    drop(rc2);
    assert_eq!(rc1.ref_count(), 1);
}

#[test]
fn bonus_value_is_dropped_exactly_once_by_the_last_owner() {
    use bonus_simple_rc::SimpleRc;
    use std::cell::Cell;
    use std::rc::Rc;

    struct Tracker(Rc<Cell<u32>>);
    impl Drop for Tracker {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    let drops = Rc::new(Cell::new(0));
    let a = SimpleRc::new(Tracker(drops.clone()));
    let b = a.clone();
    let c = b.clone();
    drop(a);
    drop(b);
    assert_eq!(drops.get(), 0, "still one owner left");
    drop(c);
    assert_eq!(
        drops.get(),
        1,
        "dropped exactly once, when the last owner went"
    );
}
