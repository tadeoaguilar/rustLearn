//! Exercise 7: indexes and query plans.
//!
//! `EXPLAIN QUERY PLAN` shows how SQLite will run a query without running it.
//! `SCAN books` reads every row; `SEARCH books USING INDEX ...` jumps to the
//! matching ones. On 6 rows nobody notices; on 6 million it's the difference
//! between a millisecond and seconds. (PostgreSQL: `EXPLAIN ANALYZE`.)

use sqlx::{Row, SqlitePool};

/// The plan's `detail` lines for a `&'static str` query with one bound value.
pub async fn query_plan(
    pool: &SqlitePool,
    sql: &'static str,
    value: i64,
) -> Result<Vec<String>, sqlx::Error> {
    let explain = format!("EXPLAIN QUERY PLAN {sql}");
    let rows = sqlx::query(sqlx::AssertSqlSafe(explain))
        .bind(value)
        .fetch_all(pool)
        .await?;
    Ok(rows.iter().map(|r| r.get::<String, _>("detail")).collect())
}

pub const BOOKS_BY_YEAR: &str = "SELECT title FROM books WHERE year = ?";
pub const BOOKS_BY_AUTHOR: &str = "SELECT title FROM books WHERE author_id = ?";

/// Adds the indexes the two queries above need. Foreign key columns are a
/// classic omission: SQLite and PostgreSQL do NOT index them automatically.
pub async fn add_indexes(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_books_year ON books(year)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_books_author ON books(author_id)")
        .execute(pool)
        .await?;
    Ok(())
}

/// Inserts `n` extra books, for timing comparisons.
pub async fn bulk_insert_books(pool: &SqlitePool, n: i64) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?; // one transaction: thousands of times faster than n autocommits
    for i in 0..n {
        sqlx::query("INSERT INTO books (author_id, title, year, copies) VALUES (?, ?, ?, 1)")
            .bind(1 + i % 3)
            .bind(format!("Generated {i}"))
            .bind(1900 + i % 120)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await
}
