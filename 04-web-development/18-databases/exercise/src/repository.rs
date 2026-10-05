//! Exercise 4: the repository pattern.
//!
//! Business logic (`Catalog`) talks to a `BookRepository` trait, not to SQL.
//! Two implementations: SQLite for real, and an in-memory one for fast unit
//! tests. The *same* test suite runs against both (see the tests crate), which
//! is how you know the fake behaves like the real thing.
//!
//! The trait's methods are written `fn ... -> impl Future<Output = ..> + Send`
//! rather than `async fn`. Both work, but plain `async fn` in a public trait
//! can't promise its futures are `Send` -- and a repository whose futures
//! aren't `Send` can't be used from `tokio::spawn` or an Axum handler. (The
//! compiler warns: `async_fn_in_trait`.) Implementations may still write
//! `async fn`; the compiler checks that each one's future really is `Send`.
//!
//! Such traits can't be used as `dyn BookRepository`, so `Catalog` is
//! generic over the repository.

use crate::queries::Book;
use sqlx::SqlitePool;
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewBook {
    pub author_id: i64,
    pub title: String,
    pub year: i64,
    pub copies: i64,
    pub isbn: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("invalid: {0}")]
    Invalid(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

use std::future::Future;

pub trait BookRepository {
    fn add(&self, book: NewBook) -> impl Future<Output = Result<Book, RepoError>> + Send;
    fn get(&self, id: i64) -> impl Future<Output = Result<Option<Book>, RepoError>> + Send;
    fn by_author(
        &self,
        author_id: i64,
    ) -> impl Future<Output = Result<Vec<Book>, RepoError>> + Send;
    fn set_copies(
        &self,
        id: i64,
        copies: i64,
    ) -> impl Future<Output = Result<Book, RepoError>> + Send;
    fn delete(&self, id: i64) -> impl Future<Output = Result<bool, RepoError>> + Send;
}

// ---- SQLite -------------------------------------------------------------------

pub struct SqliteBookRepository {
    pool: SqlitePool,
}

impl SqliteBookRepository {
    pub fn new(pool: SqlitePool) -> Self {
        todo!("Exercise 4")
    }
}

/// Database errors that are really *domain* errors get translated, so callers
/// never match on SQLite error strings.
fn translate(e: sqlx::Error) -> RepoError {
    todo!("Exercise 4")
}

impl BookRepository for SqliteBookRepository {
    async fn add(&self, b: NewBook) -> Result<Book, RepoError> {
        todo!("Exercise 4")
    }

    async fn get(&self, id: i64) -> Result<Option<Book>, RepoError> {
        todo!("Exercise 4")
    }

    async fn by_author(&self, author_id: i64) -> Result<Vec<Book>, RepoError> {
        todo!("Exercise 4")
    }

    async fn set_copies(&self, id: i64, copies: i64) -> Result<Book, RepoError> {
        todo!("Exercise 4")
    }

    async fn delete(&self, id: i64) -> Result<bool, RepoError> {
        todo!("Exercise 4")
    }
}

// ---- In memory ------------------------------------------------------------------

/// A fake for unit tests: same contract, no database. It must enforce the
/// same rules the schema does (unique ISBN, known authors, copies >= 0) --
/// otherwise tests pass against the fake and fail in production.
pub struct InMemoryBookRepository {
    books: Mutex<Vec<Book>>,
    known_authors: Vec<i64>,
}

impl InMemoryBookRepository {
    pub fn new(known_authors: Vec<i64>) -> Self {
        todo!("Exercise 4")
    }
}

impl BookRepository for InMemoryBookRepository {
    async fn add(&self, b: NewBook) -> Result<Book, RepoError> {
        todo!("Exercise 4")
    }

    async fn get(&self, id: i64) -> Result<Option<Book>, RepoError> {
        todo!("Exercise 4")
    }

    async fn by_author(&self, author_id: i64) -> Result<Vec<Book>, RepoError> {
        todo!("Exercise 4")
    }

    async fn set_copies(&self, id: i64, copies: i64) -> Result<Book, RepoError> {
        todo!("Exercise 4")
    }

    async fn delete(&self, id: i64) -> Result<bool, RepoError> {
        todo!("Exercise 4")
    }
}

// ---- Business logic, independent of storage --------------------------------------

pub struct Catalog<R: BookRepository> {
    repo: R,
    current_year: i64,
}

impl<R: BookRepository> Catalog<R> {
    pub fn new(repo: R, current_year: i64) -> Self {
        todo!("Exercise 4")
    }

    /// Rules that belong to the domain, not the database.
    pub async fn add_book(&self, book: NewBook) -> Result<Book, RepoError> {
        todo!("Exercise 4")
    }

    pub async fn restock(&self, id: i64, extra: i64) -> Result<Book, RepoError> {
        todo!("Exercise 4")
    }

    pub async fn titles_by(&self, author_id: i64) -> Result<Vec<String>, RepoError> {
        todo!("Exercise 4")
    }

    pub fn repository(&self) -> &R {
        todo!("Exercise 4")
    }
}
