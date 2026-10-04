//! Bonus Challenge: Reference Counting Simulation.
//!
//! A tiny `Rc<T>`. This is the first `unsafe` code in the course, so every
//! `unsafe` block says *why* it is sound -- a habit worth keeping.
//!
//! The raw pointer also makes `SimpleRc` automatically `!Send` and `!Sync`,
//! which is correct: the count is not atomic, so two threads touching it would
//! be a data race. (`Arc` is the thread-safe version; see module 10.)

use std::ptr::NonNull;

struct RcBox<T> {
    value: T,
    ref_count: usize,
}

pub struct SimpleRc<T> {
    // NonNull instead of `*mut`: same thing, but it can never be null, which
    // documents the invariant and lets Option<SimpleRc<T>> be pointer-sized.
    data: NonNull<RcBox<T>>,
}

impl<T> SimpleRc<T> {
    pub fn new(value: T) -> Self {
        // Box allocates on the heap; leaking it hands us the pointer and makes
        // *us* responsible for freeing it (in Drop below, with Box::from_raw).
        let boxed = Box::new(RcBox {
            value,
            ref_count: 1,
        });
        SimpleRc {
            data: NonNull::from(Box::leak(boxed)),
        }
    }

    pub fn get(&self) -> &T {
        // SAFETY: `data` came from a Box and is only freed when the count hits
        // zero. While `self` exists the count is at least 1, so it is alive.
        unsafe { &self.data.as_ref().value }
    }

    pub fn ref_count(&self) -> usize {
        // SAFETY: as in `get`.
        unsafe { self.data.as_ref().ref_count }
    }
}

// The exercise defines an inherent `fn clone(&self)`. Implementing the Clone
// *trait* instead means SimpleRc works anywhere a `T: Clone` is expected.
impl<T> Clone for SimpleRc<T> {
    fn clone(&self) -> Self {
        // SAFETY: the box is alive (see `get`). SimpleRc is !Sync, so no other
        // thread can be touching the count at the same time.
        unsafe { (*self.data.as_ptr()).ref_count += 1 };
        SimpleRc { data: self.data }
    }
}

impl<T> Drop for SimpleRc<T> {
    fn drop(&mut self) {
        // SAFETY: the box is alive until we free it below; we are the only
        // thread that can access it (see Clone).
        unsafe {
            let inner = self.data.as_ptr();
            (*inner).ref_count -= 1;
            if (*inner).ref_count == 0 {
                // Rebuild the Box so its destructor drops `value` and frees
                // the allocation -- exactly once, by the last owner.
                drop(Box::from_raw(inner));
            }
        }
    }
}

pub fn run() {
    let rc1 = SimpleRc::new(String::from("hello"));
    println!("count after new:   {}", rc1.ref_count());
    let rc2 = rc1.clone();
    println!("count after clone: {}", rc1.ref_count());
    println!("rc1 = {}, rc2 = {}", rc1.get(), rc2.get());
    drop(rc2);
    println!("count after drop:  {}", rc1.ref_count());
}
