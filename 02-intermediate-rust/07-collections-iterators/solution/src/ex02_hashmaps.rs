//! Exercise 2: HashMap Basics.

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

/// Task 1: word frequency. `entry().or_insert(0)` returns `&mut usize` to the
/// existing or freshly inserted count -- one hash lookup instead of a
/// `get` followed by an `insert`.
pub fn word_frequency(text: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for word in text
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|w| !w.is_empty())
    {
        *counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    counts
}

/// The n most frequent words; ties broken alphabetically so the result is
/// deterministic (HashMap iteration order is not).
pub fn top_words(text: &str, n: usize) -> Vec<(String, usize)> {
    let mut words: Vec<(String, usize)> = word_frequency(text).into_iter().collect();
    words.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    words.truncate(n);
    words
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Person {
    pub name: String,
    pub age: u32,
}

/// Task 2: group people by age. A BTreeMap keeps the ages sorted, which makes
/// the output (and the tests) predictable.
pub fn group_by_age(people: &[Person]) -> BTreeMap<u32, Vec<String>> {
    let mut groups: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    for p in people {
        groups.entry(p.age).or_default().push(p.name.clone());
    }
    groups
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
        assert!(capacity > 0, "capacity must be positive");
        LruCache {
            capacity,
            clock: 0,
            entries: HashMap::new(),
        }
    }

    fn tick(&mut self) -> u64 {
        self.clock += 1;
        self.clock
    }

    /// `&mut self` even though it's a read: a get counts as a "use".
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let now = self.tick();
        let (value, last_used) = self.entries.get_mut(key)?;
        *last_used = now;
        Some(value)
    }

    /// Inserts or replaces. Returns the evicted (key, value), if any.
    pub fn set(&mut self, key: K, value: V) -> Option<(K, V)> {
        let now = self.tick();
        let mut evicted = None;
        if !self.entries.contains_key(&key) && self.entries.len() == self.capacity {
            evicted = self.evict_lru();
        }
        self.entries.insert(key, (value, now));
        evicted
    }

    /// Explicit removal.
    pub fn evict(&mut self, key: &K) -> Option<V> {
        self.entries.remove(key).map(|(v, _)| v)
    }

    fn evict_lru(&mut self) -> Option<(K, V)> {
        let oldest = self
            .entries
            .iter()
            .min_by_key(|(_, (_, t))| *t)
            .map(|(k, _)| k.clone())?;
        self.entries.remove(&oldest).map(|(v, _)| (oldest, v))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

pub fn run() {
    let mut scores = HashMap::new();
    scores.insert("Blue", 10);
    scores.insert("Red", 50);
    if let Some(score) = scores.get("Blue") {
        println!("Blue: {score}");
    }
    scores.entry("Yellow").or_insert(0);
    *scores.entry("Blue").or_insert(0) += 10;
    let mut sorted: Vec<_> = scores.iter().collect();
    sorted.sort();
    println!("scores: {sorted:?}");

    let text = "the quick brown fox jumps over the lazy dog the end";
    println!("top 2 words: {:?}", top_words(text, 2));

    let people = vec![
        Person {
            name: "Ann".into(),
            age: 30,
        },
        Person {
            name: "Bob".into(),
            age: 25,
        },
        Person {
            name: "Cat".into(),
            age: 30,
        },
    ];
    println!("by age: {:?}", group_by_age(&people));

    let mut cache = LruCache::new(2);
    cache.set("a", 1);
    cache.set("b", 2);
    cache.get(&"a"); // "a" is now more recent than "b"
    println!("set c evicts {:?}", cache.set("c", 3));
}
