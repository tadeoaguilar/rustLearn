# Phase 3: Advanced Rust

Deep dive into advanced Rust features including async programming, macros, and low-level concepts.

## Overview

This phase covers advanced Rust topics that enable you to build high-performance, concurrent applications. You'll master async programming, macro systems, unsafe code, and advanced concurrency patterns.

**Duration**: 6-8 weeks
**Difficulty**: Advanced
**Prerequisites**: Completion of Phase 2 (Intermediate Rust)

## Modules

### [11-lifetimes-advanced](11-lifetimes-advanced/)

**Learning Objectives**:
- Master lifetime annotations
- Understand lifetime elision rules
- Use higher-ranked trait bounds (HRTB)
- Handle complex lifetime scenarios
- Understand the 'static lifetime

**Key Topics**:
- Lifetime annotation syntax
- Lifetime elision rules
- Generic lifetimes in structs
- Lifetime bounds on traits
- Higher-ranked trait bounds (for<'a>)
- 'static lifetime
- Lifetime subtyping and variance
- Common lifetime patterns
- Debugging lifetime errors

**Exercises**:
1. Create a struct that borrows data with lifetimes
2. Implement a parser with lifetime-annotated types
3. Build an iterator that borrows from a collection
4. Solve complex lifetime compilation errors

**Resources**:
- [Rust Book Chapter 10.3](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)
- [Rustonomicon Lifetimes](https://doc.rust-lang.org/nomicon/lifetimes.html)

---

### [12-async-await](12-async-await/)

**Learning Objectives**:
- Understand async/await fundamentals
- Work with Tokio runtime
- Handle async I/O operations
- Combine futures and streams
- Implement async traits

**Key Topics**:
- Async/await syntax
- Futures and the Future trait
- Tokio runtime and executors
- Async I/O with tokio::fs and tokio::net
- Streams and StreamExt
- Select and join macros
- Channels: mpsc, oneshot, broadcast
- Async traits with async-trait
- Pinning and Unpin
- Cancellation and timeouts
- Error handling in async code

**Exercises**:
1. Build an async HTTP client
2. Create a concurrent file downloader
3. Implement an async task queue
4. Build a simple async web server

**Resources**:
- [Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [async-trait](https://docs.rs/async-trait/)

---

### [13-macros](13-macros/)

**Learning Objectives**:
- Write declarative macros
- Create procedural macros
- Implement derive macros
- Build attribute macros
- Understand macro hygiene

**Key Topics**:
- Declarative macros with macro_rules!
- Macro patterns and repetitions
- Procedural macros overview
- Derive macros with syn and quote
- Attribute macros
- Function-like procedural macros
- TokenStream manipulation
- Macro hygiene and span
- Debugging macros with cargo expand

**Exercises**:
1. Create a macro for compile-time validation
2. Implement a custom derive macro
3. Build a DSL with declarative macros
4. Create a builder pattern derive macro

**Resources**:
- [Rust Book Chapter 19.6](https://doc.rust-lang.org/book/ch19-06-macros.html)
- [The Little Book of Rust Macros](https://veykril.github.io/tlborm/)
- [syn](https://docs.rs/syn/) and [quote](https://docs.rs/quote/)

---

### [14-unsafe-ffi](14-unsafe-ffi/)

**Learning Objectives**:
- Understand when and how to use unsafe
- Interface with C libraries (FFI)
- Work with raw pointers
- Understand memory layout
- Maintain safety invariants

**Key Topics**:
- Unsafe Rust fundamentals
- Raw pointers (*const T, *mut T)
- Dereferencing raw pointers
- Calling unsafe functions
- FFI basics (extern "C")
- Using bindgen for C bindings
- Creating safe abstractions over unsafe code
- Memory layout and representation
- Undefined behavior (UB) pitfalls
- ABI compatibility

**Exercises**:
1. Create Rust bindings for a C library
2. Implement a custom allocator
3. Build a safe wrapper around unsafe code
4. Interface with system libraries

**Resources**:
- [Rustonomicon](https://doc.rust-lang.org/nomicon/)
- [bindgen](https://rust-lang.github.io/rust-bindgen/)
- [FFI Guide](https://doc.rust-lang.org/nomicon/ffi.html)

---

### [15-concurrency](15-concurrency/)

**Learning Objectives**:
- Master thread-based concurrency
- Use sync primitives effectively
- Implement parallel algorithms
- Avoid data races and deadlocks
- Understand memory ordering

**Key Topics**:
- Thread creation and joining
- Message passing with channels
- Shared state with Arc<Mutex<T>>
- RwLock for read-heavy workloads
- Atomic types and ordering
- Rayon for data parallelism
- crossbeam utilities
- Lock-free data structures
- Thread pools
- Deadlock prevention
- Send and Sync traits

**Exercises**:
1. Build a parallel file processor
2. Implement a concurrent web crawler
3. Create a thread pool
4. Build a lock-free queue

**Resources**:
- [Rust Book Chapter 16](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [Rayon](https://docs.rs/rayon/)
- [crossbeam](https://docs.rs/crossbeam/)

---

## Phase Completion Checklist

- [ ] Can write async code with Tokio
- [ ] Understand lifetimes and can solve complex scenarios
- [ ] Created custom macros
- [ ] Comfortable with unsafe Rust when necessary
- [ ] Master concurrent and parallel programming
- [ ] Understand the difference between async and threads
- [ ] Completed at least 2 advanced projects

## Projects

Build these projects to master advanced concepts:

1. **Async Web Scraper**: Concurrent downloads, rate limiting, parsing
2. **Mini Redis Clone**: Async server, protocol parsing, persistence
3. **Custom Derive Macro**: Build a serialization derive macro
4. **Thread Pool Executor**: Work-stealing thread pool implementation

## Next Steps

After completing this phase, choose your specialization:
- [04-web-development](../04-web-development/) - Web services and APIs
- [05-cloud-native](../05-cloud-native/) - Cloud infrastructure
- [06-blockchain-solana](../06-blockchain-solana/) - Blockchain development
- [07-systems-programming](../07-systems-programming/) - Systems and tools

## Additional Resources

- [Async Book](https://rust-lang.github.io/async-book/)
- [Rustonomicon](https://doc.rust-lang.org/nomicon/)
- [Jon Gjengset's YouTube](https://www.youtube.com/c/JonGjengset)
- [Rust Atomics and Locks](https://marabos.nl/atomics/)
