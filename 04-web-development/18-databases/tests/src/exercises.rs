//! Every test gets its own in-memory database: no shared state, no cleanup.

use crate::sut::*;
use repository::{
    BookRepository, Catalog, InMemoryBookRepository, NewBook, RepoError, SqliteBookRepository,
};

fn new_book(author_id: i64, title: &str, isbn: Option<&str>) -> NewBook {
    NewBook {
        author_id,
        title: title.into(),
        year: 2000,
        copies: 1,
        isbn: isbn.map(String::from),
    }
}

// ---- Exercises 1-2: pool and migrations ---------------------------------------------

#[tokio::test]
async fn ex1_ex2_migrations_create_the_schema_once() {
    let pool = db::connect_memory().await.unwrap();
    db::migrate(&pool).await.unwrap();
    db::migrate(&pool).await.unwrap(); // idempotent
    assert_eq!(
        db::tables(&pool).await.unwrap(),
        vec!["_sqlx_migrations", "authors", "books", "loans", "members"]
    );
    let versions: Vec<i64> =
        sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(versions, vec![1, 2]);
    let isbn_column: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM pragma_table_info('books') WHERE name = 'isbn'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(isbn_column, 1, "migration 0002 adds books.isbn");
}

#[tokio::test]
async fn ex1_seed_data() {
    let pool = db::setup().await;
    let counts: (i64, i64, i64) = sqlx::query_as("SELECT (SELECT COUNT(*) FROM authors), (SELECT COUNT(*) FROM books), (SELECT COUNT(*) FROM members)").fetch_one(&pool).await.unwrap();
    assert_eq!(counts, (3, 6, 2));
}

#[tokio::test]
async fn ex1_file_database_with_a_real_pool() {
    let dir = tempfile::tempdir().unwrap();
    let pool = db::connect_file(&dir.path().join("library.db"), 4)
        .await
        .unwrap();
    db::migrate(&pool).await.unwrap();
    db::seed(&pool).await.unwrap();
    // Several connections, one database -- unlike sqlite::memory:.
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let pool = pool.clone();
            tokio::spawn(async move {
                sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM books")
                    .fetch_one(&pool)
                    .await
                    .unwrap()
            })
        })
        .collect();
    for h in handles {
        assert_eq!(h.await.unwrap(), 6);
    }
}

// ---- Exercise 3: queries --------------------------------------------------------------

#[tokio::test]
async fn ex3_insert_returning_and_lookups() {
    let pool = db::setup().await;
    let a = queries::add_author(&pool, "N. K. Jemisin", Some(1972))
        .await
        .unwrap();
    assert_eq!(
        (a.id, a.name.as_str(), a.born),
        (4, "N. K. Jemisin", Some(1972))
    );
    assert!(
        queries::add_author(&pool, "N. K. Jemisin", None)
            .await
            .is_err(),
        "names are unique"
    );
    assert_eq!(
        queries::author_by_name(&pool, "Terry Pratchett")
            .await
            .unwrap()
            .map(|a| a.id),
        Some(2)
    );
    assert_eq!(
        queries::author_by_name(&pool, "Nobody").await.unwrap(),
        None
    );
    let books = queries::books_by_author(&pool, 1).await.unwrap();
    assert_eq!(
        books.iter().map(|b| b.year).collect::<Vec<_>>(),
        vec![1968, 1969, 1974]
    );
}

#[tokio::test]
async fn ex3_search_and_aggregates() {
    let pool = db::setup().await;
    assert_eq!(
        queries::search_titles(&pool, "THE").await.unwrap(),
        vec!["The Dispossessed", "The Left Hand of Darkness"]
    );
    assert_eq!(
        queries::copies_per_author(&pool).await.unwrap(),
        vec![
            ("Octavia E. Butler".to_string(), 2),
            ("Terry Pratchett".to_string(), 4),
            ("Ursula K. Le Guin".to_string(), 3)
        ]
    );
}

#[tokio::test]
async fn ex3_bound_parameters_resist_injection() {
    let pool = db::setup().await;
    let attack = "x%' OR 1=1 --";
    assert!(
        queries::search_titles(&pool, attack)
            .await
            .unwrap()
            .is_empty(),
        "with bind, the attack is just a weird title"
    );
    assert_eq!(
        queries::search_titles_unsafe(&pool, attack)
            .await
            .unwrap()
            .len(),
        6,
        "with format!, it returns every row"
    );
}

