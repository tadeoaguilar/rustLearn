# Phase 4: Web Development

Build production-ready web applications and APIs with Rust.

## Overview

This phase focuses on building web services, REST APIs, and full-stack applications using Rust's powerful web frameworks. You'll learn to build scalable, performant web backends with database integration, authentication, and real-time communication.

**Duration**: 8-10 weeks
**Difficulty**: Advanced
**Prerequisites**: Completion of Phase 3 (Advanced Rust), especially async/await

## Modules

### [16-web-frameworks](16-web-frameworks/)

**Learning Objectives**:
- Master modern Rust web frameworks
- Compare Axum, Actix-web, and Rocket
- Handle HTTP requests and responses
- Implement routing and middleware
- Serve static files and templates

**Key Topics**:
- **Axum**: Tower-based, type-safe handlers
- **Actix-web**: Actor-based, high performance
- **Rocket**: Developer-friendly, batteries included
- Request handlers and extractors
- Routing and path parameters
- Middleware and layers
- State management
- Request/response types
- Form handling
- Template engines (askama, tera, handlebars)
- Static file serving

**Exercises**:
1. Build a simple REST API in each framework
2. Create a blog with templates
3. Implement custom middleware
4. Compare performance benchmarks

**Resources**:
- [Axum Docs](https://docs.rs/axum/)
- [Actix-web Guide](https://actix.rs/)
- [Rocket Guide](https://rocket.rs/guide/)

---

### [17-rest-apis](17-rest-apis/)

**Learning Objectives**:
- Design RESTful APIs
- Implement CRUD operations
- Handle request validation
- Generate OpenAPI documentation
- Implement API versioning

**Key Topics**:
- REST principles and best practices
- HTTP methods and status codes
- Request validation with validator
- Serialization with serde
- OpenAPI/Swagger with utoipa
- API versioning strategies
- Rate limiting
- CORS configuration
- Content negotiation
- Pagination patterns
- Error response standards

**Exercises**:
1. Build a complete CRUD API
2. Add OpenAPI documentation
3. Implement rate limiting
4. Create an API client SDK

**Resources**:
- [serde](https://serde.rs/)
- [utoipa](https://docs.rs/utoipa/) - OpenAPI generation
- [validator](https://docs.rs/validator/)

---

### [18-databases](18-databases/)

**Learning Objectives**:
- Work with SQL databases
- Use ORMs and query builders
- Implement migrations
- Optimize database queries
- Handle connection pooling

**Key Topics**:
- **SQLx**: Async, compile-time checked SQL
- **Diesel**: Sync ORM with type-safe queries
- **SeaORM**: Async ORM with migrations
- PostgreSQL integration
- MySQL/MariaDB support
- SQLite for embedded use
- Database migrations
- Connection pooling (bb8, deadpool)
- Transactions and ACID
- Query optimization
- N+1 query problem

**Exercises**:
1. Implement a data layer with SQLx
2. Create migrations for a schema
3. Build a repository pattern
4. Optimize slow queries

**Resources**:
- [SQLx](https://docs.rs/sqlx/)
- [Diesel](https://diesel.rs/)
- [SeaORM](https://www.sea-ql.org/SeaORM/)

---

### [19-authentication](19-authentication/)

**Learning Objectives**:
- Implement authentication flows
- Use JWT tokens
- Handle OAuth2 integration
- Manage sessions securely
- Implement authorization

**Key Topics**:
- Password hashing with argon2
- JWT creation and validation
- OAuth2 flows (authorization code, client credentials)
- Session management
- Cookie security (HttpOnly, Secure, SameSite)
- CSRF protection
- Role-based access control (RBAC)
- API key authentication
- Multi-factor authentication (MFA)
- Security best practices

**Exercises**:
1. Build a registration/login system
2. Implement JWT authentication
3. Add OAuth2 (Google, GitHub)
4. Create role-based permissions

**Resources**:
- [jsonwebtoken](https://docs.rs/jsonwebtoken/)
- [oauth2](https://docs.rs/oauth2/)
- [argon2](https://docs.rs/argon2/)

---

### [20-websockets](20-websockets/)

**Learning Objectives**:
- Implement WebSocket servers
- Handle real-time bidirectional communication
- Broadcast to multiple clients
- Integrate WebSockets with web frameworks
- Handle connection lifecycle

**Key Topics**:
- WebSocket protocol basics
- axum WebSocket support
- tokio-tungstenite for WebSockets
- Connection upgrading
- Message framing
- Broadcasting patterns
- Room/channel concepts
- Heartbeat/ping-pong
- Reconnection strategies
- Scaling WebSocket servers

**Exercises**:
1. Build a chat server
2. Implement real-time notifications
3. Create a collaborative editor
4. Build a live dashboard

**Resources**:
- [tokio-tungstenite](https://docs.rs/tokio-tungstenite/)
- [Axum WebSocket Example](https://github.com/tokio-rs/axum/tree/main/examples/websockets)

---

## Phase Completion Checklist

- [ ] Built multiple REST APIs with different frameworks
- [ ] Integrated SQL databases with connection pooling
- [ ] Implemented complete authentication system
- [ ] Created WebSocket real-time features
- [ ] Generated OpenAPI documentation
- [ ] Deployed a web application to production
- [ ] Completed at least 2 full-stack projects

## Projects

Build these projects to master web development:

1. **Blog Platform**: Posts, comments, authentication, markdown
2. **Task Management API**: CRUD, teams, permissions, real-time updates
3. **E-commerce Backend**: Products, cart, orders, payments, admin panel
4. **Real-time Chat Application**: Multiple rooms, private messages, notifications

## Production Checklist

Before deploying to production:
- [ ] Implement proper error handling
- [ ] Add request logging and monitoring
- [ ] Configure CORS properly
- [ ] Set up rate limiting
- [ ] Implement health check endpoints
- [ ] Add database migrations
- [ ] Configure environment-based settings
- [ ] Set up SSL/TLS
- [ ] Implement graceful shutdown
- [ ] Add comprehensive tests

## Performance Tips

1. **Use connection pooling**: bb8 or deadpool
2. **Cache frequently accessed data**: Redis, in-memory caches
3. **Optimize database queries**: Indexes, EXPLAIN ANALYZE
4. **Use async everywhere**: Non-blocking I/O
5. **Profile and benchmark**: cargo flamegraph, criterion

## Next Steps

After completing this phase:
- Move to [05-cloud-native](../05-cloud-native/) for deployment and scaling
- Build a production-grade SaaS application
- Contribute to web framework ecosystems
- Learn GraphQL with async-graphql

## Additional Resources

- [Zero to Production in Rust](https://www.zero2prod.com/)
- [Rust Web Development](https://www.manning.com/books/rust-web-development)
- [Are We Web Yet?](https://www.arewewebyet.org/)
- [Shuttle.rs](https://www.shuttle.rs/) - Deploy Rust apps easily
