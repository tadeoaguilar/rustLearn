use crate::sut::*;
use app::{AppState, TokenPair, router, spawn_with_provider};
use rbac::{Permission, Role};
use serde_json::{Value, json};
use tokens::{TokenError, TokenService, TokenType};

const SECRET: &[u8] = b"test-secret-test-secret-test-secret!";
const ALICE_PW: &str = "correct horse battery";
const BOB_PW: &str = "bob's long passphrase";

async fn app() -> String {
    crate::sut::spawn(router(AppState::new(SECRET)), "127.0.0.1:0").await
}

async fn post(base: &str, path: &str, body: Value) -> reqwest::Response {
    reqwest::Client::new()
        .post(format!("{base}{path}"))
        .json(&body)
        .send()
        .await
        .unwrap()
}

/// alice (admin, registered first) and bob (user), both logged in.
async fn two_users(base: &str) -> (TokenPair, TokenPair) {
    post(
        base,
        "/register",
        json!({"username": "alice", "password": ALICE_PW}),
    )
    .await;
    post(
        base,
        "/register",
        json!({"username": "bob", "password": BOB_PW}),
    )
    .await;
    let a = post(
        base,
        "/login",
        json!({"username": "alice", "password": ALICE_PW}),
    )
    .await
    .json()
    .await
    .unwrap();
    let b = post(
        base,
        "/login",
        json!({"username": "bob", "password": BOB_PW}),
    )
    .await
    .json()
    .await
    .unwrap();
    (a, b)
}

// ---- Exercise 1: passwords, registration, login -----------------------------------------

#[test]
fn ex1_hashing() {
    use password::*;
    let h1 = hash_password("correct horse battery");
    let h2 = hash_password("correct horse battery");
    assert!(h1.starts_with("$argon2id$"), "PHC string: {h1}");
    assert_ne!(h1, h2, "a fresh salt each time");
    assert!(verify_password("correct horse battery", &h1));
    assert!(!verify_password("correct horse batterY", &h1));
    assert!(!verify_password("anything", "not a hash"));
}

#[test]
fn ex1_policy() {
    use password::check_policy;
    assert!(check_policy("short").is_err());
    assert!(
        check_policy("Password1234").is_err(),
        "common, case-insensitively"
    );
    assert!(check_policy(&"x".repeat(129)).is_err());
    assert!(check_policy("a long but memorable phrase").is_ok());
    assert!(
        check_policy("ñandú ñandú ñandú").is_ok(),
        "length counts characters, not bytes"
    );
}

#[test]
fn ex1_user_store() {
    use users::{UserError, UserStore};
    let store = UserStore::new();
    assert_eq!(
        store.register("Alice", ALICE_PW).unwrap().role,
        Role::Admin,
        "first user is admin"
    );
    assert_eq!(store.register("bob", BOB_PW).unwrap().role, Role::User);
    assert_eq!(
        store.register("ALICE", ALICE_PW).unwrap_err(),
        UserError::Taken,
        "usernames are case-insensitive"
    );
    assert!(matches!(
        store.register("a!", ALICE_PW),
        Err(UserError::InvalidUsername(_))
    ));
    assert!(matches!(
        store.register("carol", "short"),
        Err(UserError::WeakPassword(_))
    ));
    assert_eq!(
        store.authenticate("alice", ALICE_PW).unwrap().username,
        "alice"
    );
    assert_eq!(
        store.authenticate("alice", "wrong password!!").unwrap_err(),
        UserError::InvalidCredentials
    );
    assert_eq!(
        store.authenticate("nobody", ALICE_PW).unwrap_err(),
        UserError::InvalidCredentials,
        "same error"
    );
}

