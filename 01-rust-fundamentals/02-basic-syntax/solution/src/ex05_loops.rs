//! Exercise 5: Loops.

/// Task 1: `loop` is an expression too -- `break value` makes it return.
pub fn find_first_multiple_of_7() -> i32 {
    let mut counter = 1;
    loop {
        if counter % 7 == 0 {
            break counter;
        }
        counter += 1;
    }
}

/// Task 2: while loop. Returns the lines instead of printing them.
pub fn countdown(from: i32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut n = from;
    while n > 0 {
        lines.push(n.to_string());
        n -= 1;
    }
    lines.push("Liftoff!".to_string());
    lines
}

/// Task 3: the first `n` Fibonacci numbers using a `for` loop.
pub fn fibonacci(n: u32) -> Vec<u64> {
    let mut seq = Vec::with_capacity(n as usize);
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        seq.push(a);
        (a, b) = (b, a + b); // destructuring assignment
    }
    seq
}

pub fn print_fibonacci(n: u32) {
    let as_text: Vec<String> = fibonacci(n).iter().map(u64::to_string).collect();
    println!("{}", as_text.join(", "));
}

/// Task 4: loop labels. Returns the (i, j) at which we broke out of *both*
/// loops. Without the label, `break` would only leave the inner loop and the
/// outer one would carry on with i + 1.
pub fn nested_loop_example() -> Option<(i32, i32)> {
    let mut found = None;
    'outer: for i in 0..5 {
        for j in 0..5 {
            if i * j > 10 {
                found = Some((i, j));
                break 'outer;
            }
        }
    }
    found
}

pub fn run() {
    println!("first multiple of 7: {}", find_first_multiple_of_7());
    for line in countdown(3) {
        println!("{line}");
    }
    print_fibonacci(10);
    println!("broke out of both loops at {:?}", nested_loop_example());
}
