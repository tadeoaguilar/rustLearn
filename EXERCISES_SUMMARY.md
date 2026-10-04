# Exercises Summary - Complete Learning Path

This document provides an overview of all exercises across the 40 modules in the Rust learning path.

## Status

Phases 1–3 have exercises **and** runnable code: for every module an
`exercise/` crate to fill in, a `solution/` crate, and a `tests/` crate that
checks either one. See [GETTING_STARTED.md](GETTING_STARTED.md) for the
commands.

| Phase | Exercises | Code + tests |
|---|---|---|
| 1 · Rust Fundamentals (01–05) | ✅ | ✅ 120 tests |
| 2 · Intermediate Rust (06–10) | ✅ | ✅ 88 tests + 47 in module 09's solution + 8 doc tests in module 08 |
| 3 · Advanced Rust (11–15) | ✅ | ✅ 116 tests + 12 macro doc tests |
| 4–8 · Specialisations | topic outlines below | — |

## Phase 1: Rust Fundamentals ✅

### 01-getting-started
- ✅ Hello, Rust! (10 min)
- ✅ Cargo Basics (15 min)
- ✅ Documentation Explorer (20 min)
- ✅ Number Guessing Game (45 min)
- ✅ Project Setup Best Practices (20 min)
- ✅ Debugging Practice (30 min)
- ✅ Environment Setup (15 min)
- ✅ Bonus: Cargo Extensions (60 min)

### 02-basic-syntax
- ✅ Variables and Mutability (15 min)
- ✅ Data Types (20 min)
- ✅ Functions (30 min)
- ✅ Control Flow - If/Else (25 min)
- ✅ Loops (35 min)
- ✅ Pattern Matching (30 min)
- ✅ Calculator Program (45 min)
- ✅ FizzBuzz (20 min)
- ✅ Temperature Converter (25 min)
- ✅ Bonus: Prime Number Generator (60 min)

### 03-ownership-borrowing
- ✅ Understanding Ownership (20 min)
- ✅ References and Borrowing (30 min)
- ✅ String vs &str (25 min)
- ✅ Dangling References (20 min)
- ✅ Clone vs Copy (30 min)
- ✅ Mutable vs Immutable Borrow (35 min)
- ✅ Real-World: Text Buffer (45 min)
- ✅ Ownership in Collections (30 min)
- ✅ Bonus: Reference Counting Simulation (60+ min)

### 04-structs-enums
- ✅ Basic Structs (20 min)
- ✅ Tuple Structs and Unit Structs (15 min)
- ✅ Methods and Associated Functions (35 min)
- ✅ Basic Enums (20 min)
- ✅ Option<T> (30 min)
- ✅ Pattern Matching (35 min)
- ✅ Result<T, E> (30 min)
- ✅ Complex Enum - Game State (60 min)
- ✅ Shape Calculator (40 min)
- ✅ Bonus: JSON-like Data Structure (90+ min)

### 05-error-handling
- ✅ Panic Basics (15 min)
- ✅ Result Basics (20 min)
- ✅ The ? Operator (25 min)
- ✅ Custom Error Types (35 min)
- ✅ Converting Between Error Types (40 min)
- ✅ Option to Result Conversion (20 min)
- ✅ Error Recovery (30 min)
- ✅ File Configuration Parser (60 min)
- ✅ Using thiserror Crate (25 min)
- ✅ Bonus: Result Combinators (45 min)

## Phase 2: Intermediate Rust ✅

### 06-traits-generics
- ✅ Basic Traits (25 min)
- ✅ Generic Functions (30 min)
- ✅ Trait Bounds (35 min)
- ✅ Operator Overloading (30 min)
- ✅ Associated Types (40 min)
- ✅ Trait Objects (45 min)
- ✅ Generic Data Structures (60 min)
- ✅ Bonus: Shape System (90+ min)

### 07-collections-iterators
- ✅ Vector Operations (20 min)
- ✅ HashMap Basics — word frequency, grouping, LRU cache (30 min)
- ✅ Iterator Basics (35 min)
- ✅ Advanced Iterators — fold, scan, zip, flat_map, partition (45 min)
- ✅ Custom Iterators — range, cycle, primes (60 min)
- ✅ Closures — Fn, FnMut, FnOnce, composition (35 min)
- ✅ Bonus: Data Pipeline (90+ min)

