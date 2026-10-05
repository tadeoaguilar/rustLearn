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
    (0..20).map(|_| rand::random::<u8>()).collect() // 160 bits, as RFC 4226 recommends
}

/// HOTP (RFC 4226) for a counter value.
pub fn hotp(secret: &[u8], counter: u64, digits: u32) -> String {
    let mut mac = Hmac::<Sha1>::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(&counter.to_be_bytes());
    let hash = mac.finalize().into_bytes();
    // "Dynamic truncation": the low 4 bits of the last byte pick an offset.
    let offset = (hash[19] & 0x0f) as usize;
    let code = u32::from_be_bytes([
        hash[offset] & 0x7f,
        hash[offset + 1],
        hash[offset + 2],
        hash[offset + 3],
    ]);
    format!(
        "{:0width$}",
        code % 10u32.pow(digits),
        width = digits as usize
    )
}

/// TOTP: HOTP with the counter derived from the time.
pub fn totp(secret: &[u8], unix_time: u64, digits: u32) -> String {
    hotp(secret, unix_time / STEP_SECS, digits)
}

/// Accepts the current code and one step either side, to tolerate clock
/// drift and slow typing. (Production code also remembers the last accepted
/// step, so a code can't be replayed within its window.)
pub fn verify(secret: &[u8], code: &str, unix_time: u64) -> bool {
    let step = unix_time / STEP_SECS;
    [step.saturating_sub(1), step, step + 1]
        .iter()
        .any(|&s| crate::sessions::tokens_match(&hotp(secret, s, 6), code.trim()))
}

/// RFC 4648 base32, no padding -- the format authenticator apps expect.
pub fn base32(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut out = String::new();
    let (mut buffer, mut bits) = (0u32, 0u32);
    for &b in bytes {
        buffer = (buffer << 8) | u32::from(b);
        bits += 8;
        while bits >= 5 {
            out.push(ALPHABET[((buffer >> (bits - 5)) & 31) as usize] as char);
            bits -= 5;
        }
    }
    if bits > 0 {
        out.push(ALPHABET[((buffer << (5 - bits)) & 31) as usize] as char);
    }
    out
}

/// Encode this in a QR code and an authenticator app can scan it.
pub fn otpauth_url(issuer: &str, account: &str, secret: &[u8]) -> String {
    format!(
        "otpauth://totp/{issuer}:{account}?secret={}&issuer={issuer}&digits=6&period=30",
        base32(secret)
    )
}
