//! Exercise 3: Iterator Basics.
//!
//! `iter()` yields `&T`, `iter_mut()` yields `&mut T`, `into_iter()` yields `T`.
//! That's why `filter` closures often see `&&i32`: filter passes a reference
//! to each item, and the items are already references.

/// Task 1.
pub fn sum_of_squares_of_evens(v: &[i32]) -> i32 {
    v.iter().filter(|&&x| x % 2 == 0).map(|x| x * x).sum()
}

/// Task 2. `find` stops at the first match -- the rest of the (possibly
/// infinite) iterator is never touched.
pub fn first_divisible_by_3_and_5(v: &[i32]) -> Option<i32> {
    v.iter().copied().find(|x| x % 15 == 0)
}

/// The same question over *all* positive integers: an infinite range is fine
/// because iterators are lazy.
pub fn first_positive_divisible_by(a: u32, b: u32) -> u32 {
    (1..)
        .find(|n| n % a == 0 && n % b == 0)
        .expect("an infinite range always finds one")
}

/// Task 3.
pub fn flatten<T: Clone>(nested: &[Vec<T>]) -> Vec<T> {
    nested.iter().flatten().cloned().collect()
}

/// The basics from the exercise, returned so they can be checked.
pub struct Basics {
    pub sum: i32,
    pub product: i32,
    pub doubled: Vec<i32>,
    pub evens: Vec<i32>,
    pub even_squares: Vec<i32>,
    pub first_three: Vec<i32>,
    pub skip_two: Vec<i32>,
}

pub fn basics(v: &[i32]) -> Basics {
    Basics {
        sum: v.iter().sum(),
        product: v.iter().product(),
        doubled: v.iter().map(|x| x * 2).collect(),
        evens: v.iter().filter(|x| *x % 2 == 0).copied().collect(),
        even_squares: v.iter().filter(|x| *x % 2 == 0).map(|x| x * x).collect(),
        first_three: v.iter().take(3).copied().collect(),
        skip_two: v.iter().skip(2).copied().collect(),
    }
}

pub fn run() {
    let v = vec![1, 2, 3, 4, 5];
    let b = basics(&v);
    println!(
        "sum {} product {} doubled {:?} evens {:?}",
        b.sum, b.product, b.doubled, b.evens
    );
    println!(
        "even squares {:?} take(3) {:?} skip(2) {:?}",
        b.even_squares, b.first_three, b.skip_two
    );
    for (index, value) in v.iter().enumerate() {
        print!("{index}:{value} ");
    }
    println!();
    println!("sum_of_squares_of_evens = {}", sum_of_squares_of_evens(&v));
    println!(
        "first divisible by 3 and 5 in [7, 30, 45] = {:?}",
        first_divisible_by_3_and_5(&[7, 30, 45])
    );
    println!(
        "first positive divisible by 4 and 6 = {}",
        first_positive_divisible_by(4, 6)
    );
    println!("flatten = {:?}", flatten(&[vec![1, 2], vec![], vec![3]]));

    // Laziness, visibly: nothing prints until `collect` pulls items through.
    let lazy = v.iter().map(|x| {
        println!("  mapping {x}");
        x * 10
    });
    println!("map created, nothing mapped yet");
    let first_two: Vec<i32> = lazy.take(2).collect();
    println!("took {first_two:?} -- 3, 4, 5 were never mapped");
}
