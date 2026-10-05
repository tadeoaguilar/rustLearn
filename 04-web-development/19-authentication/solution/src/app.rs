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
        AppState {
            users: UserStore::new(),
            tokens: TokenService::new(jwt_secret),
            sessions: SessionStore::new(),
            api_keys: ApiKeyStore::new(),
            oauth: None,
            display_names: Arc::default(),
            secure_cookies: false,
        }
    }

    pub fn with_oauth(mut self, client: OAuthClient) -> Self {
        self.oauth = Some(Arc::new(client));
        self
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
        let (status, error) = match self {
            AuthError::Unauthorized(e) => (StatusCode::UNAUTHORIZED, e.to_string()),
            AuthError::Forbidden(e) => (StatusCode::FORBIDDEN, e.to_string()),
            AuthError::BadRequest(e) => (StatusCode::BAD_REQUEST, e),
            AuthError::Conflict(e) => (StatusCode::CONFLICT, e),
        };
        let mut response = (status, Json(json!({ "error": error }))).into_response();
        if status == StatusCode::UNAUTHORIZED {
            // RFC 6750: tell the client which scheme to use.
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        response
    }
}

impl From<TokenError> for AuthError {
    fn from(e: TokenError) -> Self {
        AuthError::Unauthorized(match e {
            TokenError::Expired => "token_expired",
            TokenError::Revoked => "token_revoked",
            TokenError::WrongType => "wrong_token_type",
            TokenError::Invalid => "invalid_token",
        })
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
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AuthError::Unauthorized("missing_bearer_token"))?;
        Ok(AuthUser(state.tokens.verify(token, TokenType::Access)?))
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
        let AuthUser(claims) = AuthUser::from_request_parts(parts, state).await?;
        if !claims.role.can(P::PERMISSION) {
            return Err(AuthError::Forbidden("insufficient_permissions"));
        }
        Ok(Authorized(claims, PhantomData))
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
    TokenPair {
        access_token: access,
        refresh_token: refresh,
        token_type: "Bearer".into(),
        expires_in: ACCESS_TTL_SECS,
    }
}

async fn register(
    State(s): State<AppState>,
    Json(c): Json<Credentials>,
) -> Result<Response, AuthError> {
    // Argon2 is CPU-heavy (tens of ms): run it off the async worker threads.
    let users = s.users.clone();
    let user = tokio::task::spawn_blocking(move || users.register(&c.username, &c.password))
        .await
        .expect("register task")
        .map_err(|e| match e {
            UserError::Taken => AuthError::Conflict(e.to_string()),
            other => AuthError::BadRequest(other.to_string()),
        })?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "id": user.id, "username": user.username, "role": user.role })),
    )
        .into_response())
}

async fn login(
    State(s): State<AppState>,
    Json(c): Json<Credentials>,
) -> Result<Json<TokenPair>, AuthError> {
    let users = s.users.clone();
    let (username, password) = (c.username.clone(), c.password.clone());
    let user = tokio::task::spawn_blocking(move || users.authenticate(&username, &password))
        .await
        .expect("login task")
        .map_err(|_| AuthError::Unauthorized("invalid_credentials"))?;

    // Bonus: second factor. Only asked for *after* the password is right.
    if let Some(secret) = &user.totp_secret {
        let code = c.totp.ok_or(AuthError::Unauthorized("totp_required"))?;
        if !totp::verify(secret, &code, crate::tokens::now() as u64) {
            return Err(AuthError::Unauthorized("invalid_totp"));
        }
    }
    let (access, refresh) = s.tokens.issue_pair(user.id, &user.username, user.role);
    Ok(Json(pair(access, refresh)))
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
    let (access, refresh) = s.tokens.refresh(&b.refresh_token)?;
    Ok(Json(pair(access, refresh)))
}

