//! Module 19 -- Authentication. Reference solution.
//!
//! Authentication answers "who are you?"; authorization answers "what may
//! you do?". This crate implements the common answers to both, in one Axum
//! app, with in-memory storage so the security logic is all you read:
//!
//! | File               | Exercise |
//! |--------------------|----------|
//! | `password.rs`      | 1  argon2 hashing, password policy |
//! | `users.rs`         | 1  registration and login without user enumeration |
//! | `tokens.rs`        | 2  JWT access + refresh tokens, rotation, revocation |
//! | `rbac.rs`          | 3  roles, permissions, extractors that enforce them |
//! | `sessions.rs`      | 4  cookie sessions and CSRF protection |
//! | `api_keys.rs`      | 5  API keys stored as hashes |
//! | `oauth.rs`         | 6  OAuth2 authorization code + PKCE, client credentials |
//! | `mock_provider.rs` | (an in-process OAuth2 provider to log in with) |
//! | `totp.rs`          | bonus: TOTP two-factor codes (RFC 6238) |
//! | `app.rs`           | the routes and shared state |

pub mod api_keys;
pub mod app;
pub mod mock_provider;
pub mod oauth;
pub mod password;
pub mod rbac;
pub mod sessions;
pub mod tokens;
pub mod totp;
pub mod users;

/// Random bytes from the OS-seeded generator, hex encoded.
pub fn random_hex(bytes: usize) -> String {
    (0..bytes)
        .map(|_| format!("{:02x}", rand::random::<u8>()))
        .collect()
}

/// Starts a router on a background task; returns its base URL.
pub async fn spawn(router: axum::Router, addr: &str) -> String {
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    let local = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, router).await.expect("server") });
    format!("http://{local}")
}
