# Exercises: Authentication

One Axum app that implements the common ways to answer "who are you?" and
"what may you do?": passwords, JWTs, cookie sessions, API keys, OAuth2 login
and TOTP two-factor codes. Storage is in memory, so what you read is the
security logic, not database code (module 18 covers that).

**Setup**: `axum`, `axum-extra` (cookies), `argon2`, `jsonwebtoken`, `sha2`,
`sha1`, `hmac`, `base64`, `rand`, `reqwest` are in `exercise/Cargo.toml`.
`mock_provider.rs` (an OAuth2 provider) and `app::spawn_with_provider` (wires
it to your app) are provided.

> Security code is where "it works" isn't enough. For each exercise, the tests
> check the attack it defends against, not only the happy path.

---

## Exercise 1: Passwords, Registration, Login

**Difficulty**: Medium
**Time**: 60 minutes

1. `password.rs`: `hash_password` (Argon2id, PHC string), `verify_password`,
   `check_policy` (12–128 characters, not a common password — NIST
   SP 800-63B: length, not composition rules)
2. `users.rs`: `register` (usernames 3–32 of `[a-z0-9_]`, case-insensitive;
   first user is `Admin`), `authenticate`
3. `POST /register` → 201 / 400 / 409; `POST /login` → a token pair (Exercise 2)

**No user enumeration**: "unknown user" and "wrong password" must return the
*same* error — and take the same time. When the user doesn't exist, verify
against a dummy hash anyway.

**Hint**: argon2 0.6: `Argon2::default().hash_password(pw.as_bytes())?.to_string()`
generates the salt for you. Argon2 is slow on purpose — call it inside
`tokio::task::spawn_blocking` in async handlers.

---

## Exercise 2: JWT Access and Refresh Tokens

**Difficulty**: Hard
**Time**: 60 minutes

```rust
pub struct Claims { sub: u64, username: String, role: Role, typ: TokenType, jti: String, iat: i64, exp: i64 }
impl TokenService {
    pub fn issue(&self, user_id, username, role, typ, ttl_secs: i64) -> String;
    pub fn issue_pair(&self, ...) -> (String, String);      // access 15 min, refresh 7 days
    pub fn verify(&self, token: &str, expected: TokenType) -> Result<Claims, TokenError>;
    pub fn refresh(&self, refresh_token: &str) -> Result<(String, String), TokenError>;
    pub fn revoke(&self, jti: &str);
}
```

- HS256 only, **pinned** in `Validation::new(Algorithm::HS256)` (so a token
  with `"alg":"none"` is rejected), zero leeway
- a refresh token can't call the API, and an access token can't refresh
- **rotation**: each refresh token works once; reuse → `Revoked`
- an `AuthUser` extractor: `Authorization: Bearer <token>` or 401 with `WWW-Authenticate: Bearer`
- `POST /token/refresh`, `POST /logout` (revokes both tokens), `GET /me`

---

## Exercise 3: Role-Based Access Control

**Difficulty**: Medium
**Time**: 45 minutes

```rust
pub enum Role { User, Moderator, Admin }
pub enum Permission { ReadOwnProfile, ReadReports, ManageUsers }
impl Role { pub fn permissions(self) -> &'static [Permission]; pub fn can(self, p: Permission) -> bool; }
```

Make permissions part of handler signatures:

```rust
async fn list_users(_: Authorized<ManageUsers>) -> ...      // 401 if not logged in, 403 if not allowed
```

