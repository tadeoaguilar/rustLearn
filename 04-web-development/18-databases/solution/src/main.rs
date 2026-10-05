// Reference solution for 18-databases.
//
//     cargo run -p m18-databases-solution -- all           # every exercise, in memory
//     cargo run -p m18-databases-solution --release -- 7   # index timings on 200k rows

use m18_databases_solution::*;
use repository::{BookRepository, Catalog, NewBook, SqliteBookRepository};
use std::time::Instant;

async fn part1_2() {
    let pool = db::connect_memory().await.unwrap();
    db::migrate(&pool).await.unwrap();
    println!(
        "tables after migrations: {:?}",
        db::tables(&pool).await.unwrap()
    );
    let applied: Vec<(i64, String)> =
        sqlx::query_as("SELECT version, description FROM _sqlx_migrations ORDER BY version")
            .fetch_all(&pool)
            .await
            .unwrap();
    println!("applied migrations: {applied:?}");
    db::migrate(&pool).await.unwrap();
    println!("running migrate again is a no-op");
}

async fn part3() {
    let pool = db::setup().await;
    let author = queries::add_author(&pool, "N. K. Jemisin", Some(1972))
        .await
        .unwrap();
    println!("inserted with RETURNING: {author:?}");
    println!(
        "duplicate name -> {:?}",
        queries::add_author(&pool, "N. K. Jemisin", None)
            .await
            .map_err(|e| e.to_string())
    );
    println!(
        "search 'the': {:?}",
        queries::search_titles(&pool, "the").await.unwrap()
    );
    let attack = "x%' OR 1=1 --";
    println!(
        "bound parameter, attack string -> {:?}",
        queries::search_titles(&pool, attack).await.unwrap()
    );
    println!(
        "string concatenation, attack   -> {} titles (all of them!)",
        queries::search_titles_unsafe(&pool, attack)
            .await
            .unwrap()
            .len()
    );
    println!(
        "copies per author: {:?}",
        queries::copies_per_author(&pool).await.unwrap()
    );
}

async fn part4() {
    let pool = db::setup().await;
    let catalog = Catalog::new(SqliteBookRepository::new(pool), 2026);
    let book = catalog
        .add_book(NewBook {
            author_id: 2,
            title: " Mort ".into(),
            year: 1987,
            copies: 1,
            isbn: Some("978-0552131063".into()),
        })
        .await
        .unwrap();
    println!("added through the repository: {book:?}");
    let dup = NewBook {
        author_id: 2,
        title: "Mort again".into(),
        year: 1987,
        copies: 1,
        isbn: Some("978-0552131063".into()),
    };
    println!("same ISBN -> {}", catalog.add_book(dup).await.unwrap_err());
    let ghost = NewBook {
        author_id: 99,
        title: "Ghost".into(),
        year: 2000,
        copies: 1,
        isbn: None,
    };
    println!(
        "unknown author -> {}",
        catalog.add_book(ghost).await.unwrap_err()
    );
    println!(
        "restock -> {:?}",
        catalog.restock(book.id, 4).await.unwrap().copies
    );
    println!("Pratchett: {:?}", catalog.titles_by(2).await.unwrap());
    println!(
        "delete -> {}",
        catalog.repository().delete(book.id).await.unwrap()
    );
}

async fn part5() {
    let pool = db::setup().await;
    println!(
        "book 2 has {} copy",
        transactions::copies(&pool, 2).await.unwrap()
    );
    let loan = transactions::lend_book(&pool, 2, 1).await.unwrap();
    println!(
        "lent (loan {loan}); copies now {}",
        transactions::copies(&pool, 2).await.unwrap()
    );
    println!(
        "lend again -> {}",
        transactions::lend_book(&pool, 2, 2).await.unwrap_err()
    );
    transactions::return_book(&pool, loan).await.unwrap();
    println!(
        "returned; copies {}",
        transactions::copies(&pool, 2).await.unwrap()
    );

    let before = transactions::copies(&pool, 1).await.unwrap();
    let err = transactions::lend_book(&pool, 1, 999).await.unwrap_err();
    println!(
        "lend to unknown member -> {err}; copies {before} -> {} (rolled back)",
        transactions::copies(&pool, 1).await.unwrap()
    );
    let _ = transactions::lend_book_without_transaction(&pool, 1, 999).await;
    println!(
        "same WITHOUT a transaction -> copies now {} (a copy vanished)",
        transactions::copies(&pool, 1).await.unwrap()
    );
}

