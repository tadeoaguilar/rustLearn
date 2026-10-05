//! Exercise 6: logging in with an OAuth2 provider ("Sign in with GitHub").
//!
//! Authorization code flow with PKCE:
//!
//! 1. Our app redirects the browser to the provider's /authorize, with a
//!    random `state` and a `code_challenge` = base64url(sha256(verifier)).
//! 2. The user logs in *at the provider*; we never see their password.
//! 3. The provider redirects back to our callback with `?code=..&state=..`.
//! 4. We check `state` matches one we issued (CSRF protection for the login
//!    itself), then POST the code + the secret `code_verifier` to /token.
//!    A stolen code is useless without the verifier (that's PKCE).
//! 5. With the provider's access token we fetch the user's identity, then
//!    log them in to *our* app (find or create the linked account).

use base64::Engine;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub authorize_url: String,
    pub token_url: String,
    pub userinfo_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UserInfo {
    pub sub: String,
    pub preferred_username: String,
    pub email: String,
}

#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    #[error("unknown or reused state")]
    InvalidState,
    #[error("provider rejected the request: {0}")]
    Provider(String),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

pub struct OAuthClient {
    pub config: OAuthConfig,
    http: reqwest::Client,
    pending: Mutex<HashMap<String, String>>, // state -> code_verifier
}

impl OAuthClient {
    pub fn new(config: OAuthConfig) -> Self {
        todo!("Exercise 6")
    }

    /// Step 1. Returns the URL to redirect the browser to.
    pub fn start_login(&self) -> String {
        todo!("Exercise 6")
    }

    /// Steps 4-5. Each state works once.
    pub async fn finish_login(&self, code: &str, state: &str) -> Result<UserInfo, OAuthError> {
        todo!("Exercise 6")
    }

    /// The *client credentials* grant: no user at all, a service
    /// authenticating as itself (backend-to-backend calls).
    pub async fn client_credentials_token(&self) -> Result<String, OAuthError> {
        todo!("Exercise 6")
    }
}
