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
    let options = SqliteConnectOptions::from_str("sqlite::memory:")?.foreign_keys(true);
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
}

/// A database file on disk, with a real pool of connections.
pub async fn connect_file(
    path: &std::path::Path,
    max_connections: u32,
) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .foreign_keys(true);
    SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect_with(options)
        .await
}

/// `migrate!` embeds the files from `migrations/` into the binary at compile
/// time, so the deployed program carries its own schema history.
pub async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

/// Connect + migrate + seed: what most tests and demos want.
pub async fn setup() -> SqlitePool {
    let pool = connect_memory().await.expect("in-memory database");
    migrate(&pool).await.expect("migrations apply");
    seed(&pool).await.expect("seed data inserts");
    pool
}

/// 3 authors, 6 books, 2 members.
pub async fn seed(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    for (name, born) in [
        ("Ursula K. Le Guin", 1929),
        ("Terry Pratchett", 1948),
        ("Octavia E. Butler", 1947),
    ] {
        sqlx::query("INSERT INTO authors (name, born) VALUES (?, ?)")
            .bind(name)
            .bind(born)
            .execute(pool)
            .await?;
    }
    for (author, title, year, copies) in [
        (1, "A Wizard of Earthsea", 1968, 2),
        (1, "The Left Hand of Darkness", 1969, 1),
        (1, "The Dispossessed", 1974, 0),
        (2, "Guards! Guards!", 1989, 3),
        (2, "Small Gods", 1992, 1),
        (3, "Kindred", 1979, 2),
    ] {
        sqlx::query("INSERT INTO books (author_id, title, year, copies) VALUES (?, ?, ?, ?)")
            .bind(author)
            .bind(title)
            .bind(year)
            .bind(copies)
            .execute(pool)
            .await?;
    }
    for (name, email) in [("Ann", "ann@example.com"), ("Bob", "bob@example.com")] {
        sqlx::query("INSERT INTO members (name, email) VALUES (?, ?)")
            .bind(name)
            .bind(email)
            .execute(pool)
            .await?;
    }
    Ok(())
}

/// Names of the user tables, to show what the migrations created.
pub async fn tables(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .fetch_all(pool)
        .await
}
