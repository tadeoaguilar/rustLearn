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
        SharedCache {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<K: Eq + Hash + Clone, V: Clone> SharedCache<K, V> {
    pub fn new() -> Self {
        SharedCache {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// A thread that panics while holding the lock "poisons" the Mutex, and
    /// `lock()` returns Err from then on. A cache can't be left half-updated
    /// by a single insert, so recovering the guard is safe here.
    fn lock(&self) -> MutexGuard<'_, HashMap<K, V>> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Clones the value out so the lock is released before returning.
    pub fn get(&self, key: &K) -> Option<V> {
        self.lock().get(key).cloned()
    } // guard dropped here: unlocked

    pub fn insert(&self, key: K, value: V) {
        self.lock().insert(key, value);
    }

    /// Computes at most once per key, even with many threads asking at once:
    /// the check and the insert happen under one lock.
    ///
    /// Trade-off: `compute` runs while holding the lock, so while one key is
    /// being computed, *every* other caller waits. For slow computations, a
    /// per-key `OnceLock` avoids that.
    pub fn get_or_compute(&self, key: K, compute: impl FnOnce() -> V) -> V {
        let mut map = self.lock();
        map.entry(key).or_insert_with(compute).clone()
    }

    pub fn len(&self) -> usize {
        self.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.lock().is_empty()
    }
}

impl<K: Eq + Hash + Clone, V: Clone> Default for SharedCache<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

/// The RwLock version: any number of readers at once, or one writer.
pub struct ReadMostlyCache<K, V> {
    inner: Arc<RwLock<HashMap<K, V>>>,
}

impl<K, V> Clone for ReadMostlyCache<K, V> {
    fn clone(&self) -> Self {
        ReadMostlyCache {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<K: Eq + Hash + Clone, V: Clone> ReadMostlyCache<K, V> {
    pub fn new() -> Self {
        ReadMostlyCache {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.inner
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(key)
            .cloned()
    }

    /// Fast path under a read lock. If missing, take the write lock and check
    /// *again* -- another thread may have inserted it in between.
    pub fn get_or_compute(&self, key: K, compute: impl FnOnce() -> V) -> V {
        if let Some(v) = self.get(&key) {
            return v;
        }
        let mut map = self.inner.write().unwrap_or_else(PoisonError::into_inner);
        map.entry(key).or_insert_with(compute).clone()
    }

    pub fn len(&self) -> usize {
        self.inner
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K: Eq + Hash + Clone, V: Clone> Default for ReadMostlyCache<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn run() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;

    let cache: SharedCache<String, u64> = SharedCache::new();
    let computations = Arc::new(AtomicUsize::new(0));

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let cache = cache.clone(); // another handle to the same map
            let computations = Arc::clone(&computations);
            thread::spawn(move || {
                cache.get_or_compute("answer".to_string(), || {
                    computations.fetch_add(1, Ordering::SeqCst);
                    thread::sleep(Duration::from_millis(20)); // pretend it's slow
                    42
                })
            })
        })
        .collect();
    let results: Vec<u64> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    println!(
        "8 threads got {results:?}; compute ran {} time(s)",
        computations.load(Ordering::SeqCst)
    );

    let rw: ReadMostlyCache<&str, usize> = ReadMostlyCache::new();
    println!(
        "RwLock cache: {} then {}",
        rw.get_or_compute("len", || "hello".len()),
        rw.get_or_compute("len", || 999)
    );
}
