//! Exercise 2: HashMap Basics.

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

/// Task 1: word frequency. `entry().or_insert(0)` returns `&mut usize` to the
/// existing or freshly inserted count -- one hash lookup instead of a
/// `get` followed by an `insert`.
pub fn word_frequency(text: &str) -> HashMap<String, usize> {
    todo!("Exercise 2")
}

/// The n most frequent words; ties broken alphabetically so the result is
/// deterministic (HashMap iteration order is not).
pub fn top_words(text: &str, n: usize) -> Vec<(String, usize)> {
    todo!("Exercise 2")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Person {
    pub name: String,
    pub age: u32,
}

/// Task 2: group people by age. A BTreeMap keeps the ages sorted, which makes
/// the output (and the tests) predictable.
pub fn group_by_age(people: &[Person]) -> BTreeMap<u32, Vec<String>> {
    todo!("Exercise 2")
}

/// Task 3: a cache with get / set / evict, holding at most `capacity`
/// entries. When full, `set` evicts the *least recently used* entry (LRU).
///
/// Each entry stores the "time" it was last touched. Finding the oldest is a
/// linear scan -- fine for a teaching example. A production LRU pairs the map
/// with a linked list for O(1) eviction (see the `lru` crate).
#[derive(Debug)]
pub struct LruCache<K, V> {
    capacity: usize,
    clock: u64,
    entries: HashMap<K, (V, u64)>,
}

impl<K: Eq + Hash + Clone, V> LruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        todo!("Exercise 2")
    }

    fn tick(&mut self) -> u64 {
        todo!("Exercise 2")
    }

    /// `&mut self` even though it's a read: a get counts as a "use".
    pub fn get(&mut self, key: &K) -> Option<&V> {
        todo!("Exercise 2")
    }

    /// Inserts or replaces. Returns the evicted (key, value), if any.
    pub fn set(&mut self, key: K, value: V) -> Option<(K, V)> {
        todo!("Exercise 2")
    }

    /// Explicit removal.
    pub fn evict(&mut self, key: &K) -> Option<V> {
        todo!("Exercise 2")
    }

    fn evict_lru(&mut self) -> Option<(K, V)> {
        todo!("Exercise 2")
    }

    pub fn len(&self) -> usize {
        todo!("Exercise 2")
    }

    pub fn is_empty(&self) -> bool {
        todo!("Exercise 2")
    }
}

pub fn run() {
    todo!("Exercise 2")
}
