# Answers · 18 Databases

## Exercise 1: What happens with `max_connections(5)` on `sqlite::memory:`?

Each connection opens its *own* private in-memory database. Migrations run on
whichever connection the pool hands out first; the next query may get a
different connection — an empty database — and fail with `no such table:
books`, seemingly at random. Fixes: one connection (as here), a shared-cache
URL (`sqlite:file:memdb1?mode=memory&cache=shared`), or a temporary file.

## Exercise 2: What if you edit `0001` after it ran?

SQLx stores a checksum of every applied migration in `_sqlx_migrations`. On
the next `migrate().run()` it sees the file no longer matches and fails with
`migration 1 was previously applied but has been modified`. Applied migrations
are history: fix mistakes with a *new* migration.

## Exercise 3: Why is `query(&'static str)` + `AssertSqlSafe` a good design?

A string literal can't contain user input, so the common, safe case needs no
ceremony. Building SQL at runtime is sometimes legitimate (a generated list of
`?` placeholders, as in Exercise 6), but it's exactly where injections come
from — so it must be spelled `AssertSqlSafe(sql)`, which stands out in a diff
and is easy to grep for in an audit.

## Exercise 4: Why `fn ... -> impl Future + Send` instead of `async fn`?

An `async fn` in a trait returns *some* future, and callers can't require that
it's `Send`. A web server runs handlers on a multi-threaded runtime
(`tokio::spawn`, Axum), which needs `Send` futures — so a repository behind
plain `async fn` can't be used there, and the compiler warns
(`async_fn_in_trait`) for public traits. Writing the return type as
`impl Future<Output = ..> + Send` puts the guarantee in the trait;
implementations can still use `async fn`. (The `trait-variant` crate can
generate the `Send` variant for you.)

## Exercise 7: Why insert 200,000 rows in one transaction?

In autocommit mode each INSERT is its own transaction, and SQLite makes each
one durable on disk (journal writes, fsync) before moving on: tens of
thousands of slow syncs. Inside one transaction there's a single commit at the
end — easily 100× faster. The same applies, less dramatically, to every
database.