// ---- Exercise 4: the repository pattern ----------------------------------------------

/// The contract every BookRepository must meet -- run against both.
async fn repository_contract<R: BookRepository>(repo: R) {
    let a = repo
        .add(new_book(1, "First", Some("isbn-1")))
        .await
        .unwrap();
    let b = repo.add(new_book(1, "Second", None)).await.unwrap();
    let c = repo.add(new_book(2, "Other author", None)).await.unwrap();
    assert!(a.id < b.id && b.id < c.id, "ids increase");
    assert_eq!(repo.get(a.id).await.unwrap(), Some(a.clone()));
    assert_eq!(repo.get(999).await.unwrap(), None);
    let by_one: Vec<String> = repo
        .by_author(1)
        .await
        .unwrap()
        .into_iter()
        .map(|b| b.title)
        .collect();
    assert!(by_one.contains(&"First".to_string()) && by_one.contains(&"Second".to_string()));
    assert!(!by_one.contains(&"Other author".to_string()));

    assert!(matches!(
        repo.add(new_book(1, "Dup", Some("isbn-1"))).await,
        Err(RepoError::Conflict(_))
    ));
    repo.add(new_book(1, "Two NULL isbns are fine", None))
        .await
        .unwrap();
    assert!(matches!(
        repo.add(new_book(99, "Ghost", None)).await,
        Err(RepoError::Invalid(_))
    ));

    assert_eq!(repo.set_copies(a.id, 7).await.unwrap().copies, 7);
    assert!(matches!(
        repo.set_copies(999, 1).await,
        Err(RepoError::NotFound)
    ));
    assert!(
        repo.set_copies(a.id, -1).await.is_err(),
        "copies can't go negative"
    );

    assert!(repo.delete(a.id).await.unwrap());
    assert!(!repo.delete(a.id).await.unwrap());
}

#[tokio::test]
async fn ex4_sqlite_repository_meets_the_contract() {
    let pool = db::connect_memory().await.unwrap();
    db::migrate(&pool).await.unwrap();
    db::seed(&pool).await.unwrap();
    repository_contract(SqliteBookRepository::new(pool)).await;
}

#[tokio::test]
async fn ex4_in_memory_repository_meets_the_same_contract() {
    repository_contract(InMemoryBookRepository::new(vec![1, 2, 3])).await;
}

#[tokio::test]
async fn ex4_catalog_rules_apply_whatever_the_storage() {
    let catalog = Catalog::new(InMemoryBookRepository::new(vec![1]), 2026);
    assert!(matches!(
        catalog.add_book(new_book(1, "  ", None)).await,
        Err(RepoError::Invalid(_))
    ));
    assert!(matches!(
        catalog
            .add_book(NewBook {
                year: 2030,
                ..new_book(1, "Future", None)
            })
            .await,
        Err(RepoError::Invalid(_))
    ));
    let book = catalog
        .add_book(new_book(1, "  Trimmed  ", None))
        .await
        .unwrap();
    assert_eq!(book.title, "Trimmed");
    assert_eq!(catalog.restock(book.id, 3).await.unwrap().copies, 4);
    assert!(matches!(
        catalog.restock(42, 1).await,
        Err(RepoError::NotFound)
    ));
    assert_eq!(catalog.titles_by(1).await.unwrap(), vec!["Trimmed"]);
}

#[tokio::test]
async fn ex4_repository_futures_are_send() {
    // Only compiles if the trait's futures are Send: tokio::spawn requires it.
    let pool = db::setup().await;
    let repo = std::sync::Arc::new(SqliteBookRepository::new(pool));
    let r = repo.clone();
    let found = tokio::spawn(async move { r.get(1).await.unwrap() })
        .await
        .unwrap();
    assert_eq!(found.unwrap().title, "A Wizard of Earthsea");
}

// ---- Exercise 5: transactions ----------------------------------------------------------

#[tokio::test]
async fn ex5_lend_and_return() {
    use transactions::*;
    let pool = db::setup().await;
    let loan = lend_book(&pool, 2, 1).await.unwrap();
    assert_eq!(copies(&pool, 2).await.unwrap(), 0);
    assert_eq!(open_loans(&pool).await.unwrap(), 1);
    assert!(matches!(
        lend_book(&pool, 2, 1).await,
        Err(LoanError::NoCopiesLeft)
    ));
    assert!(matches!(
        lend_book(&pool, 999, 1).await,
        Err(LoanError::NoSuchBook)
    ));
    return_book(&pool, loan).await.unwrap();
    assert_eq!(copies(&pool, 2).await.unwrap(), 1);
    assert!(
        matches!(return_book(&pool, loan).await, Err(LoanError::NoSuchLoan)),
        "can't return twice"
    );
}

