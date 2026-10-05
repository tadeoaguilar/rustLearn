use crate::sut::*;
use notes::{NoteError, NoteFilter, NoteInput, NoteStore};
use serde_json::{Value, json};

fn input(title: &str, tags: &[&str]) -> NoteInput {
    NoteInput {
        title: title.into(),
        body: String::new(),
        tags: tags.iter().map(|t| t.to_string()).collect(),
    }
}

// ---- Exercise 1: the core --------------------------------------------------

#[test]
fn ex1_create_get_update_delete() {
    let store = NoteStore::new();
    let a = store
        .create(input("  First  ", &["Rust", "rust", " web "]))
        .unwrap();
    assert_eq!(a.id, 1);
    assert_eq!(a.title, "First", "trimmed");
    assert_eq!(
        a.tags,
        vec!["rust", "web"],
        "lower-cased, deduplicated, sorted"
    );
    assert_eq!(store.get(1).unwrap(), a);
    let b = store.update(1, input("Renamed", &[])).unwrap();
    assert_eq!((b.id, b.title.as_str()), (1, "Renamed"));
    store.delete(1).unwrap();
    assert_eq!(store.get(1), Err(NoteError::NotFound(1)));
    assert_eq!(store.delete(1), Err(NoteError::NotFound(1)));
    assert!(store.is_empty());
}

#[test]
fn ex1_validation() {
    let store = NoteStore::new();
    assert!(matches!(
        store.create(input("   ", &[])),
        Err(NoteError::Invalid(_))
    ));
    assert!(matches!(
        store.create(input(&"x".repeat(121), &[])),
        Err(NoteError::Invalid(_))
    ));
    assert!(store.create(input(&"x".repeat(120), &[])).is_ok());
    assert!(
        matches!(store.update(1, input("", &[])), Err(NoteError::Invalid(_))),
        "validated before lookup"
    );
}

#[test]
fn ex1_filtering_and_shared_clones() {
    let store = NoteStore::with_samples();
    let clone = store.clone();
    clone.create(input("Async in Axum", &["rust"])).unwrap();
    assert_eq!(store.len(), 4, "clones share the same notes");
    let by_tag = store.list(&NoteFilter {
        tag: Some("WEB".into()),
        q: None,
    });
    assert_eq!(by_tag.iter().map(|n| n.id).collect::<Vec<_>>(), vec![1, 2]);
    let by_text = store.list(&NoteFilter {
        tag: None,
        q: Some("AXUM".into()),
    });
    assert_eq!(by_text.len(), 2, "title or body, case-insensitive");
    let both = store.list(&NoteFilter {
        tag: Some("rust".into()),
        q: Some("eggs".into()),
    });
    assert!(both.is_empty());
}

// ---- Exercise 2: Axum -----------------------------------------------------

async fn axum_base() -> String {
    axum_app::spawn(axum_app::app(NoteStore::with_samples()), "127.0.0.1:0").await
}

