// Reference solution for 19-authentication.
//
//     cargo run -p m19-authentication-solution -- demo    # every flow, against a local app + mock provider
//     cargo run -p m19-authentication-solution -- serve   # the app on :3000, mock provider on :3001

use m19_authentication_solution::app::{AppState, TokenPair, router, spawn_with_provider};
use m19_authentication_solution::totp;
use serde_json::{Value, json};

const SECRET: &[u8] = b"demo-only-secret-load-yours-from-config-32+bytes";

async fn demo() {
    let (base, _provider, _state) = spawn_with_provider(SECRET).await;
    let http = reqwest::Client::new();
    let post = |path: &str, body: Value| http.post(format!("{base}{path}")).json(&body).send();

    let r = post(
        "/register",
        json!({"username": "alice", "password": "correct horse battery"}),
    )
    .await
    .unwrap();
    println!(
        "register alice -> {} {}",
        r.status(),
        r.text().await.unwrap()
    );
    let r = post("/register", json!({"username": "bob", "password": "short"}))
        .await
        .unwrap();
    println!(
        "weak password  -> {} {}",
        r.status(),
        r.text().await.unwrap()
    );
    post(
        "/register",
        json!({"username": "bob", "password": "bob's long passphrase"}),
    )
    .await
    .unwrap();

    for (who, pw) in [
        ("alice", "wrong password!!"),
        ("nobody", "wrong password!!"),
    ] {
        let r = post("/login", json!({"username": who, "password": pw}))
            .await
            .unwrap();
        println!(
            "login {who:<6} bad -> {} {}  (same error for both: no user enumeration)",
            r.status(),
            r.text().await.unwrap()
        );
    }
    let alice: TokenPair = post(
        "/login",
        json!({"username": "alice", "password": "correct horse battery"}),
    )
    .await
    .unwrap()
    .json()
    .await
    .unwrap();
    let bob: TokenPair = post(
        "/login",
        json!({"username": "bob", "password": "bob's long passphrase"}),
    )
    .await
    .unwrap()
    .json()
    .await
    .unwrap();

    let me: Value = http
        .get(format!("{base}/me"))
        .bearer_auth(&alice.access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    println!("GET /me as alice -> {me}");
    for (who, t) in [("alice (admin)", &alice), ("bob (user)", &bob)] {
        let r = http
            .get(format!("{base}/admin/users"))
            .bearer_auth(&t.access_token)
            .send()
            .await
            .unwrap();
        println!("GET /admin/users as {who:<13} -> {}", r.status());
    }
    println!(
        "GET /me without token -> {}",
        http.get(format!("{base}/me"))
            .send()
            .await
            .unwrap()
            .status()
    );

    let fresh: TokenPair = post(
        "/token/refresh",
        json!({"refresh_token": alice.refresh_token}),
    )
    .await
    .unwrap()
    .json()
    .await
    .unwrap();
    let reuse = post(
        "/token/refresh",
        json!({"refresh_token": alice.refresh_token}),
    )
    .await
    .unwrap();
    println!(
        "refresh once -> new pair; reuse old refresh token -> {} {}",
        reuse.status(),
        reuse.text().await.unwrap()
    );

    let key: Value = http
        .post(format!("{base}/api-keys"))
        .bearer_auth(&fresh.access_token)
        .json(&json!({"label": "ci"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let key = key["key"].as_str().unwrap().to_string();
    let who: Value = http
        .get(format!("{base}/api/whoami"))
        .header("x-api-key", &key)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    println!("API key {}… -> {who}", &key[..10]);

    // Cookie session + CSRF
    let browser = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let r = browser
        .post(format!("{base}/session/login"))
        .form(&[("username", "bob"), ("password", "bob's long passphrase")])
        .send()
        .await
        .unwrap();
    println!(
        "session login -> Set-Cookie: {:?}",
        r.headers().get("set-cookie")
    );
    let csrf = r.json::<Value>().await.unwrap()["csrf_token"]
        .as_str()
        .unwrap()
        .to_string();
    let forged = browser
        .post(format!("{base}/session/display-name"))
        .form(&[("display_name", "pwned")])
        .send()
        .await
        .unwrap();
    println!("POST without CSRF token -> {}", forged.status());
    let ok = browser
        .post(format!("{base}/session/display-name"))
        .form(&[("display_name", "Bobby"), ("csrf_token", &csrf)])
        .send()
        .await
        .unwrap();
    println!(
        "POST with CSRF token    -> {} {}",
        ok.status(),
        ok.text().await.unwrap()
    );

    // OAuth: reqwest follows app -> provider -> app redirects like a browser would.
    let oauth: Value = http
        .get(format!("{base}/auth/oauth/login"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let me: Value = http
        .get(format!("{base}/me"))
        .bearer_auth(oauth["access_token"].as_str().unwrap())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    println!("OAuth login -> logged in as {me}");

    // MFA
    let enroll: Value = http
        .post(format!("{base}/mfa/enroll"))
        .bearer_auth(&bob.access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    println!("MFA enrolled: {}", enroll["otpauth_url"]);
    let r = post(
        "/login",
        json!({"username": "bob", "password": "bob's long passphrase"}),
    )
    .await
    .unwrap();
    println!(
        "login without code -> {} {}",
        r.status(),
        r.text().await.unwrap()
    );
    let secret = base32_decode(enroll["secret"].as_str().unwrap());
    let code = totp::totp(
        &secret,
        m19_authentication_solution::tokens::now() as u64,
        6,
    );
    let r = post(
        "/login",
        json!({"username": "bob", "password": "bob's long passphrase", "totp": code}),
    )
    .await
    .unwrap();
    println!("login with code {code} -> {}", r.status());
}

/// Just for the demo (the server never needs to decode base32).
fn base32_decode(s: &str) -> Vec<u8> {
    let (mut buffer, mut bits, mut out) = (0u32, 0u32, Vec::new());
    for c in s.chars() {
        let v = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567".find(c).unwrap() as u32;
        buffer = (buffer << 5) | v;
        bits += 5;
        if bits >= 8 {
            out.push((buffer >> (bits - 8)) as u8);
            bits -= 8;
        }
    }
    out
}

#[tokio::main]
async fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("demo") | Some("all") => demo().await,
        Some("serve") => {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
                .await
                .unwrap();
            println!("Auth app on http://127.0.0.1:3000 (no OAuth in this mode -- see `demo`)");
            axum::serve(listener, router(AppState::new(SECRET)))
                .await
                .unwrap();
        }
        _ => {
            println!("19-authentication -- reference solution\n");
            println!(
                "  cargo run -p m19-authentication-solution -- demo    every flow, end to end"
            );
            println!("  cargo run -p m19-authentication-solution -- serve   the app on :3000");
        }
    }
}
