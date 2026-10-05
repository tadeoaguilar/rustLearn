# Getting Started with 19 · Authentication

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m19-authentication-solution -- demo
```

## What Is Already Here

```
19-authentication/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/src/             # ← YOUR WORKSPACE (package m19-authentication)
│   ├── password.rs, users.rs     # Ex 1
│   ├── tokens.rs                 # Ex 2
│   ├── rbac.rs                   # Ex 3
│   ├── sessions.rs               # Ex 4
│   ├── api_keys.rs               # Ex 5
│   ├── oauth.rs                  # Ex 6
│   ├── totp.rs                   # bonus
│   ├── app.rs                    # routes, extractors (spawn_with_provider is provided)
│   ├── mock_provider.rs          # PROVIDED: an OAuth2 provider to log in with
│   └── main.rs                   # runner: demo / serve
├── solution/                 # ← REFERENCE (m19-authentication-solution) + ANSWERS.md
└── tests/                    # ← 22 tests (m19-authentication-tests)
```

## The Commands You Need

```bash
cargo run  -p m19-authentication -- demo
cargo test -p m19-authentication-tests --features mine
cargo test -p m19-authentication-tests --features mine ex2_
cargo run  -p m19-authentication-solution -- serve       # the app on :3000
cargo test -p m19-authentication-tests                   # always green
```

## Try It With curl

```bash
curl -s -X POST localhost:3000/register -H 'content-type: application/json' -d '{"username":"alice","password":"correct horse battery"}'
TOKEN=$(curl -s -X POST localhost:3000/login -H 'content-type: application/json' \
  -d '{"username":"alice","password":"correct horse battery"}' | jq -r .access_token)
curl -s localhost:3000/me -H "authorization: Bearer $TOKEN"
curl -si localhost:3000/me | head -5                            # 401 + WWW-Authenticate

# Decode the token's claims (they're not secret -- just signed):
echo "$TOKEN" | cut -d. -f2 | tr '_-' '/+' | base64 -d 2>/dev/null; echo

# Cookie session, with curl's cookie jar
curl -s -c /tmp/jar -X POST localhost:3000/session/login -d 'username=alice&password=correct horse battery'
curl -s -b /tmp/jar localhost:3000/session/me
curl -si -b /tmp/jar -X POST localhost:3000/session/display-name -d 'display_name=x' | head -1    # 403: no CSRF token
```

## If You Get Stuck

1. **Every login fails** — check you hash *and* verify the same bytes (`password.as_bytes()`), and that `verify_password` gets the full PHC string.
2. **`InvalidAlgorithm` from jsonwebtoken** — the `Header` algorithm and the key type must match (`Header::new(Algorithm::HS256)` with `EncodingKey::from_secret`).
3. **Cookies never come back in tests** — the client needs a cookie store (`reqwest` feature `cookies`, `.cookie_store(true)`), and `Secure` cookies aren't sent over plain HTTP.
4. **OAuth callback says `invalid_state`** — the state must be stored at `start_login` and removed (single use) at `finish_login`.
5. **Your extractor isn't found** — implement `FromRequestParts<AppState>` (not `FromRequest`): it doesn't consume the body, so it can be combined with `Json<..>`.
6. Compare against `solution/src/` — same file and function names.
