use crate::sut::*;
use api::{ApiConfig, app};
use model::{Status, TaskInput, TaskStore};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

async fn server(tasks: usize, config: ApiConfig) -> String {
    crate::sut::spawn(app(TaskStore::with_generated(tasks), config), "127.0.0.1:0").await
}

fn http() -> reqwest::Client {
    reqwest::Client::new()
}

// ---- Exercise 1: CRUD and status codes ---------------------------------------

#[tokio::test]
async fn ex1_crud_status_codes_and_headers() {
    let base = server(0, ApiConfig::default()).await;
    let r = http()
        .post(format!("{base}/v1/tasks"))
        .json(&json!({"title": "  Write tests  "}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 201);
    assert_eq!(r.headers()["location"], "/v1/tasks/1");
    let task: Value = r.json().await.unwrap();
    assert_eq!(
        task,
        json!({"id": 1, "title": "Write tests", "description": null, "status": "todo", "priority": 3, "version": 1})
    );

    let r = http()
        .put(format!("{base}/v1/tasks/1"))
        .json(&json!({"title": "Done", "status": "done", "priority": 5}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    assert_eq!(r.json::<Value>().await.unwrap()["version"], 2);

    assert_eq!(
        http()
            .delete(format!("{base}/v1/tasks/1"))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    assert_eq!(
        http()
            .delete(format!("{base}/v1/tasks/1"))
            .send()
            .await
            .unwrap()
            .status(),
        404
    );
    assert_eq!(
        http()
            .get(format!("{base}/v1/tasks/1"))
            .send()
            .await
            .unwrap()
            .status(),
        404
    );
}

#[test]
fn ex1_store() {
    let store = TaskStore::with_generated(6);
    assert_eq!(store.all().len(), 6);
    assert_eq!(store.get(1).unwrap().priority, 2);
    assert_eq!(store.get(5).unwrap().priority, 1, "priorities cycle 1..=5");
    let replaced = store.replace(1, TaskInput::new("new")).unwrap();
    assert_eq!((replaced.title.as_str(), replaced.version), ("new", 2));
    assert!(store.replace(99, TaskInput::new("x")).is_none());
}

// ---- Exercise 2: validation and problem+json ---------------------------------

#[tokio::test]
async fn ex2_validation_errors_are_problems_with_fields() {
    let base = server(0, ApiConfig::default()).await;
    let r = http()
        .post(format!("{base}/v1/tasks"))
        .json(&json!({"title": "", "priority": 9}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 422);
    assert_eq!(r.headers()["content-type"], "application/problem+json");
    let p: Value = r.json().await.unwrap();
    assert_eq!(p["status"], 422);
    assert!(p["type"].as_str().unwrap().ends_with("/validation"));
    assert!(p["errors"]["title"].is_array());
    assert_eq!(p["errors"]["priority"][0], "must be between 1 and 5");
}

#[tokio::test]
async fn ex2_malformed_bodies_and_unknown_routes() {
    let base = server(0, ApiConfig::default()).await;
    let r = http()
        .post(format!("{base}/v1/tasks"))
        .header("content-type", "application/json")
        .body("{not json")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 400);
    assert_eq!(r.headers()["content-type"], "application/problem+json");
    let r = http()
        .post(format!("{base}/v1/tasks"))
        .json(&json!({"title": "x", "status": "sleeping"}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 400, "an unknown enum value is a malformed body");
    let r = http()
        .get(format!("{base}/no/such/route"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 404);
    assert_eq!(r.headers()["content-type"], "application/problem+json");
}

#[test]
fn ex2_validator_rules() {
    use validator::Validate;
    assert!(TaskInput::new("ok").validate().is_ok());
    assert!(TaskInput::new(&"x".repeat(101)).validate().is_err());
    assert!(
        TaskInput {
            description: Some("d".repeat(1001)),
            ..TaskInput::new("t")
        }
        .validate()
        .is_err()
    );
    assert!(
        TaskInput {
            priority: 0,
            ..TaskInput::new("t")
        }
        .validate()
        .is_err()
    );
}

// ---- Exercise 3: pagination, filtering, sorting ------------------------------

#[tokio::test]
async fn ex3_pages_and_link_header() {
    let base = server(23, ApiConfig::default()).await;
    let r = http()
        .get(format!("{base}/v1/tasks?page=2&per_page=10"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.headers()["x-total-count"], "23");
    let link = r.headers()["link"].to_str().unwrap().to_string();
    assert!(
        link.contains("</v1/tasks?page=3&per_page=10>; rel=\"next\""),
        "{link}"
    );
    assert!(
        link.contains("</v1/tasks?page=1&per_page=10>; rel=\"prev\""),
        "{link}"
    );
    assert!(
        link.contains("</v1/tasks?page=3&per_page=10>; rel=\"last\""),
        "{link}"
    );
    let page: Value = r.json().await.unwrap();
    assert_eq!(
        (
            page["page"].clone(),
            page["per_page"].clone(),
            page["total"].clone()
        ),
        (json!(2), json!(10), json!(23))
    );
    let ids: Vec<u64> = page["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["id"].as_u64().unwrap())
        .collect();
    assert_eq!(ids, (11..=20).collect::<Vec<_>>());

    let last: Value = http()
        .get(format!("{base}/v1/tasks?page=3&per_page=10"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(last["data"].as_array().unwrap().len(), 3);
    let r = http()
        .get(format!("{base}/v1/tasks?page=3&per_page=10"))
        .send()
        .await
        .unwrap();
    assert!(
        !r.headers()["link"]
            .to_str()
            .unwrap()
            .contains("rel=\"next\""),
        "no next on the last page"
    );
}

#[tokio::test]
async fn ex3_filter_and_sort() {
    let base = server(23, ApiConfig::default()).await;
    let page: Value = http()
        .get(format!(
            "{base}/v1/tasks?status=done&sort=-priority&per_page=100"
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let data = page["data"].as_array().unwrap();
    assert!(data.iter().all(|t| t["status"] == "done"));
    let prios: Vec<u64> = data
        .iter()
        .map(|t| t["priority"].as_u64().unwrap())
        .collect();
    assert!(
        prios.windows(2).all(|w| w[0] >= w[1]),
        "descending: {prios:?}"
    );
    let r = http()
        .get(format!("{base}/v1/tasks?status=done&per_page=2"))
        .send()
        .await
        .unwrap();
    assert!(
        r.headers()["link"]
            .to_str()
            .unwrap()
            .contains("status=done"),
        "links keep the filters"
    );
}

#[tokio::test]
async fn ex3_bad_parameters_are_400() {
    let base = server(3, ApiConfig::default()).await;
    for q in [
        "page=0",
        "per_page=0",
        "per_page=101",
        "sort=color",
        "status=bogus",
    ] {
        let r = http()
            .get(format!("{base}/v1/tasks?{q}"))
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), 400, "{q}");
    }
}

#[test]
fn ex3_paginate_unit() {
    use pagination::{ListQuery, paginate};
    let store = TaskStore::with_generated(5);
    let q = ListQuery {
        page: Some(1),
        per_page: Some(2),
        status: None,
        sort: Some("-id".into()),
    };
    let page = paginate(store.all(), &q);
    assert_eq!(
        page.data.iter().map(|t| t.id).collect::<Vec<_>>(),
        vec![5, 4]
    );
    let q = ListQuery {
        page: Some(9),
        per_page: Some(2),
        status: Some(Status::Todo),
        sort: None,
    };
    let page = paginate(store.all(), &q);
    assert!(
        page.data.is_empty(),
        "a page past the end is empty, not an error"
    );
    assert_eq!(page.total, 1);
}

// ---- Exercise 4: versioning ---------------------------------------------------

#[tokio::test]
async fn ex4_v2_representation() {
    let base = server(5, ApiConfig::default()).await;
    let v1: Value = http()
        .get(format!("{base}/v1/tasks/5"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let v2: Value = http()
        .get(format!("{base}/v2/tasks/5"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(v1["priority"], 1);
    assert_eq!(v2["priority"], "lowest");
    assert_eq!(v2["links"]["self"], "/v2/tasks/5");
    assert!(v2.get("version").is_none());
    let list: Value = http()
        .get(format!("{base}/v2/tasks?sort=-priority&per_page=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list["data"][0]["priority"], "highest");
}

// ---- Exercise 5: OpenAPI -------------------------------------------------------

#[tokio::test]
async fn ex5_openapi_document() {
    let base = server(0, ApiConfig::default()).await;
    let doc: Value = http()
        .get(format!("{base}/openapi.json"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(doc["openapi"].as_str().unwrap().starts_with("3."));
    for path in ["/v1/tasks", "/v1/tasks/{id}", "/v2/tasks", "/v2/tasks/{id}"] {
        assert!(doc["paths"][path].is_object(), "missing {path}");
    }
    for method in ["get", "put", "delete"] {
        assert!(
            doc["paths"]["/v1/tasks/{id}"][method].is_object(),
            "missing {method}"
        );
    }
    for schema in ["Task", "TaskInput", "Problem", "TaskV2"] {
        assert!(
            doc["components"]["schemas"][schema].is_object(),
            "missing schema {schema}"
        );
    }
}

// ---- Exercise 6: rate limiting -------------------------------------------------

#[test]
fn ex6_token_bucket_logic() {
    use rate_limit::{Decision, RateLimitConfig, RateLimiter};
    let rl = RateLimiter::new(RateLimitConfig {
        capacity: 2,
        refill_per_sec: 1.0,
    });
    let t0 = Instant::now();
    assert!(matches!(
        rl.check_at("a", t0),
        Decision::Allowed { remaining: 1 }
    ));
    assert!(matches!(
        rl.check_at("a", t0),
        Decision::Allowed { remaining: 0 }
    ));
    assert!(matches!(
        rl.check_at("a", t0),
        Decision::Limited {
            retry_after_secs: 1
        }
    ));
    assert!(
        matches!(rl.check_at("b", t0), Decision::Allowed { .. }),
        "buckets are per client"
    );
    assert!(
        matches!(
            rl.check_at("a", t0 + Duration::from_millis(1_100)),
            Decision::Allowed { .. }
        ),
        "refilled"
    );
    assert!(
        matches!(
            rl.check_at("a", t0 + Duration::from_secs(100)),
            Decision::Allowed { remaining: 1 }
        ),
        "never above capacity"
    );
}

#[tokio::test]
async fn ex6_middleware_returns_429_with_retry_after() {
    let config = ApiConfig {
        rate_limit: Some(rate_limit::RateLimitConfig {
            capacity: 3,
            refill_per_sec: 0.5,
        }),
        ..Default::default()
    };
    let base = server(1, config).await;
    let statuses: Vec<u16> = {
        let mut v = Vec::new();
        for _ in 0..4 {
            v.push(
                http()
                    .get(format!("{base}/v1/tasks/1"))
                    .header("x-api-key", "k1")
                    .send()
                    .await
                    .unwrap()
                    .status()
                    .as_u16(),
            );
        }
        v
    };
    assert_eq!(statuses, vec![200, 200, 200, 429]);
    let r = http()
        .get(format!("{base}/v1/tasks/1"))
        .header("x-api-key", "k1")
        .send()
        .await
        .unwrap();
    assert_eq!(r.headers()["retry-after"], "2");
    assert_eq!(r.headers()["x-ratelimit-remaining"], "0");
    let other = http()
        .get(format!("{base}/v1/tasks/1"))
        .header("x-api-key", "k2")
        .send()
        .await
        .unwrap();
    assert_eq!(other.status(), 200);
    assert_eq!(other.headers()["x-ratelimit-remaining"], "2");
    for _ in 0..5 {
        assert_eq!(
            http()
                .get(format!("{base}/health"))
                .send()
                .await
                .unwrap()
                .status(),
            200,
            "/health isn't limited"
        );
    }
}

// ---- Exercise 7: CORS and content negotiation ----------------------------------

#[tokio::test]
async fn ex7_cors_preflight() {
    let config = ApiConfig {
        allowed_origins: vec!["http://localhost:5173".into()],
        ..Default::default()
    };
    let base = server(1, config).await;
    let r = http()
        .request(reqwest::Method::OPTIONS, format!("{base}/v1/tasks"))
        .header("origin", "http://localhost:5173")
        .header("access-control-request-method", "POST")
        .header("access-control-request-headers", "content-type")
        .send()
        .await
        .unwrap();
    assert!(r.status().is_success());
    assert_eq!(
        r.headers()["access-control-allow-origin"],
        "http://localhost:5173"
    );
    let evil = http()
        .get(format!("{base}/v1/tasks"))
        .header("origin", "http://evil.example")
        .send()
        .await
        .unwrap();
    assert!(
        evil.headers().get("access-control-allow-origin").is_none(),
        "unknown origins get no CORS header"
    );
    let ok = http()
        .get(format!("{base}/v1/tasks"))
        .header("origin", "http://localhost:5173")
        .send()
        .await
        .unwrap();
    assert!(
        ok.headers()["access-control-expose-headers"]
            .to_str()
            .unwrap()
            .to_lowercase()
            .contains("link")
    );
}

#[tokio::test]
async fn ex7_content_negotiation() {
    let base = server(2, ApiConfig::default()).await;
    let csv = http()
        .get(format!("{base}/v1/tasks"))
        .header("accept", "text/csv")
        .send()
        .await
        .unwrap();
    assert!(
        csv.headers()["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/csv")
    );
    assert_eq!(
        csv.text().await.unwrap(),
        "id,title,status,priority\n1,Task 1,in_progress,2\n2,Task 2,done,3\n"
    );
    let pref = http()
        .get(format!("{base}/v1/tasks"))
        .header("accept", "text/csv;q=0.5, application/json")
        .send()
        .await
        .unwrap();
    assert!(
        pref.headers()["content-type"]
            .to_str()
            .unwrap()
            .starts_with("application/json"),
        "q-values decide"
    );
    let xml = http()
        .get(format!("{base}/v1/tasks"))
        .header("accept", "application/xml")
        .send()
        .await
        .unwrap();
    assert_eq!(xml.status(), 406);
}

// ---- Exercise 8: client SDK ------------------------------------------------------

#[tokio::test]
async fn ex8_client_round_trip_and_errors() {
    use client::{ClientError, TasksClient};
    let base = server(0, ApiConfig::default()).await;
    let c = TasksClient::new(&format!("{base}/"));
    let t = c.create(&TaskInput::new("sdk")).await.unwrap();
    assert_eq!(c.get(t.id).await.unwrap(), t);
    let updated = c
        .replace(
            t.id,
            &TaskInput {
                status: Status::Done,
                ..TaskInput::new("sdk")
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.status, Status::Done);
    c.delete(t.id).await.unwrap();
    match c.get(t.id).await {
        Err(ClientError::Api(p)) => assert_eq!(p.status, 404),
        other => panic!("expected a 404 problem, got {other:?}"),
    }
    match c.create(&TaskInput::new("")).await {
        Err(ClientError::Api(p)) => assert!(p.errors.contains_key("title")),
        other => panic!("expected a validation problem, got {other:?}"),
    }
}

#[tokio::test]
async fn ex8_client_follows_links_and_reports_rate_limits() {
    use client::{ClientError, TasksClient, next_link};
    let base = server(23, ApiConfig::default()).await;
    let all = TasksClient::new(&base).list_all(5).await.unwrap();
    assert_eq!(
        all.iter().map(|t| t.id).collect::<Vec<_>>(),
        (1..=23).collect::<Vec<_>>()
    );
    assert_eq!(
        next_link("<a>; rel=\"prev\", <b?page=2>; rel=\"next\""),
        Some("b?page=2".into())
    );
    assert_eq!(next_link("<a>; rel=\"last\""), None);

    let config = ApiConfig {
        rate_limit: Some(rate_limit::RateLimitConfig {
            capacity: 1,
            refill_per_sec: 0.2,
        }),
        ..Default::default()
    };
    let limited = TasksClient::new(&server(1, config).await).with_api_key("x");
    limited.get(1).await.unwrap();
    assert!(matches!(
        limited.get(1).await,
        Err(ClientError::RateLimited {
            retry_after_secs: 5
        })
    ));
}

// ---- Bonus: ETags ------------------------------------------------------------------

#[tokio::test]
async fn bonus_conditional_get_and_optimistic_locking() {
    let base = server(1, ApiConfig::default()).await;
    let r = http()
        .get(format!("{base}/v1/tasks/1"))
        .send()
        .await
        .unwrap();
    let etag = r.headers()["etag"].to_str().unwrap().to_string();
    assert_eq!(etag, "\"1-1\"");
    let r = http()
        .get(format!("{base}/v1/tasks/1"))
        .header("if-none-match", &etag)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 304);
    assert!(r.bytes().await.unwrap().is_empty());

    let body = json!({"title": "edited"});
    let ok = http()
        .put(format!("{base}/v1/tasks/1"))
        .header("if-match", &etag)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(ok.status(), 200);
    assert_eq!(ok.headers()["etag"], "\"1-2\"");
    let stale = http()
        .put(format!("{base}/v1/tasks/1"))
        .header("if-match", &etag)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(
        stale.status(),
        412,
        "a write based on an old version is refused"
    );
    let fresh = http()
        .get(format!("{base}/v1/tasks/1"))
        .header("if-none-match", &etag)
        .send()
        .await
        .unwrap();
    assert_eq!(fresh.status(), 200, "the old ETag no longer matches");
}
