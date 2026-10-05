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
    let mut queries = 1;
    let authors: Vec<Author> = sqlx::query_as("SELECT id, name, born FROM authors ORDER BY id")
        .fetch_all(pool)
        .await?;
    let mut out = Vec::new();
    for a in authors {
        queries += 1;
        let titles: Vec<String> =
            sqlx::query_scalar("SELECT title FROM books WHERE author_id = ? ORDER BY id")
                .bind(a.id)
                .fetch_all(pool)
                .await?;
        out.push(AuthorWithBooks {
            author: a.name,
            titles,
        });
    }
    Ok((out, queries))
}

#[derive(FromRow)]
struct Joined {
    author_id: i64,
    author: String,
    title: Option<String>, // NULL for authors with no books (LEFT JOIN)
}

/// Fix 1: one JOIN. Author columns repeat on every row; we group in Rust.
pub async fn joined(pool: &SqlitePool) -> Result<(Vec<AuthorWithBooks>, usize), sqlx::Error> {
    let rows: Vec<Joined> = sqlx::query_as(
        "SELECT a.id AS author_id, a.name AS author, b.title
         FROM authors a LEFT JOIN books b ON b.author_id = a.id
         ORDER BY a.id, b.id",
    )
    .fetch_all(pool)
    .await?;
    let mut grouped: BTreeMap<i64, AuthorWithBooks> = BTreeMap::new();
    for row in rows {
        let entry = grouped.entry(row.author_id).or_insert(AuthorWithBooks {
            author: row.author,
            titles: Vec::new(),
        });
        entry.titles.extend(row.title);
    }
    Ok((grouped.into_values().collect(), 1))
}

/// Fix 2: two queries -- the parents, then every child at once with `IN`.
/// What most ORMs do for "eager loading". No row duplication, and it works
/// when the two tables live in different services.
pub async fn batched(pool: &SqlitePool) -> Result<(Vec<AuthorWithBooks>, usize), sqlx::Error> {
    let authors: Vec<Author> = sqlx::query_as("SELECT id, name, born FROM authors ORDER BY id")
        .fetch_all(pool)
        .await?;
    if authors.is_empty() {
        return Ok((Vec::new(), 1));
    }
    // One `?` per id. The placeholders are generated -- never the values.
    let placeholders = vec!["?"; authors.len()].join(", ");
    let sql = format!(
        "SELECT id, author_id, title, year, copies, isbn FROM books WHERE author_id IN ({placeholders}) ORDER BY id"
    );
    let mut query = sqlx::query_as::<_, Book>(sqlx::AssertSqlSafe(sql));
    for a in &authors {
        query = query.bind(a.id);
    }
    let books = query.fetch_all(pool).await?;
    let out = authors
        .into_iter()
        .map(|a| AuthorWithBooks {
            titles: books
                .iter()
                .filter(|b| b.author_id == a.id)
                .map(|b| b.title.clone())
                .collect(),
            author: a.name,
        })
        .collect();
    Ok((out, 2))
}