#[tokio::test]
async fn ex1_http_register_and_login() {
    let base = app().await;
    let r = post(
        &base,
        "/register",
        json!({"username": "alice", "password": ALICE_PW}),
    )
    .await;
    assert_eq!(r.status(), 201);
    assert_eq!(
        post(
            &base,
            "/register",
            json!({"username": "alice", "password": ALICE_PW})
        )
        .await
        .status(),
        409
    );
    assert_eq!(
        post(
            &base,
            "/register",
            json!({"username": "zed", "password": "short"})
        )
        .await
        .status(),
        400
    );
    let bad1: Value = post(
        &base,
        "/login",
        json!({"username": "alice", "password": "nope nope nope"}),
    )
    .await
    .json()
    .await
    .unwrap();
    let bad2: Value = post(
        &base,
        "/login",
        json!({"username": "ghost", "password": "nope nope nope"}),
    )
    .await
    .json()
    .await
    .unwrap();
    assert_eq!(bad1, bad2, "no user enumeration");
    let ok: TokenPair = post(
        &base,
        "/login",
        json!({"username": "alice", "password": ALICE_PW}),
    )
    .await
    .json()
    .await
    .unwrap();
    assert_eq!(ok.token_type, "Bearer");
    assert_eq!(ok.expires_in, 900);
}

// ---- Exercise 2: JWTs -------------------------------------------------------------------

#[test]
fn ex2_token_verification() {
    let svc = TokenService::new(SECRET);
    let access = svc.issue(7, "ann", Role::User, TokenType::Access, 60);
    let claims = svc.verify(&access, TokenType::Access).unwrap();
    assert_eq!(
        (claims.sub, claims.username.as_str(), claims.role),
        (7, "ann", Role::User)
    );
    assert_eq!(
        svc.verify(&access, TokenType::Refresh),
        Err(TokenError::WrongType)
    );
    let expired = svc.issue(7, "ann", Role::User, TokenType::Access, -1);
    assert_eq!(
        svc.verify(&expired, TokenType::Access),
        Err(TokenError::Expired),
        "no leeway"
    );
    let other_key = TokenService::new(b"a-completely-different-secret-key!!");
    assert_eq!(
        other_key.verify(&access, TokenType::Access),
        Err(TokenError::Invalid)
    );
    let mut tampered = access.clone();
    tampered.pop();
    assert_eq!(
        svc.verify(&tampered, TokenType::Access),
        Err(TokenError::Invalid)
    );
}

#[test]
fn ex2_alg_none_is_rejected() {
    let svc = TokenService::new(SECRET);
    let access = svc.issue(1, "eve", Role::Admin, TokenType::Access, 60);
    let claims_part = access.split('.').nth(1).unwrap();
    // header {"alg":"none","typ":"JWT"}, no signature
    let forged = format!("eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.{claims_part}.");
    assert_eq!(
        svc.verify(&forged, TokenType::Access),
        Err(TokenError::Invalid)
    );
}

#[test]
fn ex2_refresh_rotation_detects_reuse() {
    let svc = TokenService::new(SECRET);
    let (_, refresh) = svc.issue_pair(1, "ann", Role::User);
    let (_, refresh2) = svc.refresh(&refresh).unwrap();
    assert_eq!(
        svc.refresh(&refresh),
        Err(TokenError::Revoked),
        "a refresh token works once"
    );
    assert!(svc.refresh(&refresh2).is_ok());
}

