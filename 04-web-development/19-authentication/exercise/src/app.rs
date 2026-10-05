//! The routes, the shared state, and the extractors that enforce auth.
//!
//! Bearer-token API (Exercises 1-3, 5, bonus):
//!   POST /register            {username, password}             -> 201
//!   POST /login               {username, password, totp?}      -> {access_token, refresh_token, ...}
//!   POST /token/refresh       {refresh_token}                  -> a new pair (rotation)
//!   POST /logout              bearer + {refresh_token}         -> 204
//!   GET  /me                  bearer
//!   GET  /reports             bearer, permission ReadReports
//!   GET  /admin/users         bearer, permission ManageUsers
//!   PUT  /admin/users/{id}/role
//!   POST /api-keys            bearer + {label}                 -> {key} (shown once)
//!   GET  /api/whoami          x-api-key
//!   POST /mfa/enroll          bearer                           -> {secret, otpauth_url}
//! Cookie sessions (Exercise 4):
//!   POST /session/login       form                             -> Set-Cookie + {csrf_token}
//!   GET  /session/me          cookie
//!   POST /session/display-name  form + csrf_token
//!   POST /session/logout      form + csrf_token
//! OAuth2 (Exercise 6):
//!   GET  /auth/oauth/login    -> 303 to the provider
//!   GET  /auth/oauth/callback ?code&state -> our tokens

use crate::api_keys::ApiKeyStore;
use crate::oauth::OAuthClient;
use crate::rbac::{Permission, Role};
use crate::sessions::{COOKIE_NAME, SessionStore, tokens_match};
use crate::tokens::{ACCESS_TTL_SECS, Claims, TokenError, TokenService, TokenType};
use crate::totp;
use crate::users::{UserError, UserStore};
use axum::extract::{Form, FromRequestParts, Path, Query, State};
use axum::http::request::Parts;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub users: UserStore,
    pub tokens: TokenService,
    pub sessions: SessionStore,
    pub api_keys: ApiKeyStore,
    pub oauth: Option<Arc<OAuthClient>>,
    pub display_names: Arc<Mutex<HashMap<u64, String>>>,
    /// `Secure` cookies are only sent over HTTPS -- off for local HTTP tests.
    pub secure_cookies: bool,
}

impl AppState {
    pub fn new(jwt_secret: &[u8]) -> Self {
        todo!("Exercises 1-6")
    }

    pub fn with_oauth(mut self, client: OAuthClient) -> Self {
        todo!("Exercises 1-6")
    }
}

// ---- Errors ----------------------------------------------------------------------

#[derive(Debug)]
pub enum AuthError {
    /// 401: who are you? (missing, invalid or expired credentials)
    Unauthorized(&'static str),
    /// 403: I know who you are, and you may not.
    Forbidden(&'static str),
    BadRequest(String),
    Conflict(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        todo!("Exercises 1-6")
    }
}

impl From<TokenError> for AuthError {
    fn from(e: TokenError) -> Self {
        todo!("Exercises 1-6")
    }
}

// ---- Extractors --------------------------------------------------------------------

/// Exercise 2: any handler with an `AuthUser` argument requires a valid
/// access token. The check lives in one place and can't be forgotten.
pub struct AuthUser(pub Claims);

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        todo!("Exercises 1-6")
    }
}

/// Exercise 3: permissions as *types*. `Authorized<ManageUsers>` in a
/// handler's signature means the handler can't run without that
/// permission -- the requirement is visible in the code that needs it.
pub trait RequiredPermission {
    const PERMISSION: Permission;
}

pub struct ReadReports;
impl RequiredPermission for ReadReports {
    const PERMISSION: Permission = Permission::ReadReports;
}

pub struct ManageUsers;
impl RequiredPermission for ManageUsers {
    const PERMISSION: Permission = Permission::ManageUsers;
}

pub struct Authorized<P: RequiredPermission>(pub Claims, PhantomData<P>);

impl<P: RequiredPermission + Send> FromRequestParts<AppState> for Authorized<P> {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        todo!("Exercises 1-6")
    }
}

// ---- Exercise 1: register / login --------------------------------------------------

#[derive(Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub totp: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

fn pair(access: String, refresh: String) -> TokenPair {
    todo!("Exercises 1-6")
}

async fn register(
    State(s): State<AppState>,
    Json(c): Json<Credentials>,
) -> Result<Response, AuthError> {
    todo!("Exercises 1-6")
}

async fn login(
    State(s): State<AppState>,
    Json(c): Json<Credentials>,
) -> Result<Json<TokenPair>, AuthError> {
    todo!("Exercises 1-6")
}

// ---- Exercise 2: refresh and logout --------------------------------------------------

#[derive(Deserialize)]
struct RefreshBody {
    refresh_token: String,
}

async fn refresh(
    State(s): State<AppState>,
    Json(b): Json<RefreshBody>,
) -> Result<Json<TokenPair>, AuthError> {
    todo!("Exercises 1-6")
}

/// Revokes the access token in hand and the refresh token in the body.
async fn logout(
    State(s): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(b): Json<RefreshBody>,
) -> StatusCode {
    todo!("Exercises 1-6")
}

async fn me(AuthUser(claims): AuthUser) -> Json<serde_json::Value> {
    todo!("Exercises 1-6")
}

// ---- Exercise 3: RBAC -------------------------------------------------------------------

