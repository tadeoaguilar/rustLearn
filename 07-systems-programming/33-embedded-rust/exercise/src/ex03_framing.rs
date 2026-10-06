//! Exercise 3: a serial protocol -- COBS framing with a CRC.
//!
//! A UART delivers a stream of bytes, with no idea where messages begin or
//! end, and bytes get corrupted. A common robust format:
//!
//! 1. append a **CRC-16** of the payload (to detect corruption),
//! 2. **COBS**-encode payload + CRC, which removes every `0x00` byte,
//! 3. send it followed by a single `0x00` delimiter.
//!
//! A receiver that starts listening mid-stream just waits for the next
//! `0x00` and is in sync. No heap: everything goes into caller-provided
//! buffers or fixed-capacity `heapless::Vec`s.
//!
//! COBS (Consistent Overhead Byte Stuffing): the data is split at each zero;
//! each run of non-zero bytes is preceded by a code byte = run length + 1.
//! A code of 0xFF means 254 non-zero bytes with no zero after them.
//!
//! ```text
//! [11 22 00 33]  ->  [03 11 22 02 33]        (then the 00 delimiter)
//! ```

use heapless::Vec;

/// The largest payload a frame may carry.
pub const MAX_PAYLOAD: usize = 64;
/// Payload + CRC, COBS-encoded: at most one extra byte per 254, plus one.
pub const MAX_ENCODED: usize = MAX_PAYLOAD + 2 + (MAX_PAYLOAD + 2) / 254 + 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError {
    /// The output buffer is too small / the frame too long.
    Overflow,
    /// Not valid COBS (a zero inside, or a code running past the end).
    BadEncoding,
    /// The CRC doesn't match: corrupted in transit.
    BadCrc,
    /// Shorter than a CRC.
    TooShort,
}

/// CRC-16/CCITT-FALSE: polynomial 0x1021, initial value 0xFFFF, no
/// reflection, no final XOR. `crc16(b"123456789") == 0x29B1`.
pub fn crc16(data: &[u8]) -> u16 {
    todo!("Exercise 3")
}

fn put(i: usize, b: u8, output: &mut [u8]) -> Result<(), FrameError> {
    todo!("Exercise 3")
}

/// COBS-encode `input` into `output`; returns the encoded length (no
/// trailing delimiter).
pub fn cobs_encode(input: &[u8], output: &mut [u8]) -> Result<usize, FrameError> {
    todo!("Exercise 3")
}

/// COBS-decode `input` (without the delimiter) into `output`; returns the
/// decoded length.
pub fn cobs_decode(input: &[u8], output: &mut [u8]) -> Result<usize, FrameError> {
    todo!("Exercise 3")
}

/// An encoded frame, delimiter included.
pub type Frame = Vec<u8, { MAX_ENCODED + 1 }>;

/// A complete frame for `payload`: COBS(payload + CRC big-endian) + `0x00`.
pub fn encode_frame(payload: &[u8]) -> Result<Frame, FrameError> {
    todo!("Exercise 3")
}

/// Decode one frame's bytes (without the delimiter) and check its CRC.
pub fn decode_frame(encoded: &[u8]) -> Result<Vec<u8, MAX_PAYLOAD>, FrameError> {
    todo!("Exercise 3")
}

/// Feeds on bytes one at a time (as a UART interrupt delivers them) and
/// yields a result at each delimiter.
pub struct FrameDecoder {
    buf: Vec<u8, MAX_ENCODED>,
    overflowed: bool,
}

impl Default for FrameDecoder {
    fn default() -> Self {
        todo!("Exercise 3")
    }
}

impl FrameDecoder {
    pub const fn new() -> Self {
        todo!()
    }

    /// `None` until a `0x00` arrives; then the frame (or why it was bad).
    /// An empty frame (two delimiters in a row) is skipped. After an
    /// overflow, everything up to the next delimiter is discarded.
    pub fn push(&mut self, byte: u8) -> Option<Result<Vec<u8, MAX_PAYLOAD>, FrameError>> {
        todo!("Exercise 3")
    }
}