Routes: `GET /reports` (ReadReports), `GET /admin/users` and
`PUT /admin/users/{id}/role` (ManageUsers; an admin can't demote themselves).

**Question**: after Bob is promoted, his current token still says `user`. Why,
and what are the options?

---

## Exercise 4: Cookie Sessions and CSRF

**Difficulty**: Hard
**Time**: 60 minutes

1. `POST /session/login` (a form) creates a server-side session and sets
   `session=<random id>; HttpOnly; SameSite=Lax; Path=/` (+ `Secure` in
   production). Response: `{"csrf_token": "..."}`. Always a **new** session id
   on login.
2. `GET /session/me` reads the session from the cookie.
3. `POST /session/display-name` and `POST /session/logout` require the form
   field `csrf_token` to match the session's token — compared in constant time.
4. Sessions expire after 8 hours, checked server-side.

**Question**: the cookie is `SameSite=Lax`. Why still bother with CSRF tokens?

---

## Exercise 5: API Keys

**Difficulty**: Easy
**Time**: 30 minutes

- `POST /api-keys` (bearer) → `{"key": "rlk_<48 hex>"}`, shown once
- store only `sha256(key)`; look keys up by hash
- `GET /api/whoami` with `x-api-key`

**Question**: why is a fast hash fine here, when passwords need Argon2?

---

## Exercise 6: OAuth2 Login ("Sign in with ...")

**Difficulty**: Very Hard
**Time**: 90 minutes

The authorization code flow with PKCE, against `mock_provider.rs`:

```
browser -> GET /auth/oauth/login        -> 303 to provider /authorize?response_type=code&client_id
                                            &redirect_uri&state&code_challenge&code_challenge_method=S256
provider (user approves)               -> 303 to /auth/oauth/callback?code=..&state=..
our callback: check state, POST /token with code + code_verifier, GET /userinfo,
              find-or-create the linked user, return OUR token pair
```

```rust
impl OAuthClient {
    pub fn start_login(&self) -> String;                                  // the redirect URL
    pub async fn finish_login(&self, code: &str, state: &str) -> Result<UserInfo, OAuthError>;
    pub async fn client_credentials_token(&self) -> Result<String, OAuthError>;   // service-to-service
}
```

- `state` and `code_verifier`: random, remembered per login, **single-use**
- `code_challenge = base64url_no_pad(sha256(code_verifier))`
- an unknown or reused state → 401; `?error=access_denied` → 400

**Test vector** (RFC 7636): verifier `dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk`
→ challenge `E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM`.

---

## Bonus Challenge: TOTP Two-Factor Authentication

**Difficulty**: Hard
**Time**: 60 minutes

Implement RFC 4226 (HOTP) and RFC 6238 (TOTP) with HMAC-SHA1:

```rust
pub fn hotp(secret: &[u8], counter: u64, digits: u32) -> String;
pub fn totp(secret: &[u8], unix_time: u64, digits: u32) -> String;   // counter = time / 30
pub fn verify(secret: &[u8], code: &str, unix_time: u64) -> bool;    // ±1 step
pub fn base32(bytes: &[u8]) -> String;
pub fn otpauth_url(issuer, account, secret) -> String;
```

`POST /mfa/enroll` stores a secret and returns the `otpauth://` URL; from then
on `/login` needs `"totp"` — checked only after the password is right.

**Test vectors**: secret `12345678901234567890` → HOTP(0) = `755224`;
TOTP(t=59, 8 digits) = `94287082`. Scan your `otpauth://` URL as a QR code with
a real authenticator app and log in with it.

---

## Check Your Understanding

- [ ] Hash passwords with Argon2id, and avoid user enumeration
- [ ] Issue, verify, rotate and revoke JWTs; pin the algorithm
- [ ] Return 401 vs 403 correctly; check permissions, not role names
- [ ] Set session cookies with the right flags; defend against CSRF
- [ ] Store API keys hashed
- [ ] Explain every parameter of the OAuth2 code flow, and what PKCE adds
- [ ] Implement TOTP from the RFC

---

## Additional Resources

- [OWASP Authentication Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)
- [OWASP Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)
- [OWASP CSRF Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html)
- [RFC 6749 (OAuth 2.0)](https://www.rfc-editor.org/rfc/rfc6749), [RFC 7636 (PKCE)](https://www.rfc-editor.org/rfc/rfc7636), [RFC 6238 (TOTP)](https://www.rfc-editor.org/rfc/rfc6238)
- [oauth2 crate](https://docs.rs/oauth2/) — a production OAuth2 client
