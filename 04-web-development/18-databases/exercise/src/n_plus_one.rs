//! Exercise 6: the N+1 query problem.
//!
//! "List every author with their books" written the obvious way runs one
//! query for the authors, then one more *per author*: 1 + N queries. With a
//! network round trip of 1 ms and 500 authors that's half a second before any
//! work is done. ORMs make it easy to do by accident.
//!
//! Each function returns the result *and* how many queries it ran.

use crate::queries::{Author, Book};
use sqlx::{FromRow, SqlitePool};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorWithBooks {
    pub author: String,
    pub titles: Vec<String>,
}

/// 1 + N queries.
pub async fn naive(pool: &SqlitePool) -> Result<(Vec<AuthorWithBooks>, usize), sqlx::Error> {
    todo!("Exercise 6")
}

#[derive(FromRow)]
struct Joined {
    author_id: i64,
    author: String,
    title: Option<String>, // NULL for authors with no books (LEFT JOIN)
}

/// Fix 1: one JOIN. Author columns repeat on every row; we group in Rust.
pub async fn joined(pool: &SqlitePool) -> Result<(Vec<AuthorWithBooks>, usize), sqlx::Error> {
    todo!("Exercise 6")
}

/// Fix 2: two queries -- the parents, then every child at once with `IN`.
/// What most ORMs do for "eager loading". No row duplication, and it works
/// when the two tables live in different services.
pub async fn batched(pool: &SqlitePool) -> Result<(Vec<AuthorWithBooks>, usize), sqlx::Error> {
    todo!("Exercise 6")
}
