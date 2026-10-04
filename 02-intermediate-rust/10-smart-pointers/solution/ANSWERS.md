# Answers · 10 Smart Pointers

## Exercise 4

**If the parent link were `Rc` instead of `Weak`, what would happen when you
drop the root?**

Nothing would be freed. The root's strong count would be 1 (our variable) plus
1 per child (each child's parent link). Dropping our variable takes it to the
number of children, not zero, so the root stays alive; and the root keeps its
children alive through their `Rc`s. Every node in the tree leaks.

**After dropping the root, what does `child.parent()` return?**

`None`. Once the last strong reference to a node goes, the node is dropped and
every `Weak` to it fails to `upgrade()`. In the test, dropping `root` frees
`root` and (since only `root` held it strongly) `usr`, so `bin.parent()` is
`None` even though `bin` itself is still alive because we hold an `Rc` to it.

## Exercise 7

**Try `Rc<RefCell<HashMap>>` and `thread::spawn`. What's the error, and what do
`Send` and `Sync` mean?**

`error[E0277]: Rc<RefCell<HashMap<..>>> cannot be sent between threads safely
— the trait Send is not implemented for Rc<..>`.

- `T: Send` — a `T` can be **moved** to another thread.
- `T: Sync` — a `&T` can be **shared** between threads (equivalently, `&T: Send`).

`Rc` is neither: its reference count is a plain integer, so two threads
cloning or dropping at once would race. `RefCell` is `Send` but not `Sync`: its
borrow flag isn't atomic. `Arc<Mutex<T>>` is both, as long as `T: Send`. The
compiler derives these automatically from a type's fields, which is how it
knows `SharedCache` is safe to share.

**What happens to a `Mutex` if a thread panics while holding it?**

The `Mutex` becomes **poisoned**: every later `lock()` returns `Err(PoisonError)`
— a warning that the data might be half-updated. `unwrap()` turns that into a
panic in every other thread. You can recover with
`lock().unwrap_or_else(PoisonError::into_inner)`, which `SharedCache` does
because a single `HashMap::insert` can't leave the map inconsistent. The test
`ex7_cache_survives_a_panicking_thread` checks it.
