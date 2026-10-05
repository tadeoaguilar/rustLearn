# Exercises: Databases

A small library database — authors, books, members, loans — with **SQLx**
on **SQLite**. SQLite runs inside your program, so there's nothing to
install, and every test gets a fresh in-memory database. The same SQLx code
works with PostgreSQL or MySQL by swapping the pool type and connection
string.

**Setup**: `sqlx` (features `runtime-tokio`, `sqlite`, `migrate`, `macros`),
`tokio` and `thiserror` are in `exercise/Cargo.toml`.
`exercise/migrations/0001_library.sql` holds the starting schema.

---

## Exercise 1: Connecting and Pooling

**Difficulty**: Easy
**Time**: 30 minutes

```rust
pub async fn connect_memory() -> Result<SqlitePool, sqlx::Error>;            // in-memory
pub async fn connect_file(path: &Path, max_connections: u32) -> Result<SqlitePool, sqlx::Error>;
pub async fn seed(pool: &SqlitePool) -> Result<(), sqlx::Error>;            // 3 authors, 6 books, 2 members
pub async fn setup() -> SqlitePool;                                          // connect + migrate + seed
```

**Gotcha to discover**: with `sqlite::memory:`, set `max_connections(1)`.
Try 5 and run a few queries — what happens, and why?

Seed data (the tests rely on it):

| Author (born) | Books (year, copies) |
|---|---|
| Ursula K. Le Guin (1929) | A Wizard of Earthsea (1968, 2), The Left Hand of Darkness (1969, 1), The Dispossessed (1974, 0) |
| Terry Pratchett (1948) | Guards! Guards! (1989, 3), Small Gods (1992, 1) |
| Octavia E. Butler (1947) | Kindred (1979, 2) |

Members: Ann (`ann@example.com`), Bob (`bob@example.com`).

---

## Exercise 2: Migrations

**Difficulty**: Easy
**Time**: 25 minutes

1. Run the migrations with `sqlx::migrate!("./migrations").run(&pool)`. The
   macro embeds the SQL files into your binary at compile time.
2. Write `migrations/0002_book_isbn.sql`: add a nullable `isbn TEXT` column to
   `books`, and a **unique** index on it that allows many NULLs.
3. Look inside `_sqlx_migrations`. What happens if you edit `0001` after it ran?

---

## Exercise 3: Typed Queries — and SQL Injection

**Difficulty**: Medium
**Time**: 40 minutes

```rust
#[derive(FromRow)] pub struct Author { pub id: i64, pub name: String, pub born: Option<i64> }
#[derive(FromRow)] pub struct Book { pub id: i64, pub author_id: i64, pub title: String, pub year: i64, pub copies: i64, pub isbn: Option<String> }

pub async fn add_author(pool, name, born) -> Result<Author, sqlx::Error>;          // INSERT ... RETURNING
pub async fn author_by_name(pool, name) -> Result<Option<Author>, sqlx::Error>;    // fetch_optional
pub async fn books_by_author(pool, author_id) -> Result<Vec<Book>, sqlx::Error>;   // oldest first
pub async fn search_titles(pool, fragment) -> Result<Vec<String>, sqlx::Error>;    // LIKE, case-insensitive
pub async fn copies_per_author(pool) -> Result<Vec<(String, i64)>, sqlx::Error>;   // GROUP BY, by name
```

Then write `search_titles_unsafe`, which builds the SQL with `format!`, and
attack both with the fragment `x%' OR 1=1 --`.

