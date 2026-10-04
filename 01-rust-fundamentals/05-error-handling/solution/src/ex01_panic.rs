//! Exercise 1: Panic Basics.
//!
//! `run()` uses `std::panic::catch_unwind` so the demo can show several panics
//! without the program dying at the first one. Don't use catch_unwind for
//! normal error handling -- it exists for things like keeping a thread pool
//! alive when one task panics.

/// Task 2 as written: an unrealistic age is treated as a *bug in the caller*.
pub fn set_age(age: i32) -> i32 {
    if !(0..=150).contains(&age) {
        panic!("Age {age} is unrealistic");
    }
    age
}

/// The same check as a recoverable error -- the right choice when the age
/// comes from user input, which can be wrong without anyone having a bug.
pub fn try_set_age(age: i32) -> Result<i32, String> {
    if (0..=150).contains(&age) {
        Ok(age)
    } else {
        Err(format!("Age {age} is unrealistic"))
    }
}

/// Task 3: `v[i]` panics when out of bounds; `v.get(i)` returns an Option.
pub fn element_at(v: &[i32], index: usize) -> Option<i32> {
    v.get(index).copied()
}

pub fn run() {
    // Silence the default "thread 'main' panicked at ..." message for the demo.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));

    let attempts: [(&str, fn()); 3] = [
        ("panic! with a message", || {
            panic!("This is a panic message")
        }),
        ("set_age(-5)", || {
            set_age(-5);
        }),
        ("v[99] on a 3-element array", || {
            let v = [1, 2, 3];
            let _ = v[std::hint::black_box(99)];
        }),
    ];
    for (label, f) in attempts {
        let result = std::panic::catch_unwind(f);
        let msg = result
            .err()
            .and_then(|e| {
                e.downcast_ref::<String>()
                    .cloned()
                    .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            })
            .unwrap_or_default();
        println!("{label:<26} -> panicked: {msg}");
    }
    std::panic::set_hook(default_hook);

    println!("set_age(25)                -> {}", set_age(25));
    println!("try_set_age(-5)            -> {:?}", try_set_age(-5));
    match element_at(&[1, 2, 3], 99) {
        Some(element) => println!("Element: {element}"),
        None => println!("v.get(99)                  -> No element at index 99"),
    }
}
