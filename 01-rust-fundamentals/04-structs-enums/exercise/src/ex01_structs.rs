//! Exercise 1: Basic Structs.

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub username: String,
    pub email: String,
    pub sign_in_count: u64,
    pub active: bool,
}

/// A constructor-style function. Field init shorthand: `username` instead of
/// `username: username` when the variable and field share a name.
pub fn build_user(email: String, username: String) -> User {
    todo!("Exercise 1")
}

/// Task 2: the *binding* must be `mut` -- Rust has no per-field mutability.
pub fn change_email(mut user: User, new_email: &str) -> User {
    todo!("Exercise 1")
}

/// Task 3: struct update syntax.
///
/// "Can you still use user1? Why or why not?" -- *Partly*. `..user1` moves the
/// fields it takes. `username` is a String, so it is moved into user2 and
/// `user1.username` is gone -- and so is `user1` as a whole. But `user1.email`
/// was not taken (user2 got its own), and `active`/`sign_in_count` are Copy,
/// so those individual fields are still usable. This is a *partial move*.
pub fn struct_update(user1: User) -> (User, String, bool) {
    todo!("Exercise 1")
}

pub fn run() {
    todo!("Exercise 1")
}
