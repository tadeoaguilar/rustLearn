//! Exercise 6: Shared Mutable Graphs with Rc<RefCell<T>>.
//!
//! `Rc` gives several owners; `RefCell` lets any of them mutate. Friendship
//! is mutual, so a naive `Vec<Rc<..>>` of friends would make every pair of
//! friends a reference cycle. Friends are stored as `Weak` instead: people are
//! owned by whoever holds a `PersonRef` (here, the network's `Vec`).

use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, VecDeque};
use std::rc::{Rc, Weak};

pub type PersonRef = Rc<RefCell<Person>>;

#[derive(Debug)]
pub struct Person {
    pub name: String,
    friends: Vec<Weak<RefCell<Person>>>,
}

pub fn person(name: &str) -> PersonRef {
    todo!("Exercise 6")
}

fn is_friend(of: &PersonRef, who: &PersonRef) -> bool {
    todo!("Exercise 6")
}

/// Both directions; ignores self-friendship and duplicates.
pub fn befriend(a: &PersonRef, b: &PersonRef) {
    todo!("Exercise 6")
}

/// Friends that still exist, sorted. Dropped people simply disappear.
pub fn friend_names(p: &PersonRef) -> Vec<String> {
    todo!("Exercise 6")
}

pub fn mutual_friends(a: &PersonRef, b: &PersonRef) -> Vec<String> {
    todo!("Exercise 6")
}

/// Everyone reachable through friendships, excluding `from`, sorted. BFS with
/// a visited list compared by pointer, so two people named alike stay distinct.
pub fn reachable(from: &PersonRef) -> Vec<String> {
    todo!("Exercise 6")
}

// ---- Task 2: a leak ---------------------------------------------------------

/// A node that can point at another with a *strong* reference, and counts
/// its own drops.
pub struct CycleNode {
    pub next: RefCell<Option<Rc<CycleNode>>>,
    drops: Rc<Cell<u32>>,
}

impl Drop for CycleNode {
    fn drop(&mut self) {
        // TODO Exercise 6: a todo!() here could abort the test run, so this is empty.
    }
}

/// What `leak_with_rc_cycle` observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeakReport {
    /// Drops counted right after both variables went out of scope.
    pub drops_after_dropping_variables: u32,
    /// Could node `a` still be reached through a Weak at that point?
    pub still_alive: bool,
    /// Drops counted after breaking the cycle by hand.
    pub drops_after_breaking_cycle: u32,
}

/// Builds a <-> b with strong references and drops both variables. Nothing
/// is freed: each node keeps the other's strong count at 1. Then it breaks
/// the cycle by hand, and both nodes are freed at once.
pub fn leak_with_rc_cycle() -> LeakReport {
    todo!("Exercise 6")
}

/// Two friends as `befriend` stores them -- Weak links, no strong cycle.
/// Returns whether the first person is still alive after both variables are
/// dropped: false, they were freed.
pub fn no_leak_with_weak() -> bool {
    todo!("Exercise 6")
}

pub fn run() {
    todo!("Exercise 6")
}
