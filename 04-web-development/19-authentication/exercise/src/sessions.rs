//! Exercise 4: server-side sessions in cookies, and CSRF protection.
//!
//! The browser holds only a random session id; everything else stays on the
//! server, so a session can be revoked instantly (unlike a JWT). The cookie
//! is set with:
//! * `HttpOnly` -- JavaScript can't read it, so an XSS bug can't steal it
//! * `SameSite=Lax` -- not sent on cross-site POSTs (a first CSRF defence)
//! * `Secure` -- HTTPS only (configurable here, because tests use plain HTTP)
//!
//! Cookies are sent automatically by the browser -- which is exactly what
//! CSRF exploits: evil.example submits a form to your site and the browser
//! attaches the victim's cookie. So every state-changing request must also
//! carry a **CSRF token** that evil.example can't know: it's stored in the
//! session and returned to *our* pages only.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const COOKIE_NAME: &str = "session";
pub const SESSION_TTL: Duration = Duration::from_secs(8 * 3600);

#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: u64,
    pub csrf_token: String,
    pub expires_at: Instant,
}

#[derive(Debug, Clone, Default)]
pub struct SessionStore {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        todo!("Exercise 4")
    }

    /// Returns (session id, csrf token). 32 random bytes each: unguessable.
    pub fn create(&self, user_id: u64) -> (String, String) {
        todo!("Exercise 4")
    }

    /// Expired sessions are removed when looked up.
    pub fn get(&self, id: &str) -> Option<Session> {
        todo!("Exercise 4")
    }

    pub fn destroy(&self, id: &str) {
        todo!("Exercise 4")
    }

    /// For tests: make a session look old.
    pub fn expire_now(&self, id: &str) {
        todo!("Exercise 4")
    }
}

/// Constant-time comparison: `==` on strings stops at the first difference,
/// and the response time leaks how many leading characters were right.
pub fn tokens_match(a: &str, b: &str) -> bool {
    todo!("Exercise 4")
}
