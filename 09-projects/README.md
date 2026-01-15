# Projects

Hands-on projects to solidify your Rust learning across all skill levels.

## Overview

This directory contains project ideas and starter templates organized by difficulty level. Building projects is the most effective way to internalize concepts and gain real-world experience.

## Project Structure

- **[beginner](beginner/)**: Simple projects for fundamental concepts (Phase 1-2)
- **[intermediate](intermediate/)**: Complex projects requiring design (Phase 2-3)
- **[advanced](advanced/)**: Production-quality applications (Phase 4-6)
- **[expert](expert/)**: Cutting-edge, complex systems (Phase 7-8)

## Learning Through Projects

### Benefits
1. **Practical Application**: Apply theoretical knowledge
2. **Problem Solving**: Face real-world challenges
3. **Portfolio Building**: Showcase your skills
4. **Muscle Memory**: Internalize Rust idioms
5. **Debugging Practice**: Learn to troubleshoot

### Approach
1. **Start Small**: Begin with beginner projects
2. **Finish What You Start**: Completion matters
3. **Refactor**: Improve your first attempt
4. **Add Features**: Extend basic implementations
5. **Share Code**: Get feedback from community

## Beginner Projects

Perfect for Phase 1-2 (Fundamentals & Intermediate)

### 1. CLI Calculator
**Skills**: Basic syntax, error handling, user input
```
Features:
- Basic operations (+, -, *, /)
- Error handling for invalid input
- History of calculations
- Support for parentheses
```

### 2. Todo List App
**Skills**: File I/O, structs, error handling, CLI parsing
```
Features:
- Add, remove, complete tasks
- Save to JSON file
- List tasks with filters
- Due dates and priorities
```

### 3. File Organizer
**Skills**: File system operations, pattern matching
```
Features:
- Organize files by extension
- Rename files in bulk
- Move duplicates
- Dry-run mode
```

### 4. Markdown to HTML Converter
**Skills**: String parsing, pattern matching, file I/O
```
Features:
- Headers, bold, italic, links
- Code blocks
- Lists (ordered/unordered)
- Output to HTML file
```

### 5. Password Generator
**Skills**: Random number generation, CLI args
```
Features:
- Configurable length
- Character sets (letters, numbers, symbols)
- Memorable passwords
- Strength indicator
```

## Intermediate Projects

Perfect for Phase 2-3 (Intermediate & Advanced Rust)

### 1. REST API Server
**Skills**: Web frameworks, databases, async
```
Features:
- User authentication (JWT)
- CRUD operations
- Database integration (PostgreSQL)
- API documentation
- Rate limiting
```

### 2. Chat Application
**Skills**: WebSockets, concurrency, networking
```
Features:
- Multiple chat rooms
- Private messages
- User presence
- Message history
- Web UI
```

### 3. Web Scraper
**Skills**: HTTP clients, async, HTML parsing
```
Features:
- Concurrent downloads
- Respect robots.txt
- Rate limiting
- Extract structured data
- Export to CSV/JSON
```

### 4. Key-Value Store
**Skills**: Data structures, file I/O, serialization
```
Features:
- In-memory index
- Persistent storage
- Get, Set, Delete operations
- Compaction
- Simple query language
```

### 5. Image Processing Tool
**Skills**: Binary data, algorithms, parallelism
```
Features:
- Resize, crop, rotate
- Filters (blur, sharpen, grayscale)
- Batch processing
- Format conversion
- CLI and library API
```

## Advanced Projects

Perfect for Phase 4-6 (Web, Cloud-Native, Blockchain)

### 1. URL Shortener Service
**Skills**: Web APIs, databases, caching, deployment
```
Features:
- Custom short URLs
- Analytics and click tracking
- QR code generation
- API keys and rate limiting
- Redis caching
- Kubernetes deployment
```

### 2. Blog Platform
**Skills**: Full-stack web, authentication, CMS
```
Features:
- Markdown posts with images
- Comments and likes
- User profiles
- Tags and categories
- Search functionality
- Admin dashboard
```

### 3. DEX (Decentralized Exchange)
**Skills**: Solana, smart contracts, DeFi
```
Features:
- Liquidity pools
- Token swaps
- Add/remove liquidity
- Fee collection
- Price oracle integration
- Web3 frontend
```

