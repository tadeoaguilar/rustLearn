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
    User {
        username,
        email,
        active: true,
        sign_in_count: 1,
    }
}

/// Task 2: the *binding* must be `mut` -- Rust has no per-field mutability.
pub fn change_email(mut user: User, new_email: &str) -> User {
    user.email = new_email.to_string();
    user
}

/// Task 3: struct update syntax.
///
/// "Can you still use user1? Why or why not?" -- *Partly*. `..user1` moves the
/// fields it takes. `username` is a String, so it is moved into user2 and
/// `user1.username` is gone -- and so is `user1` as a whole. But `user1.email`
/// was not taken (user2 got its own), and `active`/`sign_in_count` are Copy,
/// so those individual fields are still usable. This is a *partial move*.
pub fn struct_update(user1: User) -> (User, String, bool) {
    let user2 = User {
        email: String::from("another@example.com"),
        ..user1
    };
    // println!("{:?}", user1);
    //                  ^^^^^ error[E0382]: borrow of partially moved value: `user1`
    let still_usable_email = user1.email; // never moved, so this is fine
    let still_usable_active = user1.active; // Copy
    (user2, still_usable_email, still_usable_active)
}

pub fn run() {
    let user1 = User {
        email: String::from("user@example.com"),
        username: String::from("user123"),
        active: true,
        sign_in_count: 1,
    };
    println!("User: {}", user1.username);

    let user = change_email(
        build_user("test@example.com".into(), "test".into()),
        "newemail@example.com",
    );
    println!("changed email: {}", user.email);

    let (user2, old_email, active) = struct_update(user1);
    println!("user2: {user2:?}");
    println!("still have user1.email = {old_email}, user1.active = {active}");
}
