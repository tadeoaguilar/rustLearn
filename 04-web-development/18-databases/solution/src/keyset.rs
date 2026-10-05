//! Bonus: keyset (cursor) pagination.
//!
//! `LIMIT 20 OFFSET 100000` makes the database walk past 100,000 rows to
//! throw them away. Keyset pagination remembers where the last page ended and
//! asks for rows *after* it: `WHERE (year, id) > (?, ?)` -- an index seek,
//! equally fast on page 1 and page 5,000, and stable when rows are inserted.
//!
//! The cursor must contain every sort key plus a unique tie-breaker (id).

use crate::queries::Book;
use sqlx::SqlitePool;

/// Opaque to clients: "year:id".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub year: i64,
    pub id: i64,
}

impl Cursor {
    pub fn encode(&self) -> String {
        format!("{}:{}", self.year, self.id)
    }

    pub fn decode(s: &str) -> Option<Cursor> {
        let (year, id) = s.split_once(':')?;
        Some(Cursor {
            year: year.parse().ok()?,
            id: id.parse().ok()?,
        })
    }
}

/// Books ordered by (year, id). Returns the page and the cursor for the next
/// one (None on the last page).
pub async fn books_after(
    pool: &SqlitePool,
    after: Option<Cursor>,
    limit: i64,
) -> Result<(Vec<Book>, Option<Cursor>), sqlx::Error> {
    // Fetch one extra row to learn whether another page exists.
    let after = after.unwrap_or(Cursor {
        year: i64::MIN,
        id: i64::MIN,
    });
    let mut books: Vec<Book> = sqlx::query_as(
        "SELECT id, author_id, title, year, copies, isbn FROM books
         WHERE (year, id) > (?, ?)
         ORDER BY year, id
         LIMIT ?",
    )
    .bind(after.year)
    .bind(after.id)
    .bind(limit + 1)
    .fetch_all(pool)
    .await?;
    let next = if books.len() as i64 > limit {
        books.truncate(limit as usize);
        books.last().map(|b| Cursor {
            year: b.year,
            id: b.id,
        })
    } else {
        None
    };
    Ok((books, next))
}
