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
    x.wrapping_sub(LO) & !x & HI
}

pub fn has_zero_byte(x: u64) -> bool {
    zero_byte_mask(x) != 0
}

/// `b` in every byte.
pub fn splat(b: u8) -> u64 {
    LO * b as u64
}

/// The first index of `needle`, eight bytes at a time.
pub fn find_byte(haystack: &[u8], needle: u8) -> Option<usize> {
    let pattern = splat(needle);
    let chunks = haystack.chunks_exact(8);
    let (count, rest) = (chunks.len(), chunks.remainder());
    for (i, chunk) in chunks.enumerate() {
        let mask = zero_byte_mask(u64::from_le_bytes(chunk.try_into().expect("8 bytes")) ^ pattern);
        if mask != 0 {
            // little-endian: the lowest set bit is the first matching byte
            return Some(i * 8 + mask.trailing_zeros() as usize / 8);
        }
    }
    let start = count * 8;
    rest.iter().position(|&b| b == needle).map(|p| start + p)
}

/// Count `needle` bytes, eight at a time. Exact: for counting, false
/// positives must go, so each byte is tested on its own 7 low bits.
pub fn count_byte(haystack: &[u8], needle: u8) -> usize {
    let pattern = splat(needle);
    let chunks = haystack.chunks_exact(8);
    let rest = chunks.remainder();
    let mut count = 0usize;
    for chunk in chunks {
        let x = u64::from_le_bytes(chunk.try_into().expect("8 bytes")) ^ pattern; // zero bytes where equal
        // Adding 0x7F to a byte's low 7 bits sets its high bit iff they're
        // non-zero (and never carries into the next byte); OR in the byte's
        // own high bit: the result's high bit is set iff the byte is non-zero.
        let low7 = (x & !HI).wrapping_add(!HI);
        let nonzero = (low7 | x) & HI;
        count += 8 - nonzero.count_ones() as usize;
    }
    count + rest.iter().filter(|&&b| b == needle).count()
}