async fn part6() {
    let pool = db::setup().await;
    let (a, qa) = n_plus_one::naive(&pool).await.unwrap();
    let (b, qb) = n_plus_one::joined(&pool).await.unwrap();
    let (c, qc) = n_plus_one::batched(&pool).await.unwrap();
    println!(
        "naive: {qa} queries, join: {qb}, batched: {qc}; same result: {}",
        a == b && b == c
    );
    for x in &a {
        println!("  {}: {:?}", x.author, x.titles);
    }
}

async fn part7() {
    let pool = db::setup().await;
    indexes::bulk_insert_books(&pool, 200_000).await.unwrap();
    let time = |label: &'static str| {
        let pool = pool.clone();
        async move {
            let start = Instant::now();
            for year in 1900..1950 {
                let _: Vec<String> = sqlx::query_scalar(indexes::BOOKS_BY_YEAR)
                    .bind(year)
                    .fetch_all(&pool)
                    .await
                    .unwrap();
            }
            println!("{label:<14} 50 queries in {:?}", start.elapsed());
        }
    };
    println!(
        "plan before: {:?}",
        indexes::query_plan(&pool, indexes::BOOKS_BY_YEAR, 1990)
            .await
            .unwrap()
    );
    time("without index").await;
    indexes::add_indexes(&pool).await.unwrap();
    println!(
        "plan after:  {:?}",
        indexes::query_plan(&pool, indexes::BOOKS_BY_YEAR, 1990)
            .await
            .unwrap()
    );
    time("with index").await;
}

async fn bonus() {
    let pool = db::setup().await;
    let mut cursor = None;
    let mut page = 1;
    loop {
        let (books, next) = keyset::books_after(&pool, cursor, 4).await.unwrap();
        let titles: Vec<String> = books
            .iter()
            .map(|b| format!("{} ({})", b.title, b.year))
            .collect();
        println!(
            "page {page}: {titles:?}  next cursor: {:?}",
            next.map(|c| c.encode())
        );
        match next {
            Some(c) => cursor = Some(c),
            None => break,
        }
        page += 1;
    }
}

#[tokio::main]
async fn main() {
    let choice = std::env::args().nth(1).unwrap_or_default();
    let run = |k: &str| choice == k || choice == "all";
    if run("1") || run("2") {
        println!("\n=== EXERCISES 1-2: pool and migrations ===\n");
        part1_2().await;
    }
    if run("3") {
        println!("\n=== EXERCISE 3: queries ===\n");
        part3().await;
    }
    if run("4") {
        println!("\n=== EXERCISE 4: repository ===\n");
        part4().await;
    }
    if run("5") {
        println!("\n=== EXERCISE 5: transactions ===\n");
        part5().await;
    }
    if run("6") {
        println!("\n=== EXERCISE 6: N+1 ===\n");
        part6().await;
    }
    if run("7") {
        println!("\n=== EXERCISE 7: indexes (200k rows; try --release) ===\n");
        part7().await;
    }
    if run("bonus") {
        println!("\n=== BONUS: keyset pagination ===\n");
        bonus().await;
    }
    if !["1", "2", "3", "4", "5", "6", "7", "bonus", "all"].contains(&choice.as_str()) {
        println!("18-databases -- reference solution\n");
        println!("  cargo run -p m18-databases-solution -- <1..7|bonus|all>");
    }
}
