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
    todo!("Exercise 7")
}

pub const BOOKS_BY_YEAR: &str = "SELECT title FROM books WHERE year = ?";
pub const BOOKS_BY_AUTHOR: &str = "SELECT title FROM books WHERE author_id = ?";

/// Adds the indexes the two queries above need. Foreign key columns are a
/// classic omission: SQLite and PostgreSQL do NOT index them automatically.
pub async fn add_indexes(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    todo!("Exercise 7")
}

/// Inserts `n` extra books, for timing comparisons.
pub async fn bulk_insert_books(pool: &SqlitePool, n: i64) -> Result<(), sqlx::Error> {
    todo!("Exercise 7")
}
