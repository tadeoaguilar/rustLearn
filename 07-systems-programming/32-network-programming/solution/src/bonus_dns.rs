//! Bonus: speaking DNS -- building a query, parsing a response.
//!
//! DNS messages are a compact binary format: a 12-byte header, then
//! questions and answers. Names are sequences of length-prefixed labels
//! (`3www7example3com0`), and to save space a name may end in a *pointer*
//! (two bytes, top bits `11`) to a name earlier in the message. Parsing
//! pointers safely -- in range, no loops -- is the classic trap.

use std::net::Ipv4Addr;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DnsError {
    #[error("a label is empty or longer than 63 bytes")]
    BadLabel,
    #[error("the name is longer than 253 bytes")]
    NameTooLong,
    #[error("the message ends early")]
    Truncated,
    #[error("a compression pointer loops or points forward")]
    BadPointer,
}

pub const TYPE_A: u16 = 1;
pub const CLASS_IN: u16 = 1;

/// `www.example.com` -> `3www7example3com0`.
pub fn encode_name(name: &str) -> Result<Vec<u8>, DnsError> {
    let name = name.trim_end_matches('.');
    if name.len() > 253 {
        return Err(DnsError::NameTooLong);
    }
    let mut out = Vec::with_capacity(name.len() + 2);
    for label in name.split('.') {
        if label.is_empty() || label.len() > 63 {
            return Err(DnsError::BadLabel);
        }
        out.push(label.len() as u8);
        out.extend_from_slice(label.as_bytes());
    }
    out.push(0);
    Ok(out)
}

/// A query for `name`'s A records: header (id, recursion desired, one
/// question), then the question.
pub fn build_query(id: u16, name: &str) -> Result<Vec<u8>, DnsError> {
    let mut out = Vec::with_capacity(32);
    out.extend_from_slice(&id.to_be_bytes());
    out.extend_from_slice(&0x0100u16.to_be_bytes()); // RD
    out.extend_from_slice(&[0, 1, 0, 0, 0, 0, 0, 0]); // QD=1, AN=NS=AR=0
    out.extend_from_slice(&encode_name(name)?);
    out.extend_from_slice(&TYPE_A.to_be_bytes());
    out.extend_from_slice(&CLASS_IN.to_be_bytes());
    Ok(out)
}

fn u16_at(packet: &[u8], i: usize) -> Result<u16, DnsError> {
    Ok(u16::from_be_bytes([
        *packet.get(i).ok_or(DnsError::Truncated)?,
        *packet.get(i + 1).ok_or(DnsError::Truncated)?,
    ]))
}

/// Decode the name at `offset`, following pointers. Returns the name and
/// the offset just after it *in the original position* (pointers don't move
/// the reader past more than their two bytes). Pointers must point
/// backwards, which also rules out loops.
pub fn decode_name(packet: &[u8], offset: usize) -> Result<(String, usize), DnsError> {
    let mut labels = Vec::new();
    let mut pos = offset;
    let mut end = None;
    loop {
        let len = *packet.get(pos).ok_or(DnsError::Truncated)? as usize;
        match len {
            0 => {
                let end = end.unwrap_or(pos + 1);
                return Ok((labels.join("."), end));
            }
            l if l & 0xC0 == 0xC0 => {
                let target = (u16_at(packet, pos)? & 0x3FFF) as usize;
                if target >= pos {
                    return Err(DnsError::BadPointer);
                }
                end.get_or_insert(pos + 2);
                pos = target;
            }
            l if l <= 63 => {
                let label = packet
                    .get(pos + 1..pos + 1 + l)
                    .ok_or(DnsError::Truncated)?;
                labels.push(String::from_utf8_lossy(label).into_owned());
                pos += 1 + l;
            }
            _ => return Err(DnsError::BadLabel),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub name: String,
    pub rtype: u16,
    pub ttl: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsResponse {
    pub id: u16,
    /// 0 = no error, 3 = no such name, ...
    pub rcode: u8,
    pub answers: Vec<Answer>,
}

impl DnsResponse {
    pub fn a_records(&self) -> Vec<Ipv4Addr> {
        self.answers
            .iter()
            .filter(|a| a.rtype == TYPE_A && a.data.len() == 4)
            .map(|a| Ipv4Addr::new(a.data[0], a.data[1], a.data[2], a.data[3]))
            .collect()
    }
}

/// Parse a response: header, skip the questions, read the answers.
pub fn parse_response(packet: &[u8]) -> Result<DnsResponse, DnsError> {
    let id = u16_at(packet, 0)?;
    let rcode = (u16_at(packet, 2)? & 0x000F) as u8;
    let questions = u16_at(packet, 4)?;
    let answer_count = u16_at(packet, 6)?;
    let mut pos = 12;
    for _ in 0..questions {
        let (_, next) = decode_name(packet, pos)?;
        pos = next + 4; // type, class
    }
    let mut answers = Vec::new();
    for _ in 0..answer_count {
        let (name, next) = decode_name(packet, pos)?;
        let rtype = u16_at(packet, next)?;
        let ttl = (u16_at(packet, next + 4)? as u32) << 16 | u16_at(packet, next + 6)? as u32;
        let len = u16_at(packet, next + 8)? as usize;
        let data = packet
            .get(next + 10..next + 10 + len)
            .ok_or(DnsError::Truncated)?
            .to_vec();
        pos = next + 10 + len;
        answers.push(Answer {
            name,
            rtype,
            ttl,
            data,
        });
    }
    Ok(DnsResponse { id, rcode, answers })
}
