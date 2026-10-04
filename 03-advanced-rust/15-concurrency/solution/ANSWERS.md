# Answers · 15 Concurrency

## Exercise 1

**Why does `parallel_sum` compile with `thread::scope` but not with `thread::spawn`?**

`thread::spawn` requires a `'static` closure: the spawned thread may keep
running after the function returns, so it may not borrow anything from the
function's stack. Borrowing `data` (a `&[u64]` from the caller) is rejected
with "borrowed data escapes outside of function" / "`data` does not live long
enough". `thread::scope` guarantees that every thread spawned through the scope
handle is joined before `scope` returns, so borrows that outlive the scope are
impossible — and the compiler allows non-`'static` closures.

## Exercise 3

**The naive transfer deadlock**

```rust
let mut a = self.accounts[from].lock();
let mut b = self.accounts[to].lock();
```

Thread 1 runs `transfer(0, 1)`, thread 2 runs `transfer(1, 0)`. Thread 1 locks
account 0, thread 2 locks account 1; now thread 1 waits for 1 and thread 2
waits for 0 — forever. Locking `min(from, to)` first means both threads go for
account 0 first; one gets it and the other waits *without holding anything*,
so no cycle forms.

## Exercise 4

**Why is `Relaxed` wrong for the spin lock?**

`Relaxed` guarantees the flag itself is updated atomically, but not that other
memory operations are ordered around it. Thread A could write to the protected
value and then release the flag, and thread B could acquire the flag and still
read the *old* value (the CPU or compiler may reorder the data accesses across
the flag operations). The lock would grant exclusive access to stale data.
`Release` on unlock "publishes" every write made before it; `Acquire` on lock
guarantees those writes are visible afterwards.

**What did the three counting strategies show?**

Typical release-mode numbers on a laptop: `Mutex` ~50 ms, `AtomicUsize` ~15 ms,
local sums under 1 ms for a million increments on 8 threads. The atomic is
faster than the lock but still pays for *contention*: every core fights over
the same cache line. Not sharing at all — each thread counts locally and the
results are combined once — beats both by an order of magnitude. Share less
before you try to share faster.
