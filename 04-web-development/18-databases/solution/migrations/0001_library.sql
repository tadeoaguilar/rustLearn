-- Exercise 2: the initial schema. Migrations run in file-name order, once
-- each; sqlx records what ran in the _sqlx_migrations table.
CREATE TABLE authors (
    id    INTEGER PRIMARY KEY AUTOINCREMENT,
    name  TEXT    NOT NULL UNIQUE,
    born  INTEGER
);

CREATE TABLE books (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    author_id INTEGER NOT NULL REFERENCES authors(id) ON DELETE CASCADE,
    title     TEXT    NOT NULL,
    year      INTEGER NOT NULL,
    copies    INTEGER NOT NULL DEFAULT 1 CHECK (copies >= 0)
);

CREATE TABLE members (
    id    INTEGER PRIMARY KEY AUTOINCREMENT,
    name  TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE
);

CREATE TABLE loans (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id   INTEGER NOT NULL REFERENCES books(id),
    member_id INTEGER NOT NULL REFERENCES members(id),
    returned  INTEGER NOT NULL DEFAULT 0
);
