# 19 · Authentication

## Overview

Authentication ("who are you?") and authorization ("what may you do?") are the
parts of a web service where a small mistake becomes a breach. This module
implements the standard mechanisms — password hashing, JWTs, cookie sessions
with CSRF protection, API keys, OAuth2 with PKCE, and TOTP — in one Axum app,
with tests that check the *attacks* each one stops.

## What You'll Learn

| Exercise | Topic | Defends against |
|---|---|---|
| 1 | Argon2id, login | stolen password databases; user enumeration (by message *and* timing) |
| 2 | JWT access + refresh | forged tokens, `alg: none`, stolen refresh tokens (rotation) |
| 3 | RBAC | privilege escalation; permissions in handler types |
| 4 | Sessions + CSRF | XSS cookie theft (HttpOnly), cross-site request forgery, session fixation |
| 5 | API keys | key leaks from the database (store hashes) |
| 6 | OAuth2 + PKCE | login CSRF (`state`), intercepted codes (PKCE), open redirects |
| Bonus | TOTP | phished or reused passwords |

## Key Concepts

### Which mechanism when?

| Client | Use |
|---|---|
| Server-rendered website | **cookie session** + CSRF tokens |
| SPA / mobile app calling your API | **short-lived JWT access token + refresh token** (or a session cookie if same-site) |
| Script, CI job, partner integration | **API key** (or OAuth2 client credentials) |
| "Sign in with GitHub/Google" | **OAuth2 code flow + PKCE** (OpenID Connect on top) |
| Anything sensitive | add **MFA** |

### JWT vs session

| | JWT | Server-side session |
|---|---|---|
| Server state | none — claims are signed | a session store |
| Revocation | hard: wait for expiry, or keep a deny-list | instant: delete the session |
| Role change takes effect | at next token refresh | immediately |
| Scales across servers | trivially | needs a shared store (Redis) |

Hence short access tokens (15 min) plus revocable, rotated refresh tokens.

### 401 vs 403

401 Unauthorized means *unauthenticated*: no credentials, or bad ones — send
`WWW-Authenticate`. 403 Forbidden means *authenticated but not allowed*.
Getting them backwards confuses every client and log reader.

### Things that look fine and aren't

- `if user.is_none() { return "no such user" }` — user enumeration
- returning early for unknown users — enumeration by **timing**
- `Validation::default()` without pinning the algorithm, on older libraries — `alg: none` / key confusion
- `token == expected` — timing leak; compare in constant time
- a CSRF token in a cookie only — the browser sends it automatically, so it proves nothing
- `redirect_uri` matched with `starts_with` — the code leaks to an attacker's URL

## Common Pitfalls

1. **Fast hashes for passwords** (SHA-256, MD5) — billions of guesses per second on a GPU
2. **Secrets in JWT claims** — claims are only base64, anyone can read them
3. **Long-lived access tokens** — can't be revoked
4. **Missing `HttpOnly`/`SameSite`/`Secure`** on session cookies
5. **Reusing the session id across login** — session fixation
6. **OAuth without `state` or PKCE**
7. **Rolling your own** in production — use these exercises to understand the pieces, then prefer audited libraries (`oauth2`, `tower-sessions`, `axum-login`)

## Running This Module

```bash
cargo run  -p m19-authentication -- demo                     # your code
cargo test -p m19-authentication-tests --features mine       # test your code
cargo run  -p m19-authentication-solution -- demo            # every flow end to end
cargo test -p m19-authentication-tests                       # 22 tests against the solution
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the exercise list in
[the phase README](../README.md) (registration/login, JWT, OAuth2, roles).

- **OAuth2 runs against a mock provider** (`mock_provider.rs`), started
  in-process on a random port, instead of Google or GitHub — so it works
  offline and in tests. Switching to a real provider means changing the three
  URLs and the client credentials in `OAuthConfig`.
- **The client is hand-written** to show every step. In production use the
  [`oauth2`](https://docs.rs/oauth2) crate (and OpenID Connect for identity).
- **`Secure` cookies are off by default** because the tests use plain HTTP;
  `AppState::secure_cookies` turns them on. Always on in production.
- **Argon2 is optimised in debug builds** via `[profile.dev.package.argon2]`
  in the root `Cargo.toml`, otherwise the tests take ten times longer.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[20 · WebSockets](../20-websockets/)
