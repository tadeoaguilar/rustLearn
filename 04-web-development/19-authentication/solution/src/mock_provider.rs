//! A minimal OAuth2 provider (think "GitHub login"), run in-process so
//! Exercise 6 works offline and in tests. It auto-approves a single
//! configured user instead of showing a login page.
//!
//! This is infrastructure, not an exercise -- but reading it shows exactly
//! what the *provider* checks, which is what the client must get right.
//!
//! GET  /authorize   ?response_type=code&client_id&redirect_uri&state&code_challenge&code_challenge_method=S256
//! POST /token       grant_type=authorization_code | client_credentials
//! GET  /userinfo    Authorization: Bearer <access token>

use axum::extract::{Form, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ProviderConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub user_sub: String,
    pub user_login: String,
}

#[derive(Clone)]
struct ProviderState {
    config: Arc<Mutex<ProviderConfig>>,
    codes: Arc<Mutex<HashMap<String, (String, String)>>>, // code -> (challenge, redirect_uri)
    tokens: Arc<Mutex<HashMap<String, String>>>,          // access token -> subject
}

/// Lets the app tell the provider its callback URL once both are running.
#[derive(Clone)]
pub struct ProviderHandle {
    config: Arc<Mutex<ProviderConfig>>,
}

impl ProviderHandle {
    pub fn set_redirect_uri(&self, uri: &str) {
        self.config.lock().unwrap().redirect_uri = uri.to_string();
    }
}

pub fn pkce_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

#[derive(Deserialize)]
struct AuthorizeParams {
    response_type: String,
    client_id: String,
    redirect_uri: String,
    state: String,
    code_challenge: String,
    code_challenge_method: String,
}

fn oauth_error(status: StatusCode, error: &str) -> Response {
    (status, Json(json!({ "error": error }))).into_response()
}

async fn authorize(State(s): State<ProviderState>, Query(p): Query<AuthorizeParams>) -> Response {
    let config = s.config.lock().unwrap().clone();
    // The redirect URI must match the registered one EXACTLY: otherwise an
    // attacker could have the code sent to their own server.
    if p.client_id != config.client_id || p.redirect_uri != config.redirect_uri {
        return oauth_error(StatusCode::BAD_REQUEST, "invalid_client_or_redirect_uri");
    }
    if p.response_type != "code" || p.code_challenge_method != "S256" {
        return oauth_error(StatusCode::BAD_REQUEST, "invalid_request");
    }
    let code = crate::random_hex(16);
    s.codes
        .lock()
        .unwrap()
        .insert(code.clone(), (p.code_challenge, p.redirect_uri.clone()));
    let url = reqwest::Url::parse_with_params(
        &p.redirect_uri,
        &[("code", code.as_str()), ("state", p.state.as_str())],
    )
    .expect("registered redirect uri is a valid URL");
    Redirect::to(url.as_str()).into_response()
}

#[derive(Deserialize)]
struct TokenForm {
    grant_type: String,
    client_id: String,
    client_secret: String,
    code: Option<String>,
    redirect_uri: Option<String>,
    code_verifier: Option<String>,
}

async fn token(State(s): State<ProviderState>, Form(f): Form<TokenForm>) -> Response {
    let config = s.config.lock().unwrap().clone();
    if f.client_id != config.client_id || f.client_secret != config.client_secret {
        return oauth_error(StatusCode::UNAUTHORIZED, "invalid_client");
    }
    let subject = match f.grant_type.as_str() {
        "authorization_code" => {
            // Codes are single-use: remove() it whether or not the rest checks out.
            let Some((challenge, redirect_uri)) = f
                .code
                .as_ref()
                .and_then(|c| s.codes.lock().unwrap().remove(c))
            else {
                return oauth_error(StatusCode::BAD_REQUEST, "invalid_grant");
            };
            if f.redirect_uri.as_deref() != Some(redirect_uri.as_str()) {
                return oauth_error(StatusCode::BAD_REQUEST, "invalid_grant");
            }
            // PKCE: only whoever generated the challenge knows the verifier.
            if f.code_verifier.as_deref().map(pkce_challenge) != Some(challenge) {
                return oauth_error(StatusCode::BAD_REQUEST, "invalid_grant");
            }
            config.user_sub.clone()
        }
        "client_credentials" => format!("service:{}", config.client_id),
        _ => return oauth_error(StatusCode::BAD_REQUEST, "unsupported_grant_type"),
    };
    let access_token = crate::random_hex(24);
    s.tokens
        .lock()
        .unwrap()
        .insert(access_token.clone(), subject);
    Json(json!({ "access_token": access_token, "token_type": "Bearer", "expires_in": 3600 }))
        .into_response()
}

async fn userinfo(State(s): State<ProviderState>, headers: HeaderMap) -> Response {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    let subject = token.and_then(|t| s.tokens.lock().unwrap().get(t).cloned());
    match subject {
        Some(sub) => {
            let login = s.config.lock().unwrap().user_login.clone();
            Json(json!({ "sub": sub, "preferred_username": login, "email": format!("{login}@example.com") })).into_response()
        }
        None => oauth_error(StatusCode::UNAUTHORIZED, "invalid_token"),
    }
}

pub fn router(config: ProviderConfig) -> (Router, ProviderHandle) {
    let state = ProviderState {
        config: Arc::new(Mutex::new(config)),
        codes: Arc::default(),
        tokens: Arc::default(),
    };
    let handle = ProviderHandle {
        config: state.config.clone(),
    };
    let router = Router::new()
        .route("/authorize", get(authorize))
        .route("/token", post(token))
        .route("/userinfo", get(userinfo))
        .with_state(state);
    (router, handle)
}