#[tokio::test]
async fn ex5_failure_halfway_rolls_back_everything() {
    use transactions::*;
    let pool = db::setup().await;
    let before = copies(&pool, 1).await.unwrap();
    assert!(
        matches!(lend_book(&pool, 1, 999).await, Err(LoanError::Database(_))),
        "unknown member: FK violation"
    );
    assert_eq!(
        copies(&pool, 1).await.unwrap(),
        before,
        "the decrement was rolled back"
    );
    assert_eq!(open_loans(&pool).await.unwrap(), 0);

    let _ = lend_book_without_transaction(&pool, 1, 999).await;
    assert_eq!(
        copies(&pool, 1).await.unwrap(),
        before - 1,
        "without a transaction, a copy is lost"
    );
}

// ---- Exercise 6: N+1 --------------------------------------------------------------------

#[tokio::test]
async fn ex6_three_strategies_same_answer_different_query_counts() {
    let pool = db::setup().await;
    queries::add_author(&pool, "No Books Yet", None)
        .await
        .unwrap();
    let (naive, qn) = n_plus_one::naive(&pool).await.unwrap();
    let (joined, qj) = n_plus_one::joined(&pool).await.unwrap();
    let (batched, qb) = n_plus_one::batched(&pool).await.unwrap();
    assert_eq!(naive, joined);
    assert_eq!(joined, batched);
    assert_eq!((qn, qj, qb), (5, 1, 2), "1 + N, one JOIN, parents + IN");
    assert_eq!(
        naive.last().unwrap().titles,
        Vec::<String>::new(),
        "authors without books still appear"
    );
    assert_eq!(naive[1].titles, vec!["Guards! Guards!", "Small Gods"]);
}

// ---- Exercise 7: indexes ------------------------------------------------------------------

#[tokio::test]
async fn ex7_index_turns_a_scan_into_a_search() {
    use indexes::*;
    let pool = db::setup().await;
    let before = query_plan(&pool, BOOKS_BY_YEAR, 1990).await.unwrap();
    assert!(
        before.iter().any(|d| d.starts_with("SCAN books")),
        "{before:?}"
    );
    add_indexes(&pool).await.unwrap();
    add_indexes(&pool).await.unwrap(); // IF NOT EXISTS
    let after = query_plan(&pool, BOOKS_BY_YEAR, 1990).await.unwrap();
    assert!(
        after
            .iter()
            .any(|d| d.contains("USING INDEX idx_books_year")),
        "{after:?}"
    );
    let by_author = query_plan(&pool, BOOKS_BY_AUTHOR, 1).await.unwrap();
    assert!(
        by_author
            .iter()
            .any(|d| d.contains("USING INDEX idx_books_author")),
        "{by_author:?}"
    );
}

#[tokio::test]
async fn ex7_bulk_insert_in_one_transaction() {
    let pool = db::setup().await;
    indexes::bulk_insert_books(&pool, 1_000).await.unwrap();
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM books")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 1_006);
}

// ---- Bonus: keyset pagination -------------------------------------------------------------

#[tokio::test]
async fn bonus_keyset_pages_cover_everything_in_order() {
    use keyset::*;
    let pool = db::setup().await;
    let (p1, c1) = books_after(&pool, None, 4).await.unwrap();
    assert_eq!(
        p1.iter().map(|b| b.year).collect::<Vec<_>>(),
        vec![1968, 1969, 1974, 1979]
    );
    let c1 = c1.unwrap();
    assert_eq!(c1.encode(), "1979:6");
    assert_eq!(Cursor::decode("1979:6"), Some(c1));
    assert_eq!(Cursor::decode("garbage"), None);

    // A new book inserted *before* the cursor doesn't shift the next page.
    sqlx::query("INSERT INTO books (author_id, title, year) VALUES (1, 'Early', 1950)")
        .execute(&pool)
        .await
        .unwrap();
    let (p2, c2) = books_after(&pool, Some(c1), 4).await.unwrap();
    assert_eq!(
        p2.iter().map(|b| b.title.as_str()).collect::<Vec<_>>(),
        vec!["Guards! Guards!", "Small Gods"]
    );
    assert_eq!(c2, None, "last page");
}
