//! Exercise 2: JSON Web Tokens.
//!
//! A JWT is `base64(header).base64(claims).signature`. Anyone can *read*
//! the claims (they're not encrypted -- never put secrets in them); only the
//! holder of the key can produce a valid signature, so the server can trust
//! the claims without a database lookup.
//!
//! Two kinds of token:
//! * **access** -- short-lived (15 min), sent with every request.
//! * **refresh** -- long-lived (7 days), only exchanged for a new pair.
//!   *Rotated*: each refresh token works once. Using one twice means it was
//!   stolen, so we revoke it.

use crate::rbac::Role;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub const ACCESS_TTL_SECS: i64 = 15 * 60;
pub const REFRESH_TTL_SECS: i64 = 7 * 24 * 3600;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    pub sub: u64, // subject: the user id
    pub username: String,
    pub role: Role,
    pub typ: TokenType,
    pub jti: String, // unique token id, for revocation
    pub iat: i64,    // issued at (unix seconds)
    pub exp: i64,    // expires at
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TokenError {
    #[error("invalid token")]
    Invalid,
    #[error("token expired")]
    Expired,
    #[error("wrong token type")]
    WrongType,
    #[error("token revoked")]
    Revoked,
}

#[derive(Clone)]
pub struct TokenService {
    encoding: EncodingKey,
    decoding: DecodingKey,
    revoked: Arc<Mutex<HashSet<String>>>,
}

pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after 1970")
        .as_secs() as i64
}

impl TokenService {
    /// HS256 with a shared secret. At least 32 random bytes; load it from
    /// configuration, never from source code.
    pub fn new(secret: &[u8]) -> Self {
        TokenService {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
            revoked: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// `ttl` may be negative -- handy for testing expiry.
    pub fn issue(
        &self,
        user_id: u64,
        username: &str,
        role: Role,
        typ: TokenType,
        ttl: i64,
    ) -> String {
        let iat = now();
        let claims = Claims {
            sub: user_id,
            username: username.into(),
            role,
            typ,
            jti: crate::random_hex(16),
            iat,
            exp: iat + ttl,
        };
        encode(&Header::new(Algorithm::HS256), &claims, &self.encoding)
            .expect("HS256 encoding can't fail")
    }

    pub fn issue_pair(&self, user_id: u64, username: &str, role: Role) -> (String, String) {
        (
            self.issue(user_id, username, role, TokenType::Access, ACCESS_TTL_SECS),
            self.issue(
                user_id,
                username,
                role,
                TokenType::Refresh,
                REFRESH_TTL_SECS,
            ),
        )
    }

    /// Checks signature, expiry (no leeway), type and revocation.
    ///
    /// `Validation::new(HS256)` pins the algorithm: a token claiming
    /// `"alg": "none"` or a different algorithm is rejected -- the classic JWT
    /// vulnerability where the attacker picks the verification method.
    pub fn verify(&self, token: &str, expected: TokenType) -> Result<Claims, TokenError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 0;
        let data =
            decode::<Claims>(token, &self.decoding, &validation).map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => TokenError::Expired,
                _ => TokenError::Invalid,
            })?;
        let claims = data.claims;
        if claims.typ != expected {
            return Err(TokenError::WrongType);
        }
        if self.revoked.lock().unwrap().contains(&claims.jti) {
            return Err(TokenError::Revoked);
        }
        Ok(claims)
    }

    pub fn revoke(&self, jti: &str) {
        self.revoked.lock().unwrap().insert(jti.to_string());
    }

    /// Refresh-token rotation: the old token is revoked and a new pair
    /// issued. Presenting a revoked refresh token fails -- a stolen-and-used
    /// token is detected the next time either party uses it.
    ///
    /// (Real systems also revoke the whole token *family* on reuse, and keep
    /// the revocation list in shared storage with an expiry.)
    pub fn refresh(&self, refresh_token: &str) -> Result<(String, String), TokenError> {
        let claims = self.verify(refresh_token, TokenType::Refresh)?;
        self.revoke(&claims.jti);
        Ok(self.issue_pair(claims.sub, &claims.username, claims.role))
    }
}