### 4. Distributed Task Queue
**Skills**: Microservices, message queues, gRPC
```
Features:
- Worker nodes
- Job scheduling
- Retry logic
- Monitoring dashboard
- Multiple queues
- Priority scheduling
```

### 5. NFT Marketplace
**Skills**: Solana, NFTs, web development
```
Features:
- List NFTs for sale
- Buy/sell/auction
- Royalty enforcement
- Collection pages
- User profiles
- Wallet integration
```

## Expert Projects

Perfect for Phase 7-8 (Systems & Advanced Topics)

### 1. Container Runtime
**Skills**: OS concepts, Linux namespaces, cgroups
```
Features:
- Create containers
- Resource limits (CPU, memory)
- Network isolation
- Volume mounting
- Image management
```

### 2. Database Engine
**Skills**: Data structures, algorithms, persistence
```
Features:
- B-tree storage
- SQL parser
- Query optimizer
- Transaction log
- ACID guarantees
- Indexes
```

### 3. Game Engine
**Skills**: Graphics, ECS, WASM, performance
```
Features:
- 2D/3D rendering
- Physics engine
- Entity-component system
- Asset loading
- Web deployment (WASM)
```

### 4. Kubernetes Operator
**Skills**: Kubernetes, controllers, CRDs
```
Features:
- Custom Resource Definition
- Reconciliation loop
- Automatic scaling
- Health checks
- Status reporting
```

### 5. Programming Language
**Skills**: Compilers, parsers, code generation
```
Features:
- Lexer and parser
- Type checker
- LLVM codegen
- Standard library
- REPL
```

## Project Templates

Each project directory should include:

```
project-name/
├── README.md           # Project description and requirements
├── Cargo.toml         # Dependencies
├── src/
│   ├── main.rs        # Entry point
│   └── lib.rs         # Library code (if applicable)
├── tests/             # Integration tests
├── examples/          # Usage examples
└── docs/              # Additional documentation
```

## Project Checklist

For each project, aim to include:

- [ ] Clear README with setup instructions
- [ ] Comprehensive error handling
- [ ] Unit tests (>70% coverage)
- [ ] Integration tests
- [ ] Documentation comments
- [ ] CLI help message
- [ ] Example usage
- [ ] CI/CD pipeline (GitHub Actions)
- [ ] Performance benchmarks (if applicable)

## Testing Your Projects

```bash
# Run tests
cargo test

# Run with coverage
cargo tarpaulin --out Html

# Benchmark
cargo bench

# Check for issues
cargo clippy

# Format code
cargo fmt
```

## Publishing Projects

### To GitHub
```bash
git init
git add .
git commit -m "Initial commit"
git remote add origin <your-repo>
git push -u origin main
```

### To crates.io
```bash
# Update Cargo.toml with metadata
cargo login
cargo publish
```

## Getting Feedback

1. **Reddit**: r/rust, r/learnrust
2. **Discord**: Rust Community Discord
3. **GitHub**: Open issues for feedback
4. **Code Review**: Ask for reviews on forums

## Project Ideas by Domain

### Systems Programming
- Text editor
- Shell
- Version control system
- Build tool
- Package manager

### Web Development
- Social network
- E-commerce platform
- CMS
- API gateway
- Real-time collaboration tool

### Blockchain
- Blockchain from scratch
- Wallet
- DEX aggregator
- NFT minting platform
- DAO governance

### Data/ML
- Data pipeline
- Log analyzer
- Time series database
- Machine learning inference
- Data visualization tool

### Games
- Chess engine
- Roguelike
- Multiplayer game server
- Game assets pipeline

## Progression Path

1. **Week 1-4**: Complete 3 beginner projects
2. **Week 5-12**: Build 2 intermediate projects
3. **Week 13-24**: Create 2 advanced projects
4. **Week 25+**: Work on 1 expert project

## Resources

- [Project-Based Learning](https://github.com/practical-tutorials/project-based-learning#rust)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Exercism Rust Track](https://exercism.org/tracks/rust)
- [Advent of Code](https://adventofcode.com/) in Rust

## Contributing

Have a great project idea? Add it to this collection:
1. Create the project structure
2. Write a comprehensive README
3. Add starter code (optional)
4. Submit a pull request

---

**Remember**: The best way to learn is by building. Start coding today!
