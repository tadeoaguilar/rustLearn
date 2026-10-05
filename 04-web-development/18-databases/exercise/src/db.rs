//! Exercises 1 and 2: connecting, pooling, migrating.

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

/// An in-memory database.
///
/// Gotcha: every *connection* to `sqlite::memory:` gets its own, separate,
/// empty database. A pool with 5 connections would be 5 unrelated databases,
/// and a table created on one would be missing on the next. So in-memory
/// pools must have exactly one connection. (A file-backed database --
/// `sqlite://library.db?mode=rwc` -- can use many.)
///
/// `foreign_keys(true)`: SQLite doesn't enforce REFERENCES constraints unless
/// asked, per connection. SQLx enables it by default; we set it explicitly
/// because the transactions exercise depends on it.
pub async fn connect_memory() -> Result<SqlitePool, sqlx::Error> {
    todo!("Exercises 1-2")
}

/// A database file on disk, with a real pool of connections.
pub async fn connect_file(
    path: &std::path::Path,
    max_connections: u32,
) -> Result<SqlitePool, sqlx::Error> {
    todo!("Exercises 1-2")
}

/// `migrate!` embeds the files from `migrations/` into the binary at compile
/// time, so the deployed program carries its own schema history.
///
/// `migrations/0001_library.sql` is provided. Write `0002_book_isbn.sql`
/// yourself: add a nullable `isbn TEXT` column to `books`, and a UNIQUE index
/// on it that ignores NULLs (`... WHERE isbn IS NOT NULL`).
pub async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    todo!("Exercises 1-2: sqlx::migrate!(\"./migrations\").run(pool).await")
}

/// Connect + migrate + seed: what most tests and demos want.
pub async fn setup() -> SqlitePool {
    todo!("Exercises 1-2")
}

/// 3 authors, 6 books, 2 members.
pub async fn seed(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    todo!("Exercises 1-2")
}

/// Names of the user tables, to show what the migrations created.
pub async fn tables(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    todo!("Exercises 1-2")
}
