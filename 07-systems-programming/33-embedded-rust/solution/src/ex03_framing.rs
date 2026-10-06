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
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }
    crc
}

fn put(i: usize, b: u8, output: &mut [u8]) -> Result<(), FrameError> {
    *output.get_mut(i).ok_or(FrameError::Overflow)? = b;
    Ok(())
}

/// COBS-encode `input` into `output`; returns the encoded length (no
/// trailing delimiter).
pub fn cobs_encode(input: &[u8], output: &mut [u8]) -> Result<usize, FrameError> {
    let mut code_index = 0; // where the current run's code byte goes
    let mut out = 1;
    let mut code: u8 = 1;
    for (i, &byte) in input.iter().enumerate() {
        if byte == 0 {
            put(code_index, code, output)?;
            code_index = out;
            out += 1;
            code = 1;
        } else {
            put(out, byte, output)?;
            out += 1;
            code += 1;
            // A full run of 254: close it -- and only open a new one if
            // more input follows (the canonical encoding has no empty
            // trailing run).
            if code == 0xFF && i + 1 < input.len() {
                put(code_index, code, output)?;
                code_index = out;
                out += 1;
                code = 1;
            }
        }
    }
    put(code_index, code, output)?;
    Ok(out)
}

/// COBS-decode `input` (without the delimiter) into `output`; returns the
/// decoded length.
pub fn cobs_decode(input: &[u8], output: &mut [u8]) -> Result<usize, FrameError> {
    let mut i = 0;
    let mut out = 0;
    while i < input.len() {
        let code = input[i];
        if code == 0 {
            return Err(FrameError::BadEncoding);
        }
        i += 1;
        for _ in 1..code {
            let byte = *input.get(i).ok_or(FrameError::BadEncoding)?;
            if byte == 0 {
                return Err(FrameError::BadEncoding);
            }
            *output.get_mut(out).ok_or(FrameError::Overflow)? = byte;
            out += 1;
            i += 1;
        }
        // A zero follows each run, except after 0xFF runs and at the end.
        if code != 0xFF && i < input.len() {
            *output.get_mut(out).ok_or(FrameError::Overflow)? = 0;
            out += 1;
        }
    }
    Ok(out)
}

/// An encoded frame, delimiter included.
pub type Frame = Vec<u8, { MAX_ENCODED + 1 }>;

/// A complete frame for `payload`: COBS(payload + CRC big-endian) + `0x00`.
pub fn encode_frame(payload: &[u8]) -> Result<Frame, FrameError> {
    if payload.len() > MAX_PAYLOAD {
        return Err(FrameError::Overflow);
    }
    let mut raw: Vec<u8, { MAX_PAYLOAD + 2 }> = Vec::new();
    raw.extend_from_slice(payload)
        .map_err(|_| FrameError::Overflow)?;
    raw.extend_from_slice(&crc16(payload).to_be_bytes())
        .map_err(|_| FrameError::Overflow)?;
    let mut buf = [0u8; MAX_ENCODED];
    let n = cobs_encode(&raw, &mut buf)?;
    let mut frame = Vec::new();
    frame
        .extend_from_slice(&buf[..n])
        .map_err(|_| FrameError::Overflow)?;
    frame.push(0).map_err(|_| FrameError::Overflow)?;
    Ok(frame)
}

/// Decode one frame's bytes (without the delimiter) and check its CRC.
pub fn decode_frame(encoded: &[u8]) -> Result<Vec<u8, MAX_PAYLOAD>, FrameError> {
    let mut buf = [0u8; MAX_PAYLOAD + 2];
    let n = cobs_decode(encoded, &mut buf)?;
    if n < 2 {
        return Err(FrameError::TooShort);
    }
    let (payload, crc) = buf[..n].split_at(n - 2);
    if crc16(payload).to_be_bytes() != crc {
        return Err(FrameError::BadCrc);
    }
    Vec::from_slice(payload).map_err(|_| FrameError::Overflow)
}

/// Feeds on bytes one at a time (as a UART interrupt delivers them) and
/// yields a result at each delimiter.
pub struct FrameDecoder {
    buf: Vec<u8, MAX_ENCODED>,
    overflowed: bool,
}

impl Default for FrameDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameDecoder {
    pub const fn new() -> Self {
        FrameDecoder {
            buf: Vec::new(),
            overflowed: false,
        }
    }

    /// `None` until a `0x00` arrives; then the frame (or why it was bad).
    /// An empty frame (two delimiters in a row) is skipped. After an
    /// overflow, everything up to the next delimiter is discarded.
    pub fn push(&mut self, byte: u8) -> Option<Result<Vec<u8, MAX_PAYLOAD>, FrameError>> {
        if byte != 0 {
            if self.buf.push(byte).is_err() {
                self.overflowed = true;
            }
            return None;
        }
        let result = if self.overflowed {
            Some(Err(FrameError::Overflow))
        } else if self.buf.is_empty() {
            None
        } else {
            Some(decode_frame(&self.buf))
        };
        self.buf.clear();
        self.overflowed = false;
        result
    }
}
