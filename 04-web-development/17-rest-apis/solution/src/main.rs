// Reference solution for 17-rest-apis.
//
//     cargo run -p m17-rest-apis-solution -- demo     # a client exercises every feature
//     cargo run -p m17-rest-apis-solution -- serve    # http://127.0.0.1:3000 (try /openapi.json)

use m17_rest_apis_solution::api::{ApiConfig, app};
use m17_rest_apis_solution::client::{ClientError, TasksClient};
use m17_rest_apis_solution::model::{TaskInput, TaskStore};
use m17_rest_apis_solution::rate_limit::RateLimitConfig;

fn config() -> ApiConfig {
    ApiConfig {
        rate_limit: Some(RateLimitConfig {
            capacity: 20,
            refill_per_sec: 5.0,
        }),
        allowed_origins: vec!["http://localhost:5173".into()],
    }
}

async fn demo() {
    let base =
        m17_rest_apis_solution::spawn(app(TaskStore::with_generated(23), config()), "127.0.0.1:0")
            .await;
    let client = TasksClient::new(&base).with_api_key("demo");
    let http = reqwest::Client::new();

    let task = client
        .create(&TaskInput::new("Write the REST module"))
        .await
        .unwrap();
    println!("created: {task:?}");
    match client
        .create(&TaskInput {
            priority: 9,
            ..TaskInput::new("")
        })
        .await
    {
        Err(ClientError::Api(p)) => println!("invalid input -> {} {:?}", p.status, p.errors),
        other => println!("unexpected: {other:?}"),
    }

    let r = http
        .get(format!(
            "{base}/v1/tasks?per_page=5&sort=-priority&status=todo"
        ))
        .send()
        .await
        .unwrap();
    println!(
        "x-total-count: {:?}\nlink: {:?}",
        r.headers().get("x-total-count"),
        r.headers().get("link")
    );
    println!(
        "all tasks via Link-following client: {}",
        client.list_all(10).await.unwrap().len()
    );

    let csv = http
        .get(format!("{base}/v1/tasks?per_page=3"))
        .header("accept", "text/csv")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    println!("as CSV:\n{csv}");
    let v2: serde_json::Value = http
        .get(format!("{base}/v2/tasks/1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    println!("v2 representation: {v2}");

    let r = http.get(format!("{base}/v1/tasks/1")).send().await.unwrap();
    let etag = r.headers()["etag"].clone();
    let r = http
        .get(format!("{base}/v1/tasks/1"))
        .header("if-none-match", etag.clone())
        .send()
        .await
        .unwrap();
    println!("conditional GET with ETag {etag:?} -> {}", r.status());

    let doc: serde_json::Value = http
        .get(format!("{base}/openapi.json"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    println!(
        "OpenAPI: {} paths, title {:?}",
        doc["paths"].as_object().unwrap().len(),
        doc["info"]["title"]
    );

    let burst = TasksClient::new(&base).with_api_key("greedy");
    let mut limited = 0;
    for _ in 0..30 {
        if let Err(ClientError::RateLimited { .. }) = burst.get(1).await {
            limited += 1;
        }
    }
    println!("30 requests in a burst with capacity 20: {limited} were rate limited");
}

#[tokio::main]
async fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("serve") => {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
                .await
                .unwrap();
            println!("Tasks API on http://127.0.0.1:3000  -- /v1/tasks, /v2/tasks, /openapi.json");
            axum::serve(listener, app(TaskStore::with_generated(23), config()))
                .await
                .unwrap();
        }
        Some("demo") | Some("all") => demo().await,
        _ => {
            println!("17-rest-apis -- reference solution\n");
            println!(
                "  cargo run -p m17-rest-apis-solution -- demo    a client exercises every feature"
            );
            println!("  cargo run -p m17-rest-apis-solution -- serve   serve on :3000");
        }
    }
}
