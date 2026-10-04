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
    Rc::new(RefCell::new(Person {
        name: name.to_string(),
        friends: Vec::new(),
    }))
}

fn is_friend(of: &PersonRef, who: &PersonRef) -> bool {
    of.borrow()
        .friends
        .iter()
        .any(|w| w.upgrade().is_some_and(|p| Rc::ptr_eq(&p, who)))
}

/// Both directions; ignores self-friendship and duplicates.
pub fn befriend(a: &PersonRef, b: &PersonRef) {
    if Rc::ptr_eq(a, b) || is_friend(a, b) {
        return;
    }
    // Two separate borrow_mut()s, one after the other -- never both at once.
    a.borrow_mut().friends.push(Rc::downgrade(b));
    b.borrow_mut().friends.push(Rc::downgrade(a));
}

/// Friends that still exist, sorted. Dropped people simply disappear.
pub fn friend_names(p: &PersonRef) -> Vec<String> {
    let mut names: Vec<String> = p
        .borrow()
        .friends
        .iter()
        .filter_map(Weak::upgrade)
        .map(|f| f.borrow().name.clone())
        .collect();
    names.sort();
    names
}

pub fn mutual_friends(a: &PersonRef, b: &PersonRef) -> Vec<String> {
    let fa: BTreeSet<String> = friend_names(a).into_iter().collect();
    let fb: BTreeSet<String> = friend_names(b).into_iter().collect();
    fa.intersection(&fb).cloned().collect()
}

/// Everyone reachable through friendships, excluding `from`, sorted. BFS with
/// a visited list compared by pointer, so two people named alike stay distinct.
pub fn reachable(from: &PersonRef) -> Vec<String> {
    let mut visited: Vec<PersonRef> = vec![Rc::clone(from)];
    let mut queue = VecDeque::from([Rc::clone(from)]);
    while let Some(current) = queue.pop_front() {
        let friends: Vec<PersonRef> = current
            .borrow()
            .friends
            .iter()
            .filter_map(Weak::upgrade)
            .collect();
        for f in friends {
            if !visited.iter().any(|v| Rc::ptr_eq(v, &f)) {
                visited.push(Rc::clone(&f));
                queue.push_back(f);
            }
        }
    }
    let mut names: Vec<String> = visited
        .iter()
        .skip(1)
        .map(|p| p.borrow().name.clone())
        .collect();
    names.sort();
    names
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
        self.drops.set(self.drops.get() + 1);
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
    let drops = Rc::new(Cell::new(0));
    let a = Rc::new(CycleNode {
        next: RefCell::new(None),
        drops: Rc::clone(&drops),
    });
    let b = Rc::new(CycleNode {
        next: RefCell::new(Some(Rc::clone(&a))),
        drops: Rc::clone(&drops),
    });
    *a.next.borrow_mut() = Some(Rc::clone(&b));
    let watcher = Rc::downgrade(&a);
    drop(a); // a's strong count: 2 -> 1 (b still holds it)
    drop(b); // b's strong count: 2 -> 1 (a still holds it)

    let drops_after_dropping_variables = drops.get();
    let still_alive = watcher.upgrade().is_some();

    // Break the cycle: a lets go of b, b is freed, b's drop lets go of a.
    if let Some(a) = watcher.upgrade() {
        a.next.borrow_mut().take();
    }
    LeakReport {
        drops_after_dropping_variables,
        still_alive,
        drops_after_breaking_cycle: drops.get(),
    }
}

/// Two friends as `befriend` stores them -- Weak links, no strong cycle.
/// Returns whether the first person is still alive after both variables are
/// dropped: false, they were freed.
pub fn no_leak_with_weak() -> bool {
    let a = person("a");
    let b = person("b");
    befriend(&a, &b);
    let watcher = Rc::downgrade(&a);
    drop(a);
    drop(b);
    watcher.upgrade().is_some()
}

pub fn run() {
    let names = ["Ann", "Bob", "Cat", "Dan", "Eve"];
    let people: Vec<PersonRef> = names.iter().map(|n| person(n)).collect();
    let [ann, bob, cat, dan, eve] = [0, 1, 2, 3, 4].map(|i| &people[i]);
    befriend(ann, bob);
    befriend(ann, cat);
    befriend(bob, cat);
    befriend(cat, dan);
    println!("Ann's friends: {:?}", friend_names(ann));
    println!("mutual(Ann, Bob): {:?}", mutual_friends(ann, bob));
    println!("reachable from Ann: {:?}", reachable(ann));
    println!("reachable from Eve: {:?}", reachable(eve));
    let r = leak_with_rc_cycle();
    println!(
        "strong Rc cycle: {} drops after both variables went away, still alive = {} -- leaked;\n                 {} drops once the cycle was broken by hand",
        r.drops_after_dropping_variables, r.still_alive, r.drops_after_breaking_cycle
    );
    println!(
        "Weak friend links: still alive after dropping = {}",
        no_leak_with_weak()
    );
}