/// Revokes the access token in hand and the refresh token in the body.
async fn logout(
    State(s): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(b): Json<RefreshBody>,
) -> StatusCode {
    s.tokens.revoke(&claims.jti);
    if let Ok(r) = s.tokens.verify(&b.refresh_token, TokenType::Refresh)
        && r.sub == claims.sub
    {
        s.tokens.revoke(&r.jti);
    }
    StatusCode::NO_CONTENT
}

async fn me(AuthUser(claims): AuthUser) -> Json<serde_json::Value> {
    Json(json!({ "id": claims.sub, "username": claims.username, "role": claims.role }))
}

// ---- Exercise 3: RBAC -------------------------------------------------------------------

async fn reports(Authorized(claims, _): Authorized<ReadReports>) -> Json<serde_json::Value> {
    Json(json!({ "reports": ["q3-revenue", "signups"], "viewer": claims.username }))
}

async fn list_users(
    State(s): State<AppState>,
    _: Authorized<ManageUsers>,
) -> Json<serde_json::Value> {
    let users: Vec<_> = s
        .users
        .all()
        .into_iter()
        .map(|u| json!({ "id": u.id, "username": u.username, "role": u.role }))
        .collect();
    Json(json!(users))
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
    if id == admin.sub && b.role != Role::Admin {
        return Err(AuthError::BadRequest(
            "admins can't demote themselves".into(),
        ));
    }
    let user = s
        .users
        .set_role(id, b.role)
        .ok_or(AuthError::BadRequest(format!("no user {id}")))?;
    Ok(Json(
        json!({ "id": user.id, "username": user.username, "role": user.role }),
    ))
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
    let key = s.api_keys.create(c.sub, &b.label);
    (StatusCode::CREATED, Json(json!({ "key": key, "label": b.label, "note": "store this now; it won't be shown again" })))
        .into_response()
}

async fn api_whoami(
    State(s): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, AuthError> {
    let key = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .ok_or(AuthError::Unauthorized("missing_api_key"))?;
    let record = s
        .api_keys
        .verify(key)
        .ok_or(AuthError::Unauthorized("invalid_api_key"))?;
    let user = s
        .users
        .find_by_id(record.user_id)
        .ok_or(AuthError::Unauthorized("invalid_api_key"))?;
    Ok(Json(
        json!({ "user_id": user.id, "username": user.username, "key_label": record.label }),
    ))
}

// ---- Bonus: MFA enrolment -------------------------------------------------------------------

async fn mfa_enroll(State(s): State<AppState>, AuthUser(c): AuthUser) -> Json<serde_json::Value> {
    let secret = totp::generate_secret();
    s.users.set_totp_secret(c.sub, secret.clone());
    // (A real flow confirms with one valid code before switching MFA on.)
    Json(
        json!({ "secret": totp::base32(&secret), "otpauth_url": totp::otpauth_url("rustlearn", &c.username, &secret) }),
    )
}

// ---- Exercise 4: cookie sessions and CSRF -------------------------------------------------

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

fn session_cookie(id: String, secure: bool) -> Cookie<'static> {
    Cookie::build((COOKIE_NAME, id))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .secure(secure)
        .build()
}

/// Rotate on login: always a brand-new session id, never an existing one
/// (prevents "session fixation").
async fn session_login(
    State(s): State<AppState>,
    jar: CookieJar,
    Form(f): Form<LoginForm>,
) -> Result<(CookieJar, Json<serde_json::Value>), AuthError> {
    let users = s.users.clone();
    let user = tokio::task::spawn_blocking(move || users.authenticate(&f.username, &f.password))
        .await
        .expect("login task")
        .map_err(|_| AuthError::Unauthorized("invalid_credentials"))?;
    if let Some(old) = jar.get(COOKIE_NAME) {
        s.sessions.destroy(old.value());
    }
    let (id, csrf) = s.sessions.create(user.id);
    Ok((
        jar.add(session_cookie(id, s.secure_cookies)),
        Json(json!({ "csrf_token": csrf })),
    ))
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
        let jar = CookieJar::from_headers(&parts.headers);
        let id = jar
            .get(COOKIE_NAME)
            .map(|c| c.value().to_string())
            .ok_or(AuthError::Unauthorized("no_session"))?;
        let session = state
            .sessions
            .get(&id)
            .ok_or(AuthError::Unauthorized("session_expired"))?;
        Ok(SessionUser {
            session_id: id,
            user_id: session.user_id,
            csrf_token: session.csrf_token,
        })
    }
}

