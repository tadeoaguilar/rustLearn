# 18 · Databases

## Overview

Most web services are a thin layer over a database, and most of their
performance and correctness problems live in that layer: queries built by
string concatenation, multi-step writes that half-succeed, a loop that issues
a query per row, a missing index. This module works through each of them with
**SQLx** — async, plain SQL, no ORM — on SQLite.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Pools | `SqlitePoolOptions`; why in-memory SQLite needs one connection |
| 2 | Migrations | `migrate!` embeds versioned SQL into the binary |
| 3 | Queries | `query_as` + `FromRow`, `bind`, and a live SQL injection |
| 4 | Repository | A trait, two implementations, one contract test suite |
| 5 | Transactions | Rollback on drop; what goes wrong without one |
| 6 | N+1 | 1 + N queries vs a JOIN vs an `IN` batch |
| 7 | Indexes | `EXPLAIN QUERY PLAN`: SCAN → SEARCH |
| Bonus | Keyset pagination | Cursors instead of offsets |

## Key Concepts

### SQLx in one table

| Need | Call |
|---|---|
| exactly one row | `.fetch_one(&pool)` |
| zero or one | `.fetch_optional(&pool)` → `Option<T>` |
| many | `.fetch_all(&pool)` |
| no rows (INSERT/UPDATE/DELETE) | `.execute(&pool)` → `rows_affected()`, `last_insert_rowid()` |
| map to a struct | `query_as::<_, T>` with `#[derive(FromRow)]` |
| a single value | `query_scalar` |
| inside a transaction | pass `&mut *tx` instead of `&pool` |

Choosing a library: **SQLx** (async, write SQL yourself, optional compile-time
checking), **Diesel** (sync core + `diesel-async`, a type-safe query DSL, the
most compile-time guarantees), **SeaORM** (async ORM built on SQLx, entities and
relations). The concepts in this module apply to all three.

### Never build SQL from data

`format!("... LIKE '%{input}%'")` lets the input *become* SQL. `.bind(input)`
sends it separately, so it can only ever be data. SQLx 0.9 enforces the habit
in the type system: `query()` takes a `&'static str` — anything dynamic must be
wrapped in `AssertSqlSafe(..)`, a flag for code review.

### Transactions roll back on drop

```rust
let mut tx = pool.begin().await?;
sqlx::query("UPDATE ...").execute(&mut *tx).await?;   // if this...
sqlx::query("INSERT ...").execute(&mut *tx).await?;   // ...or this fails, `?` returns,
tx.commit().await?;                                    // tx is dropped -> ROLLBACK
```

There's no "forgot to roll back" path. Without the transaction (Exercise 5),
the failed INSERT leaves the UPDATE behind and a book copy disappears.

### N+1

A loop that runs a query per item is the most common performance bug in data
access code — invisible with 3 test rows, crippling with 3,000 production
ones. Fix it with a JOIN or by loading children in one batch (`IN (...)`).
Count your queries in tests.

### Indexes

Filter, join and sort columns need indexes; foreign keys are **not**
indexed automatically in SQLite or PostgreSQL. `EXPLAIN QUERY PLAN` (SQLite) /
`EXPLAIN ANALYZE` (PostgreSQL) tells you whether one is used.

### Compile-time checked queries

`sqlx::query!("SELECT ...")` checks the SQL and the result types against a
real database while compiling. It needs `DATABASE_URL` at build time, or
pre-generated metadata from `cargo sqlx prepare` committed to the repo. This
module uses the runtime-checked `query`/`query_as` so it builds anywhere; for a
real service, the macros are well worth the setup.

## Common Pitfalls

1. **String-formatted SQL** — injection; use `.bind`
2. **`max_connections > 1` with `sqlite::memory:`** — each connection is a separate, empty database
3. **Multi-step writes without a transaction** — partial failures corrupt data
4. **Queries in a loop** — N+1
5. **Unindexed foreign keys** — slow joins and slow cascading deletes
6. **Editing an applied migration** — SQLx's checksum refuses to run
7. **Thousands of single-row inserts in autocommit mode** — wrap them in one transaction

## Running This Module

```bash
cargo run  -p m18-databases -- all                         # your code
cargo test -p m18-databases-tests --features mine          # test your code
cargo run  -p m18-databases-solution -- all                # every exercise
cargo run  -p m18-databases-solution --release -- 7        # index timings on 200k rows
cargo test -p m18-databases-tests                          # 16 tests against the solution
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the exercise list in
[the phase README](../README.md) (a data layer with SQLx, migrations, a
repository, optimising slow queries).

- **SQLite instead of PostgreSQL**, so nothing needs installing or Docker.
  Everything except a few SQL details (`RETURNING`, `AUTOINCREMENT`, row
  values) carries over unchanged.
- **Migration 0002 is yours to write.** `exercise/migrations/` has only 0001;
  the tests check that 0002 added `books.isbn`.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[19 · Authentication](../19-authentication/)