### 08-modules-crates *(exercises written for this repo)*
- ✅ Refactor a Large File into Modules (30 min)
- ✅ Visibility and Encapsulation (40 min)
- ✅ Re-exports and a Prelude (20 min)
- ✅ Features and Conditional Compilation (40 min)
- ✅ A Workspace with Multiple Crates (45 min)
- ✅ Documentation and Doc Tests (30 min)
- ✅ Publishing (optional, 30 min)
- ✅ Bonus: Feature-Gated Inventory Report (45 min)

### 09-testing *(exercises written for this repo)*
- ✅ Unit Tests (30 min)
- ✅ Testing Errors and Panics (35 min)
- ✅ Integration Tests and Test Organisation (30 min)
- ✅ Documentation Tests (25 min)
- ✅ Mocking with mockall (45 min)
- ✅ Property-Based Testing with proptest (45 min)
- ✅ Benchmarking with criterion (30 min)
- ✅ Bonus: Coverage and the Bowling TDD Kata (60 min)

### 10-smart-pointers *(exercises written for this repo)*
- ✅ Box and Recursive Types (25 min)
- ✅ A Linked List with Box (60 min)
- ✅ Deref and Drop (35 min)
- ✅ Rc and Weak — a Tree with Parent Links (60 min)
- ✅ Interior Mutability with RefCell and Cell (40 min)
- ✅ Shared Mutable Graphs with Rc<RefCell<T>> (50 min)
- ✅ A Thread-Safe Cache with Arc and Mutex (60 min)
- ✅ Bonus: Cow (30 min)

## Phase 3: Advanced Rust ✅

### 11-lifetimes-advanced *(exercises written for this repo)*
- ✅ Annotations and Elision (30 min)
- ✅ Structs That Borrow (40 min)
- ✅ A Zero-Copy HTTP Parser (60 min)
- ✅ Iterators That Borrow (60 min)
- ✅ 'static and Lifetime Bounds (40 min)
- ✅ Higher-Ranked Trait Bounds (40 min)
- ✅ Fix the Lifetime Errors (60 min)
- ✅ Bonus: StrSplit with Two Lifetimes (60 min)

### 12-async-await
- ✅ Basic Async Functions (20 min)
- ✅ Concurrent Execution (30 min)
- ✅ Async File I/O (35 min)
- ✅ Async HTTP Client (40 min)
- ✅ Channels (35 min)
- ✅ Spawning Tasks (40 min)
- ✅ Async Web Server (60 min)
- ✅ Streams (45 min)
- ✅ Bonus: Async Task Queue (90+ min)

### 13-macros *(exercises written for this repo)*
- ✅ Your First macro_rules! (30 min)
- ✅ Generating Items (40 min)
- ✅ Compile-Time Validation (40 min)
- ✅ A State-Machine DSL (60 min)
- ✅ Hygiene, $crate and Debugging (30 min)
- ✅ A Derive Macro (60 min)
- ✅ A Builder Derive (90 min)
- ✅ Bonus: An Attribute Macro (45 min)

### 14-unsafe-ffi *(exercises written for this repo)*
- ✅ Raw Pointers (40 min)
- ✅ Calling the C Standard Library (40 min)
- ✅ Bindings to Your Own C Library (60 min)
- ✅ Calling Rust from C (30 min)
- ✅ A Safe Abstraction: StackVec (60 min)
- ✅ A Custom Allocator and an Arena (45 min)
- ✅ System Libraries (30 min)
- ✅ Bonus: Hunting UB with Miri (45 min)

### 15-concurrency *(exercises written for this repo)*
- ✅ Threads and Scoped Threads (30 min)
- ✅ Message Passing (40 min)
- ✅ Shared State: Mutex, RwLock, Condvar (60 min)
- ✅ Atomics and Memory Ordering (45 min)
- ✅ A Thread Pool (60 min)
- ✅ Data Parallelism with Rayon (40 min)
- ✅ A Concurrent Web Crawler (90 min)
- ✅ A Lock-Free Stack (90 min)
- ✅ Bonus: Dining Philosophers (45 min)

## Phase 4: Web Development

### 16-web-frameworks
- Axum "Hello World"
- Routing and handlers
- Middleware
- State management
- Template rendering

### 17-rest-apis
- CRUD endpoints
- Request validation
- Error responses
- OpenAPI docs
- Versioning

