//! Bonus: TOTP two-factor codes (RFC 6238), as used by authenticator apps.
//!
//! The server and the phone share a secret. Every 30 seconds both compute
//! HMAC-SHA1(secret, number of 30-second steps since 1970), truncate it to a
//! 6-digit code, and compare. Nothing is sent over the network at enrolment
//! except the secret (in a QR code).

use hmac::{Hmac, Mac};
use sha1::Sha1;

pub const STEP_SECS: u64 = 30;

pub fn generate_secret() -> Vec<u8> {
    todo!("Bonus")
}

/// HOTP (RFC 4226) for a counter value.
pub fn hotp(secret: &[u8], counter: u64, digits: u32) -> String {
    todo!("Bonus")
}

/// TOTP: HOTP with the counter derived from the time.
pub fn totp(secret: &[u8], unix_time: u64, digits: u32) -> String {
    todo!("Bonus")
}

/// Accepts the current code and one step either side, to tolerate clock
/// drift and slow typing. (Production code also remembers the last accepted
/// step, so a code can't be replayed within its window.)
pub fn verify(secret: &[u8], code: &str, unix_time: u64) -> bool {
    todo!("Bonus")
}

/// RFC 4648 base32, no padding -- the format authenticator apps expect.
pub fn base32(bytes: &[u8]) -> String {
    todo!("Bonus")
}

/// Encode this in a QR code and an authenticator app can scan it.
pub fn otpauth_url(issuer: &str, account: &str, secret: &[u8]) -> String {
    todo!("Bonus")
}
