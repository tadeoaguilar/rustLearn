//! Exercise 5: Clone vs Copy.

// ---- Task 1: which types are Copy? -------------------------------------------
//
// | value                         | Copy? | why                                    |
// |-------------------------------|-------|----------------------------------------|
// | a: i32 = 42                   | yes   | all integers                           |
// | b: f64 = 2.5                  | yes   | all floats                             |
// | c: bool = true                | yes   |                                        |
// | d: char = 'a'                 | yes   |                                        |
// | e: &str = "hello"             | yes   | shared references are Copy             |
// | f: String                     | NO    | owns a heap buffer                     |
// | g: Vec<i32>                   | NO    | owns a heap buffer                     |
// | h: (i32, String)              | NO    | a tuple is Copy only if every field is |
//
// The rule: a type can be Copy only if copying its bytes is a complete,
// safe copy. Anything that owns heap memory (or implements Drop) can't be.
//
// The tests prove the table with `fn assert_copy<T: Copy>() {}` -- a function
// that only compiles for Copy types.

/// Task 2: a Copy type. Assignment copies; both variables stay valid.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

// ---- Task 3 ------------------------------------------------------------------
//
// #[derive(Copy, Clone)]
// struct Person { name: String, age: u32 }
//          ^^^^ error[E0204]: the trait `Copy` cannot be implemented for this type
//               this field does not implement `Copy`
//
// `name` owns heap memory. Copying the bytes would give two Persons pointing
// at one buffer -- a double free. So Person can only be Clone.

#[derive(Debug, PartialEq)]
pub struct Person {
    pub name: String,
    pub age: u32,
}

/// "Implement manual clone instead." `#[derive(Clone)]` generates exactly
/// this; writing it by hand shows that clone is just a function you call.
impl Clone for Person {
    fn clone(&self) -> Self {
        Person {
            name: self.name.clone(), // a new heap allocation
            age: self.age,           // u32 is Copy
        }
    }
}

pub fn run() {
    let p1 = Point { x: 5, y: 10 };
    let p2 = p1; // copy
    println!("p1: {p1:?}, p2: {p2:?}  (both valid)");

    let alice = Person {
        name: "Alice".into(),
        age: 30,
    };
    let alice2 = alice.clone(); // explicit, and visibly not free
    println!("{alice:?} / {alice2:?}");
    // let alice3 = alice; then using `alice` again -> error[E0382]: use of moved value
}