### 18-databases
- SQLx setup
- CRUD operations
- Migrations
- Connection pooling
- Transactions

### 19-authentication
- Password hashing
- JWT tokens
- OAuth2 integration
- Session management
- RBAC

### 20-websockets
- WebSocket server
- Broadcasting
- Rooms/channels
- Reconnection
- Real-time chat

## Phase 5: Cloud Native

### 21-containerization
- Dockerfile for Rust
- Multi-stage builds
- Alpine images
- Docker Compose
- Health checks

### 22-kubernetes-operators
- CRD definition
- Controller pattern
- kube-rs basics
- Reconciliation loop
- Testing operators

### 23-microservices
- gRPC with Tonic
- Service discovery
- Circuit breakers
- Message queues
- Event-driven arch

### 24-observability
- Structured logging
- Prometheus metrics
- Distributed tracing
- Dashboards
- Alerting

### 25-service-mesh
- Linkerd setup
- mTLS configuration
- Traffic management
- Canary deployments
- Policy enforcement

## Phase 6: Blockchain & Solana

### 26-solana-basics
- Wallet setup
- Account model
- Transactions
- PDAs
- Deploy program

### 27-anchor-framework
- Anchor project setup
- Account validation
- Instruction handlers
- Testing
- IDL generation

### 28-smart-contracts
- Counter program
- Voting system
- Escrow
- Security patterns
- Upgrades

### 29-nfts-tokens
- Create token
- Mint NFTs
- Token metadata
- ATAs
- Token-2022

### 30-defi-protocols
- Simple AMM
- Liquidity pools
- Lending protocol
- Oracle integration
- Staking

## Phase 7: Systems Programming

### 31-cli-tools
- Argument parsing
- TUI with ratatui
- Progress bars
- Signal handling
- Shell completion

### 32-network-programming
- TCP server/client
- UDP communication
- Protocol parsing
- TLS with rustls
- Proxy server

### 33-embedded-rust
- no_std basics
- LED blink
- I2C/SPI
- Interrupts
- Embassy async

### 34-os-concepts
- Process management
- IPC mechanisms
- Memory mapping
- System calls
- Signals

### 35-memory-management
- Custom allocator
- Memory pools
- Profiling
- Cache optimization
- NUMA awareness

## Phase 8: Advanced Topics

### 36-performance-optimization
- Profiling with flamegraph
- Benchmarking
- SIMD operations
- LTO and PGO
- Binary size reduction

### 37-wasm
- WASM basics
- wasm-bindgen
- Yew application
- JS interop
- WASI

### 38-proc-macros
- Builder derive
- Custom serialization
- Attribute macros
- Function-like macros
- Error reporting

### 39-compiler-internals
- Reading MIR
- Custom clippy lint
- Understanding borrow checker
- Contributing to rustc

### 40-contributing
- Finding projects
- Writing PRs
- Code review
- RFC process
- Community engagement

## Exercise Difficulty Guide

- 🟢 **Easy**: 10-20 minutes, fundamental concepts
- 🟡 **Medium**: 25-45 minutes, requires thinking
- 🔴 **Hard**: 60+ minutes, complex implementation
- 🟣 **Bonus**: 90+ minutes, advanced challenges

## How to Use This Guide

1. **Sequential Learning**: Follow modules in order
2. **Complete All Exercises**: Don't skip basics
3. **Build Projects**: Apply concepts immediately
4. **Review Solutions**: Compare approaches
5. **Challenge Yourself**: Attempt bonus exercises
6. **Track Progress**: Use PROGRESS.md template

## Estimated Time Investment

- **Phase 1** (Fundamentals): 15-20 hours
- **Phase 2** (Intermediate): 20-25 hours
- **Phase 3** (Advanced): 25-30 hours
- **Phase 4** (Web Dev): 30-40 hours
- **Phase 5** (Cloud Native): 30-40 hours
- **Phase 6** (Blockchain): 40-50 hours
- **Phase 7** (Systems): 25-30 hours
- **Phase 8** (Advanced Topics): 30-40 hours

**Total**: 215-275 hours of hands-on practice

## Next Steps

Each completed exercise file includes:
- Clear objectives
- Step-by-step tasks
- Code templates
- Expected outputs
- Hints and tips
- Bonus challenges
- Resources

Check individual exercise.md files in each module directory for complete details.
