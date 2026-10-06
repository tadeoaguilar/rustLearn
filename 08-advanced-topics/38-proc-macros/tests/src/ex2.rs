//! Exercise 2: #[derive(Table)] on real types -- and the SQL run in SQLite.

use crate::sut::Table;
use crate::sut::orm::{Column, OrmError, Table as _, Value};

#[derive(Table, Debug, Clone, PartialEq)]
#[table(name = "people")]
struct Person {
    #[column(primary_key)]
    id: i64,
    #[column(name = "full_name")]
    name: String,
    email: Option<String>,
    active: bool,
    score: f64,
    #[column(skip)]
    cached: u32,
}

#[derive(Table, Debug, PartialEq)]
struct BlogPost {
    #[column(primary_key)]
    slug: String,
    views: i32,
}

fn ana() -> Person {
    Person {
        id: 1,
        name: "Ana".into(),
        email: Some("ana@example.com".into()),
        active: true,
        score: 9.5,
        cached: 42,
    }
}

#[test]
fn ex2_metadata_and_sql() {
    assert_eq!(Person::TABLE, "people");
    assert_eq!(BlogPost::TABLE, "blog_posts");
    assert_eq!(
        Person::columns()[0],
        Column {
            name: "id",
            sql_type: "INTEGER",
            nullable: false,
            primary_key: true
        }
    );
    assert_eq!(
        Person::columns()[2],
        Column {
            name: "email",
            sql_type: "TEXT",
            nullable: true,
            primary_key: false
        }
    );
    assert_eq!(Person::columns().len(), 5, "skipped fields aren't columns");
    assert_eq!(
        Person::create_table_sql(),
        "CREATE TABLE people (id INTEGER PRIMARY KEY, full_name TEXT NOT NULL, email TEXT, active INTEGER NOT NULL, score REAL NOT NULL)"
    );
    assert_eq!(
        Person::insert_sql(),
        "INSERT INTO people (id, full_name, email, active, score) VALUES (?1, ?2, ?3, ?4, ?5)"
    );
    assert_eq!(
        BlogPost::select_by_key_sql(),
        "SELECT slug, views FROM blog_posts WHERE slug = ?1"
    );
}

#[test]
fn ex2_rows() {
    let values = ana().values();
    assert_eq!(
        values,
        [
            Value::Integer(1),
            Value::Text("Ana".into()),
            Value::Text("ana@example.com".into()),
            Value::Integer(1),
            Value::Real(9.5)
        ]
    );
    let back = Person::from_row(&values).unwrap();
    assert_eq!(
        back,
        Person { cached: 0, ..ana() },
        "a skipped field comes back as its default"
    );
    assert_eq!(
        Person::from_row(&values[..2]),
        Err(OrmError::WrongColumnCount {
            expected: 5,
            got: 2
        })
    );
    let mut bad = values.clone();
    bad[3] = Value::Text("yes".into());
    assert_eq!(
        Person::from_row(&bad),
        Err(OrmError::TypeMismatch {
            column: "active".into(),
            expected: "INTEGER 0 or 1"
        })
    );
}

/// The generated statements, executed by a real SQLite (in memory).
#[tokio::test]
async fn ex2_the_sql_runs_in_sqlite() {
    // The statements are built from compile-time metadata, never from input:
    // safe to mark as audited (SQLx 0.9 requires it for non-literal SQL).
    use sqlx::{AssertSqlSafe, Row};
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::query(AssertSqlSafe(Person::create_table_sql()))
        .execute(&pool)
        .await
        .unwrap();
    let people = [
        ana(),
        Person {
            id: 2,
            name: "Ben".into(),
            email: None,
            active: false,
            score: 0.0,
            cached: 0,
        },
    ];
    for person in &people {
        let mut query = sqlx::query(AssertSqlSafe(Person::insert_sql()));
        for value in person.values() {
            query = match value {
                Value::Null => query.bind(None::<i64>),
                Value::Integer(i) => query.bind(i),
                Value::Real(r) => query.bind(r),
                Value::Text(s) => query.bind(s),
            };
        }
        query.execute(&pool).await.unwrap();
    }
    let select = Person::select_by_key_sql();
    for person in &people {
        let row = sqlx::query(AssertSqlSafe(select.clone()))
            .bind(person.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        let values: Vec<Value> = Person::columns()
            .iter()
            .enumerate()
            .map(|(i, c)| match c.sql_type {
                "INTEGER" => row
                    .get::<Option<i64>, _>(i)
                    .map_or(Value::Null, Value::Integer),
                "REAL" => row
                    .get::<Option<f64>, _>(i)
                    .map_or(Value::Null, Value::Real),
                _ => row
                    .get::<Option<String>, _>(i)
                    .map_or(Value::Null, Value::Text),
            })
            .collect();
        assert_eq!(
            Person::from_row(&values).unwrap(),
            Person {
                cached: 0,
                ..person.clone()
            }
        );
    }
}