async fn session_me(
    State(s): State<AppState>,
    u: SessionUser,
) -> Result<Json<serde_json::Value>, AuthError> {
    let user = s
        .users
        .find_by_id(u.user_id)
        .ok_or(AuthError::Unauthorized("no_session"))?;
    let display = s.display_names.lock().unwrap().get(&u.user_id).cloned();
    Ok(Json(
        json!({ "username": user.username, "display_name": display }),
    ))
}

#[derive(Deserialize)]
struct DisplayNameForm {
    display_name: String,
    csrf_token: Option<String>,
}

fn check_csrf(session: &SessionUser, submitted: Option<&str>) -> Result<(), AuthError> {
    match submitted {
        Some(t) if tokens_match(t, &session.csrf_token) => Ok(()),
        _ => Err(AuthError::Forbidden("csrf_token_invalid")),
    }
}

async fn session_set_display_name(
    State(s): State<AppState>,
    u: SessionUser,
    Form(f): Form<DisplayNameForm>,
) -> Result<Json<serde_json::Value>, AuthError> {
    check_csrf(&u, f.csrf_token.as_deref())?;
    s.display_names
        .lock()
        .unwrap()
        .insert(u.user_id, f.display_name.clone());
    Ok(Json(json!({ "display_name": f.display_name })))
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
    check_csrf(&u, f.csrf_token.as_deref())?;
    s.sessions.destroy(&u.session_id);
    Ok((
        jar.remove(Cookie::build(COOKIE_NAME).path("/")),
        StatusCode::NO_CONTENT,
    ))
}

// ---- Exercise 6: OAuth2 login ---------------------------------------------------------------

async fn oauth_login(State(s): State<AppState>) -> Result<Redirect, AuthError> {
    let client = s
        .oauth
        .as_ref()
        .ok_or(AuthError::BadRequest("oauth not configured".into()))?;
    Ok(Redirect::to(&client.start_login()))
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
    let client = s
        .oauth
        .as_ref()
        .ok_or(AuthError::BadRequest("oauth not configured".into()))?;
    if let Some(e) = p.error {
        return Err(AuthError::BadRequest(format!("provider error: {e}")));
    }
    let (Some(code), Some(state)) = (p.code, p.state) else {
        return Err(AuthError::BadRequest("missing code or state".into()));
    };
    let info = client
        .finish_login(&code, &state)
        .await
        .map_err(|e| match e {
            crate::oauth::OAuthError::InvalidState => AuthError::Unauthorized("invalid_state"),
            other => AuthError::BadRequest(other.to_string()),
        })?;
    let user = s
        .users
        .find_or_create_oauth(&info.sub, &info.preferred_username);
    let (access, refresh) = s.tokens.issue_pair(user.id, &user.username, user.role);
    Ok(Json(pair(access, refresh)))
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/token/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .route("/reports", get(reports))
        .route("/admin/users", get(list_users))
        .route("/admin/users/{id}/role", put(set_role))
        .route("/api-keys", post(create_api_key))
        .route("/api/whoami", get(api_whoami))
        .route("/mfa/enroll", post(mfa_enroll))
        .route("/session/login", post(session_login))
        .route("/session/me", get(session_me))
        .route("/session/display-name", post(session_set_display_name))
        .route("/session/logout", post(session_logout))
        .route("/auth/oauth/login", get(oauth_login))
        .route("/auth/oauth/callback", get(oauth_callback))
        .with_state(state)
}

/// Starts the mock provider and the app wired to each other; returns
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
