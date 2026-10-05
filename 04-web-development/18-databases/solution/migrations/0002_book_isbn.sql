-- Exercise 2: a second migration changes the schema *after* the first has
-- shipped. Never edit 0001 once it has run anywhere: sqlx checksums every
-- applied migration and refuses to continue if one changed.
ALTER TABLE books ADD COLUMN isbn TEXT;
CREATE UNIQUE INDEX books_isbn_unique ON books(isbn) WHERE isbn IS NOT NULL;
