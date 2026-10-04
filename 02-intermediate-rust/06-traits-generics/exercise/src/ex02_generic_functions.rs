//! Exercise 2: Generic Functions.

/// Task 1 as written. `PartialOrd` because we use `>`. Returns a reference so
/// it works for types that can't be copied (e.g. String) without cloning.
///
/// # Panics
/// On an empty slice -- `list[0]` doesn't exist. See `largest_checked`.
pub fn largest<T: PartialOrd>(list: &[T]) -> &T {
    todo!("Exercise 2")
}

/// The non-panicking version: an empty slice has no largest element.
pub fn largest_checked<T: PartialOrd>(list: &[T]) -> Option<&T> {
    todo!("Exercise 2")
}

/// Task 2. `std::mem::swap` works for any T because it moves bytes without
/// needing to know anything about the type -- no bounds required.
pub fn swap<T>(a: &mut T, b: &mut T) {
    todo!("Exercise 2")
}

/// Task 3: two independent type parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct Pair<T, U> {
    first: T,
    second: U,
}

impl<T, U> Pair<T, U> {
    pub fn new(first: T, second: U) -> Self {
        todo!("Exercise 2")
    }

    pub fn get_first(&self) -> &T {
        todo!("Exercise 2")
    }

    pub fn get_second(&self) -> &U {
        todo!("Exercise 2")
    }

    /// Consumes the pair and returns it with the types swapped: Pair<U, T>.
    pub fn swap(self) -> Pair<U, T> {
        todo!("Exercise 2")
    }
}

pub fn run() {
    todo!("Exercise 2")
}
