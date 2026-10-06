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
    todo!("Bonus")
}

/// A query for `name`'s A records: header (id, recursion desired, one
/// question), then the question.
pub fn build_query(id: u16, name: &str) -> Result<Vec<u8>, DnsError> {
    todo!("Bonus")
}

fn u16_at(packet: &[u8], i: usize) -> Result<u16, DnsError> {
    todo!("Bonus")
}

/// Decode the name at `offset`, following pointers. Returns the name and
/// the offset just after it *in the original position* (pointers don't move
/// the reader past more than their two bytes). Pointers must point
/// backwards, which also rules out loops.
pub fn decode_name(packet: &[u8], offset: usize) -> Result<(String, usize), DnsError> {
    todo!("Bonus")
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
        todo!("Bonus")
    }
}

/// Parse a response: header, skip the questions, read the answers.
pub fn parse_response(packet: &[u8]) -> Result<DnsResponse, DnsError> {
    todo!("Bonus")
}