#[tokio::test]
async fn ex2_axum_crud_and_status_codes() {
    let base = axum_base().await;
    let http = reqwest::Client::new();
    let health: Value = http
        .get(format!("{base}/health"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(health["status"], "ok");

    let r = http
        .post(format!("{base}/notes"))
        .json(&json!({"title": "New", "tags": ["x"]}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 201);
    assert_eq!(r.headers()["location"], "/notes/4");
    let created: Value = r.json().await.unwrap();
    assert_eq!(created["body"], "", "body defaults to empty");

    let list: Vec<Value> = http
        .get(format!("{base}/notes?tag=web"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list.len(), 2);

    let r = http
        .put(format!("{base}/notes/4"))
        .json(&json!({"title": "Changed"}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    assert_eq!(r.json::<Value>().await.unwrap()["title"], "Changed");

    assert_eq!(
        http.delete(format!("{base}/notes/4"))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    assert_eq!(
        http.get(format!("{base}/notes/4"))
            .send()
            .await
            .unwrap()
            .status(),
        404
    );
}

#[tokio::test]
async fn ex2_axum_errors_are_json() {
    let base = axum_base().await;
    let http = reqwest::Client::new();
    let r = http
        .post(format!("{base}/notes"))
        .json(&json!({"title": ""}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 422);
    let body: Value = r.json().await.unwrap();
    assert!(
        body["error"].as_str().unwrap().contains("title"),
        "got {body}"
    );
    let r = http.get(format!("{base}/notes/77")).send().await.unwrap();
    assert_eq!(
        r.json::<Value>().await.unwrap()["error"],
        "note 77 not found"
    );
    let r = http
        .get(format!("{base}/notes/not-a-number"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        r.status(),
        400,
        "the Path extractor rejects it before the handler runs"
    );
}

// ---- Exercise 3: Actix-web ------------------------------------------------

// Aliased: importing `actix_web::test` as `test` would shadow the built-in
// #[test] attribute used by the other tests in this file.
use actix_web::{App, test as atest, web};

#[actix_web::test]
async fn ex3_actix_crud() {
    let app = atest::init_service(
        App::new()
            .app_data(web::Data::new(NoteStore::with_samples()))
            .configure(actix_app::configure),
    )
    .await;

    let req = atest::TestRequest::post()
        .uri("/notes")
        .set_json(json!({"title": "From actix"}))
        .to_request();
    let resp = atest::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    assert_eq!(resp.headers().get("location").unwrap(), "/notes/4");

    let req = atest::TestRequest::get().uri("/notes?q=actix").to_request();
    let notes: Vec<Value> = atest::call_and_read_body_json(&app, req).await;
    assert_eq!(
        notes.len(),
        2,
        "the sample 'Actix-web' note and the new one"
    );

    let req = atest::TestRequest::put()
        .uri("/notes/4")
        .set_json(json!({"title": "Edited", "tags": ["A"]}))
        .to_request();
    let note: Value = atest::call_and_read_body_json(&app, req).await;
    assert_eq!(note["tags"], json!(["a"]));

    let req = atest::TestRequest::delete().uri("/notes/4").to_request();
    assert_eq!(atest::call_service(&app, req).await.status(), 204);
}

#[actix_web::test]
async fn ex3_actix_errors_match_axum() {
    let app = atest::init_service(
        App::new()
            .app_data(web::Data::new(NoteStore::new()))
            .configure(actix_app::configure),
    )
    .await;
    let resp =
        atest::call_service(&app, atest::TestRequest::get().uri("/notes/9").to_request()).await;
    assert_eq!(resp.status(), 404);
    let body: Value = atest::read_body_json(resp).await;
    assert_eq!(body["error"], "note 9 not found");
    let req = atest::TestRequest::post()
        .uri("/notes")
        .set_json(json!({"title": " "}))
        .to_request();
    assert_eq!(atest::call_service(&app, req).await.status(), 422);
}

#[tokio::test]
async fn ex3_actix_server_runs_on_tokio() {
    let (base, server) = actix_app::server(NoteStore::with_samples(), "127.0.0.1:0", 1).unwrap();
    tokio::spawn(server);
    let health: Value = reqwest::get(format!("{base}/health"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(health["framework"], "actix-web");
}

// ---- Exercise 4: middleware -----------------------------------------------

#[tokio::test]
async fn ex4_axum_request_id_and_timing() {
    let base = axum_base().await;
    let http = reqwest::Client::new();
    let r = http.get(format!("{base}/notes")).send().await.unwrap();
    let id = r
        .headers()
        .get("x-request-id")
        .expect("generated request id")
        .to_str()
        .unwrap()
        .to_string();
    assert_eq!(id.len(), 36, "a UUID");
    assert!(
        r.headers()
            .get("x-response-time-ms")
            .unwrap()
            .to_str()
            .unwrap()
            .parse::<f64>()
            .is_ok()
    );
    let r = http
        .get(format!("{base}/notes/404"))
        .header("x-request-id", "abc-123")
        .send()
        .await
        .unwrap();
    assert_eq!(
        r.headers()["x-request-id"],
        "abc-123",
        "echoed, even on errors"
    );
}

#[actix_web::test]
async fn ex4_actix_request_id() {
    let app = atest::init_service(
        App::new()
            .app_data(web::Data::new(NoteStore::new()))
            .wrap(actix_web::middleware::from_fn(middleware::actix_request_id))
            .configure(actix_app::configure),
    )
    .await;
    let resp = atest::call_service(
        &app,
        atest::TestRequest::get()
            .uri("/health")
            .insert_header(("x-request-id", "xyz"))
            .to_request(),
    )
    .await;
    assert_eq!(resp.headers().get("x-request-id").unwrap(), "xyz");
    assert!(resp.headers().get("x-response-time-ms").is_some());
}

// ---- Exercises 5 and 6: templates, forms, static files --------------------

#[test]
fn ex5_slugs_and_publishing() {
    use blog::{Blog, slugify};
    assert_eq!(slugify("Hello, World! 2024"), "hello-world-2024");
    let blog = Blog::new();
    assert_eq!(blog.publish("Same", "a").unwrap().slug, "same");
    assert_eq!(blog.publish("Same", "b").unwrap().slug, "same-2");
    assert_eq!(blog.publish("Same", "c").unwrap().slug, "same-3");
    assert!(blog.publish(" ", "body").is_err());
    assert!(blog.publish("!!!", "body").is_err());
    assert!(blog.publish("Title", "  ").is_err());
    let post = blog.find("same-2").unwrap();
    assert_eq!(post.body, "b");
}

#[tokio::test]
async fn ex5_html_pages_and_form_flow() {
    let base = axum_app::spawn(blog::router(blog::Blog::new()), "127.0.0.1:0").await;
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let index = http
        .get(format!("{base}/blog"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(index.contains("No posts yet"));

    let r = http
        .post(format!("{base}/blog"))
        .form(&[("title", "Hello <b>World</b>"), ("body", "One.\n\nTwo.")])
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 303, "Post/Redirect/Get");
    let location = r.headers()["location"].to_str().unwrap().to_string();
    assert_eq!(location, "/blog/hello-b-world-b");

    let page = http
        .get(format!("{base}{location}"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(
        !page.contains("<b>World</b>"),
        "HTML in the title must be escaped"
    );
    assert!(
        page.contains("<p>One.</p>") && page.contains("<p>Two.</p>"),
        "paragraphs"
    );

    let r = http
        .post(format!("{base}/blog"))
        .form(&[("title", ""), ("body", "kept text")])
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 422);
    let html = r.text().await.unwrap();
    assert!(html.contains("Title is required"));
    assert!(
        html.contains("kept text"),
        "the form keeps what the user typed"
    );

    assert_eq!(
        http.get(format!("{base}/blog/nope"))
            .send()
            .await
            .unwrap()
            .status(),
        404
    );
}

#[tokio::test]
async fn ex6_static_files() {
    let base = axum_app::spawn(blog::router(blog::Blog::new()), "127.0.0.1:0").await;
    let r = reqwest::get(format!("{base}/static/style.css"))
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    assert_eq!(r.headers()["content-type"], "text/css");
    assert_eq!(
        reqwest::get(format!("{base}/static/missing.css"))
            .await
            .unwrap()
            .status(),
        404
    );
}

// ---- Exercise 7: load testing ---------------------------------------------

#[tokio::test]
async fn ex7_load_test_reports_sensible_numbers() {
    let base = axum_base().await;
    let report = loadtest::load_test(&format!("{base}/health"), 200, 8).await;
    assert_eq!(report.requests, 200);
    assert_eq!(report.errors, 0);
    assert!(report.p50 <= report.p99);
    assert!(report.requests_per_second() > 0.0);
    let failing = loadtest::load_test(&format!("{base}/notes/999"), 20, 4).await;
    assert_eq!(failing.errors, 20, "non-2xx responses count as errors");
}
