//! Exercise 4: Advanced Iterators.

/// The exercise's examples, returned for testing.
#[derive(Debug, Clone, PartialEq)]
pub struct AdvancedBasics {
    pub sum: i32,
    pub running_sum: Vec<i32>,
    pub combined: Vec<(i32, &'static str)>,
    pub flat: Vec<i32>,
}

pub fn advanced_basics() -> AdvancedBasics {
    let numbers = [1, 2, 3, 4, 5];
    // Clippy points out that this particular fold is just `.sum()` -- true,
    // but fold is the general tool that sum, product, max... are built from.
    #[allow(clippy::unnecessary_fold)]
    let sum = numbers.iter().fold(0, |acc, x| acc + x);
    let running_sum: Vec<i32> = numbers
        .iter()
        .scan(0, |state, x| {
            *state += x;
            Some(*state)
        })
        .collect();
    let a = [1, 2, 3];
    let b = ["one", "two", "three"];
    let combined: Vec<(i32, &str)> = a.iter().copied().zip(b.iter().copied()).collect();
    let nested = [vec![1, 2], vec![3, 4]];
    let flat: Vec<i32> = nested.iter().flat_map(|v| v.iter()).copied().collect();
    AdvancedBasics {
        sum,
        running_sum,
        combined,
        flat,
    }
}

/// Task 1: Fibonacci with `std::iter::successors`, which repeatedly applies a
/// function to the previous item until it returns None.
///
/// The state is (current, next). `next` is an Option: `checked_add` turns it
/// into None when the following number would overflow a u64, and the
/// iterator ends one step later -- after yielding the largest u64 Fibonacci
/// number (F93), not before it.
pub fn fibonacci() -> impl Iterator<Item = u64> {
    std::iter::successors(Some((0u64, Some(1u64))), |&(a, b)| {
        b.map(|b| (b, a.checked_add(b)))
    })
    .map(|(a, _)| a)
}

/// Task 2: every (x, y) pair. `flat_map` + `map` is a nested loop as an iterator.
pub fn cartesian_product<A: Clone, B: Clone>(a: &[A], b: &[B]) -> Vec<(A, B)> {
    a.iter()
        .flat_map(|x| b.iter().map(move |y| (x.clone(), y.clone())))
        .collect()
}

/// Task 3: one pass, two outputs.
pub fn partition_even_odd(v: &[i32]) -> (Vec<i32>, Vec<i32>) {
    v.iter().partition(|&&x| x % 2 == 0)
}

pub fn run() {
    let b = advanced_basics();
    println!(
        "fold sum {}, scan {:?}\nzip {:?}\nflat_map {:?}",
        b.sum, b.running_sum, b.combined, b.flat
    );
    println!("fibonacci: {:?}", fibonacci().take(12).collect::<Vec<_>>());
    println!(
        "fibonacci numbers that fit in a u64: {}",
        fibonacci().count()
    );
    println!("cartesian: {:?}", cartesian_product(&[1, 2], &['a', 'b']));
    println!("partition: {:?}", partition_even_odd(&[1, 2, 3, 4, 5, 6]));
    // chunks, windows, rev, step_by, max_by_key: a few more worth knowing.
    let v = [3, 1, 4, 1, 5, 9, 2, 6];
    println!(
        "windows(2) increasing pairs: {}",
        v.windows(2).filter(|w| w[1] > w[0]).count()
    );
    println!("chunks(3): {:?}", v.chunks(3).collect::<Vec<_>>());
    println!("step_by(3): {:?}", v.iter().step_by(3).collect::<Vec<_>>());
    println!(
        "position of max: {:?}",
        v.iter()
            .enumerate()
            .max_by_key(|(_, x)| **x)
            .map(|(i, _)| i)
    );
}
