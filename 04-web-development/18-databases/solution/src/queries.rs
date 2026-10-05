//! Exercise 3: typed queries.
//!
//! `query_as::<_, T>` maps each row to a struct with `#[derive(FromRow)]`,
//! matching columns by name. Values always go in through `.bind(..)`: the SQL
//! and the data travel to the database *separately*, so data can never be
//! interpreted as SQL.
//!
//! SQLx 0.9 enforces this in the types: `query()` accepts a `&'static str`
//! (a literal, which can't contain user input) or a string you explicitly
//! wrap in `AssertSqlSafe(..)` -- a visible "I promise" for code review.
//!
//! (SQLx can also check queries against a real database at *compile* time
//! with `query!`/`query_as!` -- see the README. These exercises use the
//! runtime-checked functions so they build without a database.)

use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Author {
    pub id: i64,
    pub name: String,
    pub born: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Book {
    pub id: i64,
    pub author_id: i64,
    pub title: String,
    pub year: i64,
    pub copies: i64,
    pub isbn: Option<String>,
}

pub async fn add_author(
    pool: &SqlitePool,
    name: &str,
    born: Option<i64>,
) -> Result<Author, sqlx::Error> {
    // RETURNING (SQLite 3.35+, PostgreSQL) gives back the inserted row in one round trip.
    sqlx::query_as::<_, Author>(
        "INSERT INTO authors (name, born) VALUES (?, ?) RETURNING id, name, born",
    )
    .bind(name)
    .bind(born)
    .fetch_one(pool)
    .await
}

pub async fn author_by_name(pool: &SqlitePool, name: &str) -> Result<Option<Author>, sqlx::Error> {
    sqlx::query_as::<_, Author>("SELECT id, name, born FROM authors WHERE name = ?")
        .bind(name)
        .fetch_optional(pool) // Ok(None) when there's no row, instead of an error
        .await
}

pub async fn books_by_author(pool: &SqlitePool, author_id: i64) -> Result<Vec<Book>, sqlx::Error> {
    sqlx::query_as::<_, Book>("SELECT id, author_id, title, year, copies, isbn FROM books WHERE author_id = ? ORDER BY year")
        .bind(author_id)
        .fetch_all(pool)
        .await
}

/// Case-insensitive title search. The wildcard is part of the *bound value*,
/// not glued into the SQL.
pub async fn search_titles(pool: &SqlitePool, fragment: &str) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT title FROM books WHERE title LIKE ? ORDER BY title")
        .bind(format!("%{fragment}%"))
        .fetch_all(pool)
        .await
}

/// DON'T DO THIS -- kept to show why. Note that SQLx 0.9 wouldn't even
/// accept the `format!`ed string without the `AssertSqlSafe` wrapper.
///
/// With fragment = `x%' OR 1=1 --` the SQL becomes
/// `... WHERE title LIKE '%x%' OR 1=1 --%' ORDER BY title`: the quote closes
/// the string, `OR 1=1` matches every row, and `--` comments out the rest.
/// The same trick with `UNION SELECT` reads other tables.
pub async fn search_titles_unsafe(
    pool: &SqlitePool,
    fragment: &str,
) -> Result<Vec<String>, sqlx::Error> {
    let sql = format!("SELECT title FROM books WHERE title LIKE '%{fragment}%' ORDER BY title");
    sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .fetch_all(pool)
        .await
}

/// Aggregates map to tuples just as well as to structs.
pub async fn copies_per_author(pool: &SqlitePool) -> Result<Vec<(String, i64)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT a.name, COALESCE(SUM(b.copies), 0) AS copies
         FROM authors a LEFT JOIN books b ON b.author_id = a.id
         GROUP BY a.id ORDER BY a.name",
    )
    .fetch_all(pool)
    .await
}
