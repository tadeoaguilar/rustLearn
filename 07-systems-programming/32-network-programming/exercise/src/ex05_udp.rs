//! Exercise 5: UDP -- ping with loss and round-trip times.
//!
//! UDP sends independent datagrams: no connection, no ordering, no
//! retransmission. A datagram arrives whole or not at all. So the
//! application numbers its packets, notices which never came back, and
//! decides how long to wait. Fields go over the wire in *network byte order*
//! (big-endian), whatever the machine's own order is.
//!
//! ```text
//! ping/pong: [seq: u16 BE][sent: u64 BE nanoseconds since the client started]
//! ```

use std::io;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use tokio::net::UdpSocket;

pub const PACKET_LEN: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ping {
    pub seq: u16,
    pub sent_nanos: u64,
}

impl Ping {
    pub fn encode(&self) -> [u8; PACKET_LEN] {
        todo!("Exercise 5")
    }

    /// `None` unless exactly `PACKET_LEN` bytes.
    pub fn decode(bytes: &[u8]) -> Option<Ping> {
        todo!("Exercise 5")
    }
}

/// Echo every valid ping back to its sender. With `drop_every: Some(k)`,
/// silently drop every k-th one (to simulate a lossy network).
pub async fn serve_pong(socket: UdpSocket, drop_every: Option<u32>) -> io::Result<()> {
    todo!("Exercise 5")
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PingStats {
    pub sent: u32,
    pub received: u32,
    pub rtts: Vec<Duration>,
}

impl PingStats {
    pub fn loss_percent(&self) -> f64 {
        todo!("Exercise 5")
    }

    pub fn min(&self) -> Option<Duration> {
        todo!("Exercise 5")
    }

    pub fn max(&self) -> Option<Duration> {
        todo!("Exercise 5")
    }

    pub fn avg(&self) -> Option<Duration> {
        todo!("Exercise 5")
    }
}

/// Send `count` pings, `interval` apart, each waiting up to `timeout` for
/// its own pong (late pongs for earlier pings are ignored).
pub async fn ping(
    server: SocketAddr,
    count: u16,
    interval: Duration,
    timeout: Duration,
) -> io::Result<PingStats> {
    todo!("Exercise 5")
}
