# Phase 2: Intermediate Rust

Build a solid foundation in Rust's type system, idioms, and project organization.

## Overview

This phase deepens your understanding of Rust's powerful type system, teaches you functional programming patterns, and introduces you to professional Rust development practices including testing, project organization, and memory management patterns.

**Duration**: 6-8 weeks
**Difficulty**: Intermediate
**Prerequisites**: Completion of Phase 1 (Rust Fundamentals)

## Modules

### [06-traits-generics](06-traits-generics/)

**Learning Objectives**:
- Master traits and trait bounds
- Write generic functions and types
- Understand trait objects and dynamic dispatch
- Use associated types
- Implement operator overloading

**Key Topics**:
- Defining and implementing traits
- Default trait implementations
- Trait bounds and where clauses
- Generic functions and structs
- Trait objects (dyn Trait)
- Static vs dynamic dispatch
- Associated types vs generic parameters
- Operator overloading (Add, Display, etc.)
- Supertraits and trait inheritance
- Blanket implementations

**Exercises**:
1. Create a Shape trait with multiple implementations
2. Build a generic container type
3. Implement Display and Debug for custom types
4. Create a plugin system using trait objects

**Resources**:
- [Rust Book Chapter 10](https://doc.rust-lang.org/book/ch10-00-generics.html)
- [Trait Objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)

---

### [07-collections-iterators](07-collections-iterators/)

**Learning Objectives**:
- Master standard library collections
- Write functional code with iterators
- Understand iterator adaptors and consumers
- Use closures effectively
- Optimize collection performance

**Key Topics**:
- Vec<T>: Dynamic arrays
- HashMap<K, V> and HashSet<T>
- BTreeMap and BTreeSet
- VecDeque for queues
- Closures: Fn, FnMut, FnOnce
- Iterator trait and iteration
- Iterator adaptors: map, filter, fold, etc.
- Consuming adaptors: collect, sum, any, all
- Custom iterators
- Lazy evaluation

**Exercises**:
1. Implement a custom collection type
2. Build a data processing pipeline with iterators
3. Create an iterator for Fibonacci sequence
4. Solve algorithm problems using functional patterns

**Resources**:
- [Rust Book Chapter 13](https://doc.rust-lang.org/book/ch13-00-functional-features.html)
- [Iterator Trait Docs](https://doc.rust-lang.org/std/iter/trait.Iterator.html)

---

### [08-modules-crates](08-modules-crates/)

**Learning Objectives**:
- Organize code with modules
- Manage dependencies with Cargo
- Create and publish crates
- Use workspaces for multi-crate projects
- Understand visibility and privacy

**Key Topics**:
- Module system: mod, use, pub
- File-based module organization
- Nested modules and re-exports
- Cargo.toml configuration
- Dependency management
- Semantic versioning
- Workspaces for monorepos
- Features and conditional compilation
- Publishing to crates.io
- Documentation comments (///)

**Exercises**:
1. Refactor a large file into modules
2. Create a library crate with public API
3. Build a workspace with multiple crates
4. Publish a crate to crates.io (optional)

**Resources**:
- [Rust Book Chapter 7](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)
- [Cargo Book](https://doc.rust-lang.org/cargo/)

---

### [09-testing](09-testing/)

**Learning Objectives**:
- Write effective unit tests
- Create integration tests
- Use documentation tests
- Mock dependencies
- Measure code coverage
- Benchmark performance

**Key Topics**:
- Unit tests with #[test]
- Test organization (tests module)
- Integration tests (tests/ directory)
- Documentation tests
- Test assertions and should_panic
- Testing private functions
- Test organization patterns
- Mocking with mockall or mockito
- Property-based testing with proptest
- Benchmarking with criterion
- Code coverage with tarpaulin

**Exercises**:
1. Add comprehensive tests to a library
2. Write integration tests for an API
3. Create documentation examples that test
4. Benchmark different implementations

**Resources**:
- [Rust Book Chapter 11](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Criterion.rs](https://github.com/bheisler/criterion.rs)

---

### [10-smart-pointers](10-smart-pointers/)

**Learning Objectives**:
- Understand Rust's pointer types
- Use Box for heap allocation
- Share ownership with Rc and Arc
- Enable interior mutability with RefCell and Mutex
- Prevent memory leaks with Weak

**Key Topics**:
- Box<T> for heap allocation
- Deref and Drop traits
- Rc<T> for reference counting
- Arc<T> for atomic reference counting
- RefCell<T> for interior mutability
- Mutex<T> and RwLock<T>
- Weak<T> to prevent cycles
- Cell<T> for Copy types
- Combining smart pointers (Rc<RefCell<T>>)
- When to use each pointer type

**Exercises**:
1. Implement a linked list with Box
2. Build a tree structure with Rc and Weak
3. Create a graph with shared ownership
4. Implement a simple cache with Arc and Mutex

**Resources**:
- [Rust Book Chapter 15](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html)
- [Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/)

---

## Phase Completion Checklist

- [ ] Can write and use generic types and functions
- [ ] Understand and implement traits effectively
- [ ] Use iterators and closures fluently
- [ ] Organize code into modules and crates
- [ ] Write comprehensive tests
- [ ] Choose appropriate smart pointers for different scenarios
- [ ] Completed at least 2 intermediate projects

## Projects

Build these projects to master intermediate concepts:

1. **JSON Parser**: Implement a JSON parser using enums and traits
2. **Data Query Engine**: Build a SQL-like query engine with iterators
3. **Caching Proxy**: Implement a caching layer with Arc and Mutex
4. **Plugin System**: Create an extensible plugin architecture with traits

## Next Steps

After completing this phase, you should:
- Move on to [03-advanced-rust](../03-advanced-rust/)
- Contribute to an open-source Rust project
- Solve medium difficulty problems on LeetCode in Rust
- Read source code of popular crates (serde, tokio, etc.)

## Additional Resources

- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)
- [API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Effective Rust](https://www.lurklurk.org/effective-rust/)
- [Crate Comparison](https://blessed.rs/)
