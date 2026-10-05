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
        SqliteBookRepository { pool }
    }
}

/// Database errors that are really *domain* errors get translated, so callers
/// never match on SQLite error strings.
fn translate(e: sqlx::Error) -> RepoError {
    if let Some(db) = e.as_database_error() {
        if db.is_unique_violation() {
            return RepoError::Conflict("a book with that ISBN already exists".into());
        }
        if db.is_foreign_key_violation() {
            return RepoError::Invalid("no such author".into());
        }
    }
    RepoError::Database(e)
}

impl BookRepository for SqliteBookRepository {
    async fn add(&self, b: NewBook) -> Result<Book, RepoError> {
        sqlx::query_as(
            "INSERT INTO books (author_id, title, year, copies, isbn) VALUES (?, ?, ?, ?, ?)
             RETURNING id, author_id, title, year, copies, isbn",
        )
        .bind(b.author_id)
        .bind(&b.title)
        .bind(b.year)
        .bind(b.copies)
        .bind(&b.isbn)
        .fetch_one(&self.pool)
        .await
        .map_err(translate)
    }

    async fn get(&self, id: i64) -> Result<Option<Book>, RepoError> {
        Ok(sqlx::query_as(
            "SELECT id, author_id, title, year, copies, isbn FROM books WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?)
    }

    async fn by_author(&self, author_id: i64) -> Result<Vec<Book>, RepoError> {
        Ok(sqlx::query_as("SELECT id, author_id, title, year, copies, isbn FROM books WHERE author_id = ? ORDER BY id")
            .bind(author_id)
            .fetch_all(&self.pool)
            .await?)
    }

    async fn set_copies(&self, id: i64, copies: i64) -> Result<Book, RepoError> {
        sqlx::query_as("UPDATE books SET copies = ? WHERE id = ? RETURNING id, author_id, title, year, copies, isbn")
            .bind(copies)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(translate)?
            .ok_or(RepoError::NotFound)
    }

    async fn delete(&self, id: i64) -> Result<bool, RepoError> {
        let result = sqlx::query("DELETE FROM books WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() == 1)
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
        InMemoryBookRepository {
            books: Mutex::new(Vec::new()),
            known_authors,
        }
    }
}

impl BookRepository for InMemoryBookRepository {
    async fn add(&self, b: NewBook) -> Result<Book, RepoError> {
        if !self.known_authors.contains(&b.author_id) {
            return Err(RepoError::Invalid("no such author".into()));
        }
        let mut books = self.books.lock().unwrap();
        if b.isbn.is_some() && books.iter().any(|x| x.isbn == b.isbn) {
            return Err(RepoError::Conflict(
                "a book with that ISBN already exists".into(),
            ));
        }
        let id = books.iter().map(|x| x.id).max().unwrap_or(0) + 1;
        let book = Book {
            id,
            author_id: b.author_id,
            title: b.title,
            year: b.year,
            copies: b.copies,
            isbn: b.isbn,
        };
        books.push(book.clone());
        Ok(book)
    }

    async fn get(&self, id: i64) -> Result<Option<Book>, RepoError> {
        Ok(self
            .books
            .lock()
            .unwrap()
            .iter()
            .find(|b| b.id == id)
            .cloned())
    }

    async fn by_author(&self, author_id: i64) -> Result<Vec<Book>, RepoError> {
        Ok(self
            .books
            .lock()
            .unwrap()
            .iter()
            .filter(|b| b.author_id == author_id)
            .cloned()
            .collect())
    }

    async fn set_copies(&self, id: i64, copies: i64) -> Result<Book, RepoError> {
        if copies < 0 {
            return Err(RepoError::Database(sqlx::Error::Protocol(
                "CHECK constraint failed: copies >= 0".into(),
            )));
        }
        let mut books = self.books.lock().unwrap();
        let book = books
            .iter_mut()
            .find(|b| b.id == id)
            .ok_or(RepoError::NotFound)?;
        book.copies = copies;
        Ok(book.clone())
    }

    async fn delete(&self, id: i64) -> Result<bool, RepoError> {
        let mut books = self.books.lock().unwrap();
        let before = books.len();
        books.retain(|b| b.id != id);
        Ok(books.len() != before)
    }
}

// ---- Business logic, independent of storage --------------------------------------

pub struct Catalog<R: BookRepository> {
    repo: R,
    current_year: i64,
}

impl<R: BookRepository> Catalog<R> {
    pub fn new(repo: R, current_year: i64) -> Self {
        Catalog { repo, current_year }
    }

    /// Rules that belong to the domain, not the database.
    pub async fn add_book(&self, book: NewBook) -> Result<Book, RepoError> {
        if book.title.trim().is_empty() {
            return Err(RepoError::Invalid("title is required".into()));
        }
        if book.year > self.current_year {
            return Err(RepoError::Invalid(format!(
                "{} is in the future",
                book.year
            )));
        }
        if book.copies < 0 {
            return Err(RepoError::Invalid("copies can't be negative".into()));
        }
        self.repo
            .add(NewBook {
                title: book.title.trim().to_string(),
                ..book
            })
            .await
    }

    pub async fn restock(&self, id: i64, extra: i64) -> Result<Book, RepoError> {
        let book = self.repo.get(id).await?.ok_or(RepoError::NotFound)?;
        self.repo.set_copies(id, book.copies + extra).await
    }

    pub async fn titles_by(&self, author_id: i64) -> Result<Vec<String>, RepoError> {
        Ok(self
            .repo
            .by_author(author_id)
            .await?
            .into_iter()
            .map(|b| b.title)
            .collect())
    }

    pub fn repository(&self) -> &R {
        &self.repo
    }
}
