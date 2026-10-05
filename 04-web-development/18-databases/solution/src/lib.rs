//! Module 18 -- Databases. Reference solution: a small library database on
//! SQLite, with SQLx.
//!
//! Everything runs against an in-memory SQLite database, so there is no
//! server to install and every test starts from a clean slate. The same SQLx
//! code works against PostgreSQL or MySQL by changing the pool type and the
//! connection string (and a little SQL dialect).
//!
//! | File               | Exercise |
//! |--------------------|----------|
//! | `db.rs`            | 1, 2  connection pool, migrations, seed data |
//! | `queries.rs`       | 3  typed queries, parameters, SQL injection |
//! | `repository.rs`    | 4  the repository pattern, two implementations |
//! | `transactions.rs`  | 5  lending a book atomically |
//! | `n_plus_one.rs`    | 6  the N+1 problem and two fixes |
//! | `indexes.rs`       | 7  EXPLAIN QUERY PLAN, before and after an index |
//! | `keyset.rs`        | bonus: keyset pagination |

pub mod db;
pub mod indexes;
pub mod keyset;
pub mod n_plus_one;
pub mod queries;
pub mod repository;
pub mod transactions;
