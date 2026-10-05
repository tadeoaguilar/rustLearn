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
        todo!("Bonus")
    }

    pub fn decode(s: &str) -> Option<Cursor> {
        todo!("Bonus")
    }
}

/// Books ordered by (year, id). Returns the page and the cursor for the next
/// one (None on the last page).
pub async fn books_after(
    pool: &SqlitePool,
    after: Option<Cursor>,
    limit: i64,
) -> Result<(Vec<Book>, Option<Cursor>), sqlx::Error> {
    todo!("Bonus")
}
