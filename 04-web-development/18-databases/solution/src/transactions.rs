//! Exercise 5: transactions.
//!
//! Lending a book is two writes: decrement `books.copies`, insert a `loans`
//! row. If the second fails after the first succeeded, a copy vanishes from
//! the shelf with no loan to explain it. A transaction makes the two writes
//! one: both happen, or neither.

use sqlx::SqlitePool;

#[derive(Debug, thiserror::Error)]
pub enum LoanError {
    #[error("no such book")]
    NoSuchBook,
    #[error("no copies left")]
    NoCopiesLeft,
    #[error("no such loan, or it was already returned")]
    NoSuchLoan,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Returns the new loan's id.
///
/// `pool.begin()` starts a transaction on one connection; every query passes
/// `&mut *tx` so it runs inside it. If this function returns early -- `?` or
/// `return Err` -- `tx` is dropped without `commit()`, and the transaction is
/// **rolled back automatically**. That's the whole trick: there's no code path
/// that forgets to clean up.
pub async fn lend_book(pool: &SqlitePool, book_id: i64, member_id: i64) -> Result<i64, LoanError> {
    let mut tx = pool.begin().await?;

    let copies: Option<i64> = sqlx::query_scalar("SELECT copies FROM books WHERE id = ?")
        .bind(book_id)
        .fetch_optional(&mut *tx)
        .await?;
    match copies {
        None => return Err(LoanError::NoSuchBook),
        Some(0) => return Err(LoanError::NoCopiesLeft),
        Some(_) => {}
    }

    sqlx::query("UPDATE books SET copies = copies - 1 WHERE id = ?")
        .bind(book_id)
        .execute(&mut *tx)
        .await?;

    // If member_id doesn't exist this INSERT fails a foreign key -- *after*
    // the UPDATE above. The `?` returns, `tx` is dropped, the UPDATE is undone.
    let loan_id = sqlx::query("INSERT INTO loans (book_id, member_id) VALUES (?, ?)")
        .bind(book_id)
        .bind(member_id)
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();

    tx.commit().await?;
    Ok(loan_id)
}

/// The same two steps *without* a transaction -- the bug, for comparison.
pub async fn lend_book_without_transaction(
    pool: &SqlitePool,
    book_id: i64,
    member_id: i64,
) -> Result<i64, LoanError> {
    sqlx::query("UPDATE books SET copies = copies - 1 WHERE id = ? AND copies > 0")
        .bind(book_id)
        .execute(pool)
        .await?;
    let id = sqlx::query("INSERT INTO loans (book_id, member_id) VALUES (?, ?)")
        .bind(book_id)
        .bind(member_id)
        .execute(pool)
        .await?
        .last_insert_rowid();
    Ok(id)
}

pub async fn return_book(pool: &SqlitePool, loan_id: i64) -> Result<(), LoanError> {
    let mut tx = pool.begin().await?;
    // Only an open loan can be returned; the WHERE makes the check and the
    // update one atomic statement.
    let book_id: Option<i64> = sqlx::query_scalar(
        "UPDATE loans SET returned = 1 WHERE id = ? AND returned = 0 RETURNING book_id",
    )
    .bind(loan_id)
    .fetch_optional(&mut *tx)
    .await?;
    let book_id = book_id.ok_or(LoanError::NoSuchLoan)?;
    sqlx::query("UPDATE books SET copies = copies + 1 WHERE id = ?")
        .bind(book_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn copies(pool: &SqlitePool, book_id: i64) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT copies FROM books WHERE id = ?")
        .bind(book_id)
        .fetch_one(pool)
        .await
}

pub async fn open_loans(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM loans WHERE returned = 0")
        .fetch_one(pool)
        .await
}
