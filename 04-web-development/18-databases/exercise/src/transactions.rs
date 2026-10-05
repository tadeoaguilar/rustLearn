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
    todo!("Exercise 5")
}

/// The same two steps *without* a transaction -- the bug, for comparison.
pub async fn lend_book_without_transaction(
    pool: &SqlitePool,
    book_id: i64,
    member_id: i64,
) -> Result<i64, LoanError> {
    todo!("Exercise 5")
}

pub async fn return_book(pool: &SqlitePool, loan_id: i64) -> Result<(), LoanError> {
    todo!("Exercise 5")
}

pub async fn copies(pool: &SqlitePool, book_id: i64) -> Result<i64, sqlx::Error> {
    todo!("Exercise 5")
}

pub async fn open_loans(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    todo!("Exercise 5")
}
