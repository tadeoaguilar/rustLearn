//! Exercise 7: A Thread-Safe Cache with Arc and Mutex.
//!
//! `Rc<RefCell<HashMap>>` won't compile with `thread::spawn`:
//!
//! ```text
//! error[E0277]: `Rc<RefCell<HashMap<..>>>` cannot be sent between threads safely
//!               the trait `Send` is not implemented for `Rc<..>`
//! ```
//!
//! `Rc`'s count isn't atomic, so two threads cloning it at once could corrupt
//! it. `Arc` uses atomic operations (a bit slower) and is `Send + Sync`.
//! `RefCell`'s borrow flag isn't thread-safe either; `Mutex` is the
//! thread-safe equivalent.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, RwLock};

/// Cloning a SharedCache clones the `Arc` -- a new handle to the *same* map.
pub struct SharedCache<K, V> {
    inner: Arc<Mutex<HashMap<K, V>>>,
}

// Written by hand: #[derive(Clone)] would require K: Clone and V: Clone,
// which cloning an Arc doesn't need.
impl<K, V> Clone for SharedCache<K, V> {
    fn clone(&self) -> Self {
        todo!("Exercise 7")
    }
}

impl<K: Eq + Hash + Clone, V: Clone> SharedCache<K, V> {
    pub fn new() -> Self {
        todo!("Exercise 7")
    }

    /// A thread that panics while holding the lock "poisons" the Mutex, and
    /// `lock()` returns Err from then on. A cache can't be left half-updated
    /// by a single insert, so recovering the guard is safe here.
    fn lock(&self) -> MutexGuard<'_, HashMap<K, V>> {
        todo!("Exercise 7")
    }

    /// Clones the value out so the lock is released before returning.
    pub fn get(&self, key: &K) -> Option<V> {
        todo!("Exercise 7")
    } // guard dropped here: unlocked

    pub fn insert(&self, key: K, value: V) {
        todo!("Exercise 7")
    }

    /// Computes at most once per key, even with many threads asking at once:
    /// the check and the insert happen under one lock.
    ///
    /// Trade-off: `compute` runs while holding the lock, so while one key is
    /// being computed, *every* other caller waits. For slow computations, a
    /// per-key `OnceLock` avoids that.
    pub fn get_or_compute(&self, key: K, compute: impl FnOnce() -> V) -> V {
        todo!("Exercise 7")
    }

    pub fn len(&self) -> usize {
        todo!("Exercise 7")
    }

    pub fn is_empty(&self) -> bool {
        todo!("Exercise 7")
    }
}

impl<K: Eq + Hash + Clone, V: Clone> Default for SharedCache<K, V> {
    fn default() -> Self {
        todo!("Exercise 7")
    }
}

/// The RwLock version: any number of readers at once, or one writer.
pub struct ReadMostlyCache<K, V> {
    inner: Arc<RwLock<HashMap<K, V>>>,
}

impl<K, V> Clone for ReadMostlyCache<K, V> {
    fn clone(&self) -> Self {
        todo!("Exercise 7")
    }
}

impl<K: Eq + Hash + Clone, V: Clone> ReadMostlyCache<K, V> {
    pub fn new() -> Self {
        todo!("Exercise 7")
    }

    pub fn get(&self, key: &K) -> Option<V> {
        todo!("Exercise 7")
    }

    /// Fast path under a read lock. If missing, take the write lock and check
    /// *again* -- another thread may have inserted it in between.
    pub fn get_or_compute(&self, key: K, compute: impl FnOnce() -> V) -> V {
        todo!("Exercise 7")
    }

    pub fn len(&self) -> usize {
        todo!("Exercise 7")
    }

    pub fn is_empty(&self) -> bool {
        todo!("Exercise 7")
    }
}

impl<K: Eq + Hash + Clone, V: Clone> Default for ReadMostlyCache<K, V> {
    fn default() -> Self {
        todo!("Exercise 7")
    }
}

pub fn run() {
    todo!("Exercise 7")
}
