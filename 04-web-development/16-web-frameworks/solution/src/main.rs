// Reference solution for 16-web-frameworks.
//
//     cargo run -p m16-web-frameworks-solution -- demo      # both APIs on random ports, exercised by a client
//     cargo run -p m16-web-frameworks-solution -- axum      # Axum: notes API + blog on http://127.0.0.1:3000
//     cargo run -p m16-web-frameworks-solution -- actix     # Actix-web: notes API on http://127.0.0.1:3001
//     cargo run -p m16-web-frameworks-solution --release -- bench   # load-test both

use m16_web_frameworks_solution::{actix_app, axum_app, blog, loadtest, notes::NoteStore};

fn full_axum_app() -> axum::Router {
    axum_app::app(NoteStore::with_samples()).merge(blog::router(blog::Blog::new()))
}

async fn demo() {
    let axum_url = axum_app::spawn(full_axum_app(), "127.0.0.1:0").await;
    let (actix_url, server) =
        actix_app::server(NoteStore::with_samples(), "127.0.0.1:0", 2).expect("bind");
    tokio::spawn(server);

    let http = reqwest::Client::new();
    for (name, base) in [("axum", &axum_url), ("actix", &actix_url)] {
        println!("--- {name} at {base}");
        let health = http
            .get(format!("{base}/health"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        println!("GET /health            -> {health}");
        let r = http
            .post(format!("{base}/notes"))
            .json(&serde_json::json!({"title": "From the demo", "tags": ["Demo"]}))
            .send()
            .await
            .unwrap();
        println!(
            "POST /notes            -> {} Location: {:?}",
            r.status(),
            r.headers().get("location")
        );
        let r = http
            .get(format!("{base}/notes?tag=web"))
            .send()
            .await
            .unwrap();
        let generated_id = r.headers().get("x-request-id").cloned();
        let notes: Vec<serde_json::Value> = r.json().await.unwrap();
        println!(
            "GET /notes?tag=web     -> {} notes, generated x-request-id {generated_id:?}",
            notes.len()
        );
        let r = http
            .post(format!("{base}/notes"))
            .json(&serde_json::json!({"title": "  "}))
            .send()
            .await
            .unwrap();
        println!(
            "POST /notes (no title) -> {} {}",
            r.status(),
            r.text().await.unwrap()
        );
        let r = http
            .get(format!("{base}/notes/999"))
            .header("x-request-id", "demo-123")
            .send()
            .await
            .unwrap();
        println!(
            "GET /notes/999         -> {} x-request-id={:?}",
            r.status(),
            r.headers().get("x-request-id")
        );
    }
    println!("--- blog (axum)");
    let r = http
        .post(format!("{axum_url}/blog"))
        .form(&[
            ("title", "Hello <World>"),
            ("body", "First paragraph.\n\nSecond."),
        ])
        .send()
        .await
        .unwrap();
    println!(
        "POST /blog -> followed redirect to {} ({})",
        r.url().path(),
        r.status()
    );
    let html = r.text().await.unwrap();
    println!(
        "escaped title in page: {}",
        html.contains("Hello &#60;World&#62;") || html.contains("Hello &lt;World&gt;")
    );
    let css = http
        .get(format!("{axum_url}/static/style.css"))
        .send()
        .await
        .unwrap();
    println!(
        "GET /static/style.css -> {} {:?}",
        css.status(),
        css.headers().get("content-type")
    );
}

async fn bench() {
    let axum_url = axum_app::spawn(axum_app::app(NoteStore::with_samples()), "127.0.0.1:0").await;
    let (actix_url, server) =
        actix_app::server(NoteStore::with_samples(), "127.0.0.1:0", 4).expect("bind");
    tokio::spawn(server);
    for (name, base) in [("axum", axum_url), ("actix-web", actix_url)] {
        let url = format!("{base}/notes");
        let _ = loadtest::load_test(&url, 500, 10).await; // warm-up
        let r = loadtest::load_test(&url, 20_000, 50).await;
        println!(
            "{name:<10} {:>8.0} req/s  p50 {:>8.2?}  p99 {:>8.2?}  errors {}",
            r.requests_per_second(),
            r.p50,
            r.p99,
            r.errors
        );
    }
    println!(
        "(client and servers share this machine; compare the shape, not the absolute numbers)"
    );
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    match std::env::args().nth(1).as_deref() {
        Some("axum") => {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
            println!("Axum on http://127.0.0.1:3000  -- try /notes, /blog, /static/style.css");
            axum::serve(listener, full_axum_app()).await
        }
        Some("actix") => {
            let (url, server) = actix_app::server(NoteStore::with_samples(), "127.0.0.1:3001", 4)?;
            println!("Actix-web on {url}  -- try /notes");
            server.await
        }
        Some("bench") => {
            bench().await;
            Ok(())
        }
        Some("demo") | Some("all") => {
            demo().await;
            Ok(())
        }
        _ => {
            println!("16-web-frameworks -- reference solution\n");
            println!(
                "  cargo run -p m16-web-frameworks-solution -- demo             both APIs, exercised by a client"
            );
            println!(
                "  cargo run -p m16-web-frameworks-solution -- axum             serve Axum on :3000 (notes + blog)"
            );
            println!(
                "  cargo run -p m16-web-frameworks-solution -- actix            serve Actix-web on :3001"
            );
            println!(
                "  cargo run -p m16-web-frameworks-solution --release -- bench  load-test both"
            );
            Ok(())
        }
    }
}