**Notice**: SQLx 0.9's `query()` only accepts a `&'static str`. To pass a
`String` you must wrap it: `sqlx::query(sqlx::AssertSqlSafe(sql))`. Why is that
a good design?

---

## Exercise 4: The Repository Pattern

**Difficulty**: Hard
**Time**: 60 minutes

```rust
pub trait BookRepository {
    fn add(&self, book: NewBook) -> impl Future<Output = Result<Book, RepoError>> + Send;
    fn get(&self, id: i64) -> impl Future<Output = Result<Option<Book>, RepoError>> + Send;
    fn by_author(&self, author_id: i64) -> impl Future<Output = Result<Vec<Book>, RepoError>> + Send;
    fn set_copies(&self, id: i64, copies: i64) -> impl Future<Output = Result<Book, RepoError>> + Send;
    fn delete(&self, id: i64) -> impl Future<Output = Result<bool, RepoError>> + Send;
}
pub enum RepoError { NotFound, Conflict(String), Invalid(String), Database(sqlx::Error) }
```

1. `SqliteBookRepository`: translate a unique violation (duplicate ISBN) to
   `Conflict` and a foreign-key violation (unknown author) to `Invalid` —
   `e.as_database_error()`, `is_unique_violation()`, `is_foreign_key_violation()`.
2. `InMemoryBookRepository`: the same behaviour with a `Mutex<Vec<Book>>`.
3. `Catalog<R: BookRepository>`: domain rules (title required, year not in
   the future, copies ≥ 0) and `restock`, `titles_by`.

The tests run **one contract test suite against both** repositories.

**Question**: why `fn ... -> impl Future + Send` instead of `async fn` in the trait?

---

## Exercise 5: Transactions

**Difficulty**: Medium
**Time**: 45 minutes

```rust
pub async fn lend_book(pool, book_id, member_id) -> Result<i64, LoanError>;   // returns loan id
pub async fn return_book(pool, loan_id) -> Result<(), LoanError>;
pub enum LoanError { NoSuchBook, NoCopiesLeft, NoSuchLoan, Database(sqlx::Error) }
```

Lending = decrement `copies` **and** insert a loan — atomically. Use
`pool.begin()`, pass `&mut *tx` to each query, `tx.commit()` at the end.

Then lend to member `999` (doesn't exist). The INSERT fails a foreign key
*after* the UPDATE ran. Check `copies` — unchanged? Now write
`lend_book_without_transaction` and repeat.

---

## Exercise 6: The N+1 Query Problem

**Difficulty**: Medium
**Time**: 40 minutes

"Every author with the titles of their books", three ways, each returning
`(Vec<AuthorWithBooks>, number_of_queries)`:
1. `naive`: authors, then one query per author — 1 + N
2. `joined`: one `LEFT JOIN`, grouped in Rust — 1
3. `batched`: authors, then all their books with one `WHERE author_id IN (?, ?, ...)` — 2

All three must return identical results, including authors with no books.

---

## Exercise 7: Indexes and Query Plans

**Difficulty**: Medium
**Time**: 30 minutes

1. `query_plan(pool, sql, value) -> Vec<String>` using `EXPLAIN QUERY PLAN`
2. Show that `SELECT title FROM books WHERE year = ?` does `SCAN books`
3. `add_indexes`: indexes on `books(year)` and `books(author_id)` — now `SEARCH books USING INDEX ...`
4. Insert 200,000 rows (in **one** transaction — why?) and time 50 queries before and after

---

## Bonus Challenge: Keyset Pagination

**Difficulty**: Hard
**Time**: 45 minutes

```rust
pub async fn books_after(pool, after: Option<Cursor>, limit: i64) -> Result<(Vec<Book>, Option<Cursor>), sqlx::Error>;
```

Order by `(year, id)`; the cursor `"1979:6"` means "after year 1979, id 6";
`WHERE (year, id) > (?, ?)`. Show that inserting a row *before* the cursor
doesn't shift the next page.

---

## Check Your Understanding

- [ ] Configure a pool and know SQLite's in-memory gotcha
- [ ] Write and apply migrations; never edit an applied one
- [ ] Map rows to structs; always bind parameters
- [ ] Separate storage from domain logic with a repository trait
- [ ] Make multi-step writes atomic with a transaction
- [ ] Spot and fix N+1 queries
- [ ] Read a query plan and add the right index

---

## Additional Resources

- [SQLx](https://docs.rs/sqlx/) and its [examples](https://github.com/launchbadge/sqlx/tree/main/examples)
- [SQLite query planner](https://www.sqlite.org/eqp.html)
- [Use The Index, Luke](https://use-the-index-luke.com/) — indexing, for every database
- [Diesel](https://diesel.rs/) and [SeaORM](https://www.sea-ql.org/SeaORM/) — the ORM alternatives
