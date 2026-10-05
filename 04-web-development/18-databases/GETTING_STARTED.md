# Getting Started with 18 · Databases

## Quick Start

All commands run from the **repository root**. No database server needed.

```bash
cargo run -p m18-databases-solution -- all
```

## What Is Already Here

```
18-databases/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/                     # ← YOUR WORKSPACE (package m18-databases)
│   ├── migrations/0001_library.sql   #   provided; you add 0002 (Exercise 2)
│   └── src/db.rs, queries.rs, repository.rs, transactions.rs,
│            n_plus_one.rs, indexes.rs, keyset.rs, main.rs
├── solution/                     # ← REFERENCE (m18-databases-solution) + ANSWERS.md
│   └── migrations/0001_library.sql, 0002_book_isbn.sql
└── tests/                        # ← 16 tests (m18-databases-tests)
```

## The Commands You Need

```bash
cargo run  -p m18-databases -- 3                         # your code, one exercise (1..7, bonus, all)
cargo test -p m18-databases-tests --features mine
cargo test -p m18-databases-tests --features mine ex5_
cargo run  -p m18-databases-solution -- 5                # transactions demo
cargo run  -p m18-databases-solution --release -- 7      # index timings
cargo test -p m18-databases-tests                        # always green
```

## Poke at a Real Database File

```bash
# with the sqlite3 CLI (preinstalled on macOS)
sqlite3 /tmp/library.db < 04-web-development/18-databases/solution/migrations/0001_library.sql
sqlite3 /tmp/library.db "EXPLAIN QUERY PLAN SELECT title FROM books WHERE year = 1990"
sqlite3 /tmp/library.db "CREATE INDEX idx_books_year ON books(year)"
sqlite3 /tmp/library.db "EXPLAIN QUERY PLAN SELECT title FROM books WHERE year = 1990"
```

`db::connect_file(path, n)` points the Rust code at a file like this.

## Why There's a `build.rs`

`sqlx::migrate!` embeds the files in `migrations/` at **compile** time. On
stable Rust, Cargo doesn't know to rebuild when you *add* a file there, so
your new `0002_book_isbn.sql` could be silently ignored. `build.rs` prints
`cargo:rerun-if-changed=migrations`, which fixes that. Keep it in any project
that uses `migrate!`.

## If You Get Stuck

1. **`no such table: books`** — migrations didn't run, or (in memory) you have more than one connection.
2. **`the trait SqlSafeStr is not implemented for String`** — SQLx 0.9 wants a `&'static str`; wrap dynamic SQL in `sqlx::AssertSqlSafe(..)` (and make sure it contains no user input).
3. **`FromRow`: `no column found for name: ...`** — the SELECT list must name every struct field (alias with `AS` if needed).
4. **`FOREIGN KEY constraint failed`** — that's Exercise 5 working; make sure the transaction rolled back.
5. **`future cannot be sent between threads safely`** — a repository future isn't `Send`; see the Exercise 4 question.
6. Compare against `solution/src/` — same file and function names.