#[tokio::test]
async fn ex2_http_bearer_refresh_logout() {
    let base = app().await;
    let (alice, _) = two_users(&base).await;
    let http = reqwest::Client::new();
    let r = http.get(format!("{base}/me")).send().await.unwrap();
    assert_eq!(r.status(), 401);
    assert_eq!(r.headers()["www-authenticate"], "Bearer");
    let me: Value = http
        .get(format!("{base}/me"))
        .bearer_auth(&alice.access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(me["username"], "alice");
    assert_eq!(
        http.get(format!("{base}/me"))
            .bearer_auth(&alice.refresh_token)
            .send()
            .await
            .unwrap()
            .status(),
        401,
        "refresh tokens can't call the API"
    );

    let new: TokenPair = post(
        &base,
        "/token/refresh",
        json!({"refresh_token": alice.refresh_token}),
    )
    .await
    .json()
    .await
    .unwrap();
    assert_eq!(
        post(
            &base,
            "/token/refresh",
            json!({"refresh_token": alice.refresh_token})
        )
        .await
        .status(),
        401
    );

    let r = http
        .post(format!("{base}/logout"))
        .bearer_auth(&new.access_token)
        .json(&json!({"refresh_token": new.refresh_token}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 204);
    assert_eq!(
        http.get(format!("{base}/me"))
            .bearer_auth(&new.access_token)
            .send()
            .await
            .unwrap()
            .status(),
        401,
        "access token revoked"
    );
    assert_eq!(
        post(
            &base,
            "/token/refresh",
            json!({"refresh_token": new.refresh_token})
        )
        .await
        .status(),
        401,
        "refresh token revoked"
    );
}

// ---- Exercise 3: RBAC ---------------------------------------------------------------------

#[test]
fn ex3_roles_are_bundles_of_permissions() {
    assert!(Role::User.can(Permission::ReadOwnProfile));
    assert!(!Role::User.can(Permission::ReadReports));
    assert!(Role::Moderator.can(Permission::ReadReports));
    assert!(!Role::Moderator.can(Permission::ManageUsers));
    assert!(Role::Admin.can(Permission::ManageUsers));
}

#[tokio::test]
async fn ex3_http_403_vs_401_and_role_changes() {
    let base = app().await;
    let (alice, bob) = two_users(&base).await;
    let http = reqwest::Client::new();
    assert_eq!(
        http.get(format!("{base}/admin/users"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        http.get(format!("{base}/admin/users"))
            .bearer_auth(&bob.access_token)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        http.get(format!("{base}/reports"))
            .bearer_auth(&bob.access_token)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    let users: Value = http
        .get(format!("{base}/admin/users"))
        .bearer_auth(&alice.access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(users.as_array().unwrap().len(), 2);

    let r = http
        .put(format!("{base}/admin/users/2/role"))
        .bearer_auth(&alice.access_token)
        .json(&json!({"role": "moderator"}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    assert_eq!(
        http.get(format!("{base}/reports"))
            .bearer_auth(&bob.access_token)
            .send()
            .await
            .unwrap()
            .status(),
        403,
        "old token, old role"
    );
    let bob2: TokenPair = post(
        &base,
        "/login",
        json!({"username": "bob", "password": BOB_PW}),
    )
    .await
    .json()
    .await
    .unwrap();
    assert_eq!(
        http.get(format!("{base}/reports"))
            .bearer_auth(&bob2.access_token)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let r = http
        .put(format!("{base}/admin/users/1/role"))
        .bearer_auth(&alice.access_token)
        .json(&json!({"role": "user"}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 400, "an admin can't demote themselves");
}

// ---- Exercise 4: sessions and CSRF ---------------------------------------------------------

#[tokio::test]
async fn ex4_session_cookie_flags_and_csrf() {
    let base = app().await;
    two_users(&base).await;
    let browser = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let r = browser
        .post(format!("{base}/session/login"))
        .form(&[("username", "bob"), ("password", BOB_PW)])
        .send()
        .await
        .unwrap();
    let cookie = r.headers()["set-cookie"].to_str().unwrap().to_string();
    assert!(cookie.starts_with("session="));
    assert!(
        cookie.contains("HttpOnly") && cookie.contains("SameSite=Lax") && cookie.contains("Path=/"),
        "{cookie}"
    );
    let csrf = r.json::<Value>().await.unwrap()["csrf_token"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(csrf.len(), 64);

    let me: Value = browser
        .get(format!("{base}/session/me"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(me["username"], "bob");

    let no_token = browser
        .post(format!("{base}/session/display-name"))
        .form(&[("display_name", "evil")])
        .send()
        .await
        .unwrap();
    assert_eq!(no_token.status(), 403);
    let wrong = browser
        .post(format!("{base}/session/display-name"))
        .form(&[("display_name", "evil"), ("csrf_token", &"0".repeat(64))])
        .send()
        .await
        .unwrap();
    assert_eq!(wrong.status(), 403);
    let ok = browser
        .post(format!("{base}/session/display-name"))
        .form(&[("display_name", "Bobby"), ("csrf_token", &csrf)])
        .send()
        .await
        .unwrap();
    assert_eq!(ok.status(), 200);
    let me: Value = browser
        .get(format!("{base}/session/me"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(me["display_name"], "Bobby");

    let out = browser
        .post(format!("{base}/session/logout"))
        .form(&[("csrf_token", &csrf)])
        .send()
        .await
        .unwrap();
    assert_eq!(out.status(), 204);
    assert_eq!(
        browser
            .get(format!("{base}/session/me"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
}

#[tokio::test]
async fn ex4_no_cookie_or_bad_login() {
    let base = app().await;
    two_users(&base).await;
    assert_eq!(
        reqwest::get(format!("{base}/session/me"))
            .await
            .unwrap()
            .status(),
        401
    );
    let r = reqwest::Client::new()
        .post(format!("{base}/session/login"))
        .form(&[("username", "bob"), ("password", "nope nope nope")])
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 401);
    assert!(r.headers().get("set-cookie").is_none());
}

#[test]
fn ex4_session_store_expiry_and_constant_time_compare() {
    use sessions::{SessionStore, tokens_match};
    let store = SessionStore::new();
    let (id, csrf) = store.create(9);
    assert_eq!(store.get(&id).unwrap().csrf_token, csrf);
    store.expire_now(&id);
    assert!(store.get(&id).is_none());
    assert!(tokens_match("abc", "abc"));
    assert!(!tokens_match("abc", "abd"));
    assert!(!tokens_match("abc", "abcd"));
}

// ---- Exercise 5: API keys ----------------------------------------------------------------

#[test]
fn ex5_keys_are_stored_hashed() {
    let store = api_keys::ApiKeyStore::new();
    let key = store.create(3, "ci");
    assert!(key.starts_with("rlk_") && key.len() == 52);
    assert_eq!(store.verify(&key).unwrap().user_id, 3);
    assert!(
        !store.stored_hashes().contains(&key),
        "plaintext is never stored"
    );
    assert_eq!(store.stored_hashes(), vec![api_keys::hash_key(&key)]);
    assert!(store.verify("rlk_wrong").is_none());
    assert!(store.revoke(&key));
    assert!(store.verify(&key).is_none());
}

#[tokio::test]
async fn ex5_http_api_keys() {
    let base = app().await;
    let (alice, _) = two_users(&base).await;
    let http = reqwest::Client::new();
    let r = http
        .post(format!("{base}/api-keys"))
        .bearer_auth(&alice.access_token)
        .json(&json!({"label": "ci"}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 201);
    let key = r.json::<Value>().await.unwrap()["key"]
        .as_str()
        .unwrap()
        .to_string();
    let who: Value = http
        .get(format!("{base}/api/whoami"))
        .header("x-api-key", &key)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        (who["username"].as_str(), who["key_label"].as_str()),
        (Some("alice"), Some("ci"))
    );
    assert_eq!(
        http.get(format!("{base}/api/whoami"))
            .header("x-api-key", "rlk_nope")
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        http.get(format!("{base}/api/whoami"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
}

// ---- Exercise 6: OAuth2 -------------------------------------------------------------------

#[tokio::test]
async fn ex6_authorization_code_flow_with_pkce() {
    let (base, _provider, _) = spawn_with_provider(SECRET).await;
    let http = reqwest::Client::new(); // follows redirects: app -> provider -> app
    let r = http
        .get(format!("{base}/auth/oauth/login"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "ended at our callback");
    let tokens: TokenPair = r.json().await.unwrap();
    let me: Value = http
        .get(format!("{base}/me"))
        .bearer_auth(&tokens.access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(me["username"], "octocat_oauth");

    // Logging in again finds the same linked account.
    let again: TokenPair = http
        .get(format!("{base}/auth/oauth/login"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let me2: Value = http
        .get(format!("{base}/me"))
        .bearer_auth(&again.access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(me2["id"], me["id"]);
}

#[tokio::test]
async fn ex6_redirect_carries_state_and_challenge() {
    let (base, provider, _) = spawn_with_provider(SECRET).await;
    let no_redirects = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let r = no_redirects
        .get(format!("{base}/auth/oauth/login"))
        .send()
        .await
        .unwrap();
    assert!(r.status().is_redirection());
    let location = reqwest::Url::parse(r.headers()["location"].to_str().unwrap()).unwrap();
    assert!(
        location
            .as_str()
            .starts_with(&format!("{provider}/authorize"))
    );
    let q: std::collections::HashMap<_, _> = location.query_pairs().into_owned().collect();
    assert_eq!(q["response_type"], "code");
    assert_eq!(q["code_challenge_method"], "S256");
    assert_eq!(
        q["code_challenge"].len(),
        43,
        "base64url(sha256) without padding"
    );
    assert!(q["state"].len() >= 16);
}

#[tokio::test]
async fn ex6_forged_or_replayed_callbacks_fail() {
    let (base, _provider, _) = spawn_with_provider(SECRET).await;
    let http = reqwest::Client::new();
    let r = http
        .get(format!("{base}/auth/oauth/callback?code=abc&state=made-up"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 401, "unknown state");
    let r = http
        .get(format!("{base}/auth/oauth/callback?error=access_denied"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 400);

    // Capture a real callback URL, use it once, then replay it.
    let manual = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let to_provider = manual
        .get(format!("{base}/auth/oauth/login"))
        .send()
        .await
        .unwrap()
        .headers()["location"]
        .to_str()
        .unwrap()
        .to_string();
    let callback = manual.get(&to_provider).send().await.unwrap().headers()["location"]
        .to_str()
        .unwrap()
        .to_string();
    assert_eq!(manual.get(&callback).send().await.unwrap().status(), 200);
    assert_eq!(
        manual.get(&callback).send().await.unwrap().status(),
        401,
        "state and code are single-use"
    );
}

#[tokio::test]
async fn ex6_client_credentials_and_pkce_checks() {
    use mock_provider::pkce_challenge;
    // RFC 7636 Appendix B test vector
    assert_eq!(
        pkce_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
    );
    let (_, provider, state) = spawn_with_provider(SECRET).await;
    let token = state
        .oauth
        .as_ref()
        .unwrap()
        .client_credentials_token()
        .await
        .unwrap();
    assert_eq!(token.len(), 48);
    // Exchanging a code without the right verifier fails at the provider:
    let r = reqwest::Client::new()
        .post(format!("{provider}/token"))
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", "rustlearn-app"),
            ("client_secret", "provider-secret"),
            ("code", "x"),
            ("code_verifier", "y"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 400);
}

// ---- Bonus: TOTP ----------------------------------------------------------------------------

#[test]
fn bonus_rfc_test_vectors() {
    use totp::*;
    let secret = b"12345678901234567890";
    // RFC 4226 Appendix D (HOTP)
    assert_eq!(hotp(secret, 0, 6), "755224");
    assert_eq!(hotp(secret, 9, 6), "520489");
    // RFC 6238 Appendix B (TOTP, SHA-1, 8 digits)
    assert_eq!(totp(secret, 59, 8), "94287082");
    assert_eq!(totp(secret, 1111111109, 8), "07081804");
    assert_eq!(totp(secret, 2000000000, 8), "69279037");
    assert_eq!(base32(b"foobar"), "MZXW6YTBOI", "RFC 4648 test vector");
}

#[test]
fn bonus_verify_window() {
    let secret = totp::generate_secret();
    let t = 1_700_000_000;
    let code = totp::totp(&secret, t, 6);
    assert!(totp::verify(&secret, &code, t));
    assert!(totp::verify(&secret, &code, t + 30), "one step late is OK");
    assert!(
        !totp::verify(&secret, &code, t + 90),
        "three steps late isn't"
    );
}

#[tokio::test]
async fn bonus_login_requires_totp_after_enrolment() {
    let base = app().await;
    let (_, bob) = two_users(&base).await;
    let enroll: Value = reqwest::Client::new()
        .post(format!("{base}/mfa/enroll"))
        .bearer_auth(&bob.access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        enroll["otpauth_url"]
            .as_str()
            .unwrap()
            .starts_with("otpauth://totp/rustlearn:bob?secret=")
    );
    let r = post(
        &base,
        "/login",
        json!({"username": "bob", "password": BOB_PW}),
    )
    .await;
    assert_eq!(r.json::<Value>().await.unwrap()["error"], "totp_required");
    let r = post(
        &base,
        "/login",
        json!({"username": "bob", "password": BOB_PW, "totp": "000000"}),
    )
    .await;
    assert_eq!(r.status(), 401);
    let wrong_pw = post(
        &base,
        "/login",
        json!({"username": "bob", "password": "wrong wrong wrong"}),
    )
    .await;
    assert_eq!(
        wrong_pw.json::<Value>().await.unwrap()["error"],
        "invalid_credentials",
        "password is checked first"
    );
}