async fn reports(Authorized(claims, _): Authorized<ReadReports>) -> Json<serde_json::Value> {
    todo!("Exercises 1-6")
}

async fn list_users(
    State(s): State<AppState>,
    _: Authorized<ManageUsers>,
) -> Json<serde_json::Value> {
    todo!("Exercises 1-6")
}

#[derive(Deserialize)]
struct RoleBody {
    role: Role,
}

/// Note: the user's *existing* access tokens still carry the old role until
/// they expire (at most 15 minutes) -- the price of stateless tokens.
async fn set_role(
    State(s): State<AppState>,
    Authorized(admin, _): Authorized<ManageUsers>,
    Path(id): Path<u64>,
    Json(b): Json<RoleBody>,
) -> Result<Json<serde_json::Value>, AuthError> {
    todo!("Exercises 1-6")
}

// ---- Exercise 5: API keys -----------------------------------------------------------------

#[derive(Deserialize)]
struct KeyBody {
    label: String,
}

async fn create_api_key(
    State(s): State<AppState>,
    AuthUser(c): AuthUser,
    Json(b): Json<KeyBody>,
) -> Response {
    todo!("Exercises 1-6")
}

async fn api_whoami(
    State(s): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, AuthError> {
    todo!("Exercises 1-6")
}

// ---- Bonus: MFA enrolment -------------------------------------------------------------------

async fn mfa_enroll(State(s): State<AppState>, AuthUser(c): AuthUser) -> Json<serde_json::Value> {
    todo!("Exercises 1-6")
}

// ---- Exercise 4: cookie sessions and CSRF -------------------------------------------------

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

fn session_cookie(id: String, secure: bool) -> Cookie<'static> {
    todo!("Exercises 1-6")
}

/// Rotate on login: always a brand-new session id, never an existing one
/// (prevents "session fixation").
async fn session_login(
    State(s): State<AppState>,
    jar: CookieJar,
    Form(f): Form<LoginForm>,
) -> Result<(CookieJar, Json<serde_json::Value>), AuthError> {
    todo!("Exercises 1-6")
}

/// The session extractor for cookie routes.
pub struct SessionUser {
    pub session_id: String,
    pub user_id: u64,
    pub csrf_token: String,
}

impl FromRequestParts<AppState> for SessionUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        todo!("Exercises 1-6")
    }
}

async fn session_me(
    State(s): State<AppState>,
    u: SessionUser,
) -> Result<Json<serde_json::Value>, AuthError> {
    todo!("Exercises 1-6")
}

#[derive(Deserialize)]
struct DisplayNameForm {
    display_name: String,
    csrf_token: Option<String>,
}

fn check_csrf(session: &SessionUser, submitted: Option<&str>) -> Result<(), AuthError> {
    todo!("Exercises 1-6")
}

async fn session_set_display_name(
    State(s): State<AppState>,
    u: SessionUser,
    Form(f): Form<DisplayNameForm>,
) -> Result<Json<serde_json::Value>, AuthError> {
    todo!("Exercises 1-6")
}

#[derive(Deserialize)]
struct CsrfForm {
    csrf_token: Option<String>,
}

async fn session_logout(
    State(s): State<AppState>,
    u: SessionUser,
    jar: CookieJar,
    Form(f): Form<CsrfForm>,
) -> Result<(CookieJar, StatusCode), AuthError> {
    todo!("Exercises 1-6")
}

// ---- Exercise 6: OAuth2 login ---------------------------------------------------------------

async fn oauth_login(State(s): State<AppState>) -> Result<Redirect, AuthError> {
    todo!("Exercises 1-6")
}

#[derive(Deserialize)]
struct CallbackParams {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

async fn oauth_callback(
    State(s): State<AppState>,
    Query(p): Query<CallbackParams>,
) -> Result<Json<TokenPair>, AuthError> {
    todo!("Exercises 1-6")
}

pub fn router(state: AppState) -> Router {
    todo!("Exercises 1-6")
}

/// (Provided -- not an exercise.) Starts the mock provider and the app wired to each other; returns
/// (app base URL, provider base URL, state). Used by the demo and the tests.
pub async fn spawn_with_provider(jwt_secret: &[u8]) -> (String, String, AppState) {
    use crate::mock_provider::{ProviderConfig, router as provider_router};
    let config = ProviderConfig {
        client_id: "rustlearn-app".into(),
        client_secret: "provider-secret".into(),
        redirect_uri: String::new(), // set once the app's port is known
        user_sub: "gh-4242".into(),
        user_login: "octocat".into(),
    };
    let (provider, handle) = provider_router(config);
    let provider_url = crate::spawn(provider, "127.0.0.1:0").await;

    // Bind the app first so the callback URL (with its port) is known.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let app_url = format!("http://{}", listener.local_addr().unwrap());
    let redirect_uri = format!("{app_url}/auth/oauth/callback");
    handle.set_redirect_uri(&redirect_uri);

    let state = AppState::new(jwt_secret).with_oauth(OAuthClient::new(crate::oauth::OAuthConfig {
        authorize_url: format!("{provider_url}/authorize"),
        token_url: format!("{provider_url}/token"),
        userinfo_url: format!("{provider_url}/userinfo"),
        client_id: "rustlearn-app".into(),
        client_secret: "provider-secret".into(),
        redirect_uri,
    }));
    let app = router(state.clone());
    tokio::spawn(async move { axum::serve(listener, app).await.expect("app server") });
    (app_url, provider_url, state)
}
