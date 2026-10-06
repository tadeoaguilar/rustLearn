//! Bonus: SWAR -- SIMD within a register.
//!
//! Before (or without) vector instructions, a `u64` is eight bytes you can
//! test at once with arithmetic. The classic: does a word contain a zero
//! byte? `(x - 0x0101..01) & !x & 0x8080..80` is non-zero exactly when one
//! of its bytes is zero. XOR with a repeated byte turns "find byte b" into
//! "find zero". This is how `memchr` and `strlen` worked for decades.

const LO: u64 = 0x0101_0101_0101_0101;
const HI: u64 = 0x8080_8080_8080_8080;

/// Non-zero iff some byte of `x` is zero. The lowest set high bit marks the
/// first zero byte (bits above it may be false positives).
pub fn zero_byte_mask(x: u64) -> u64 {
    todo!("Bonus")
}

pub fn has_zero_byte(x: u64) -> bool {
    todo!("Bonus")
}

/// `b` in every byte.
pub fn splat(b: u8) -> u64 {
    todo!("Bonus")
}

/// The first index of `needle`, eight bytes at a time.
pub fn find_byte(haystack: &[u8], needle: u8) -> Option<usize> {
    todo!("Bonus")
}

/// Count `needle` bytes, eight at a time. Exact: for counting, false
/// positives must go, so each byte is tested on its own 7 low bits.
pub fn count_byte(haystack: &[u8], needle: u8) -> usize {
    todo!("Bonus")
}
