//! Exercise 2: Generic Functions.

/// Task 1 as written. `PartialOrd` because we use `>`. Returns a reference so
/// it works for types that can't be copied (e.g. String) without cloning.
///
/// # Panics
/// On an empty slice -- `list[0]` doesn't exist. See `largest_checked`.
pub fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

/// The non-panicking version: an empty slice has no largest element.
pub fn largest_checked<T: PartialOrd>(list: &[T]) -> Option<&T> {
    let mut iter = list.iter();
    let first = iter.next()?;
    Some(iter.fold(first, |best, item| if item > best { item } else { best }))
    // For T: Ord, the standard library has it already: list.iter().max()
}

/// Task 2. `std::mem::swap` works for any T because it moves bytes without
/// needing to know anything about the type -- no bounds required.
pub fn swap<T>(a: &mut T, b: &mut T) {
    std::mem::swap(a, b);
}

/// Task 3: two independent type parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct Pair<T, U> {
    first: T,
    second: U,
}

impl<T, U> Pair<T, U> {
    pub fn new(first: T, second: U) -> Self {
        Pair { first, second }
    }

    pub fn get_first(&self) -> &T {
        &self.first
    }

    pub fn get_second(&self) -> &U {
        &self.second
    }

    /// Consumes the pair and returns it with the types swapped: Pair<U, T>.
    pub fn swap(self) -> Pair<U, T> {
        Pair {
            first: self.second,
            second: self.first,
        }
    }
}

pub fn run() {
    let numbers = vec![34, 50, 25, 100, 65];
    println!("Largest: {}", largest(&numbers));
    let chars = vec!['y', 'm', 'a', 'q'];
    println!("Largest: {}", largest(&chars));
    let words = vec![String::from("apple"), String::from("pear")];
    println!("Largest: {} (no clone needed)", largest(&words));
    println!("largest_checked(&[]) = {:?}", largest_checked::<i32>(&[]));

    let (mut x, mut y) = (5, 10);
    swap(&mut x, &mut y);
    println!("after swap: x = {x}, y = {y}");

    let pair = Pair::new(5, "hello");
    println!("{}, {}", pair.get_first(), pair.get_second());
    println!("{:?}", pair.swap());
}
