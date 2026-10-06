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
        let mut out = [0u8; PACKET_LEN];
        out[..2].copy_from_slice(&self.seq.to_be_bytes());
        out[2..].copy_from_slice(&self.sent_nanos.to_be_bytes());
        out
    }

    /// `None` unless exactly `PACKET_LEN` bytes.
    pub fn decode(bytes: &[u8]) -> Option<Ping> {
        let bytes: &[u8; PACKET_LEN] = bytes.try_into().ok()?;
        Some(Ping {
            seq: u16::from_be_bytes([bytes[0], bytes[1]]),
            sent_nanos: u64::from_be_bytes(bytes[2..].try_into().expect("8 bytes")),
        })
    }
}

/// Echo every valid ping back to its sender. With `drop_every: Some(k)`,
/// silently drop every k-th one (to simulate a lossy network).
pub async fn serve_pong(socket: UdpSocket, drop_every: Option<u32>) -> io::Result<()> {
    let mut buf = [0u8; 1500];
    let mut count: u32 = 0;
    loop {
        let (n, from) = socket.recv_from(&mut buf).await?;
        if Ping::decode(&buf[..n]).is_none() {
            continue;
        }
        count += 1;
        if drop_every.is_some_and(|k| k > 0 && count.is_multiple_of(k)) {
            continue;
        }
        socket.send_to(&buf[..n], from).await?;
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PingStats {
    pub sent: u32,
    pub received: u32,
    pub rtts: Vec<Duration>,
}

impl PingStats {
    pub fn loss_percent(&self) -> f64 {
        if self.sent == 0 {
            0.0
        } else {
            100.0 * (self.sent - self.received) as f64 / self.sent as f64
        }
    }

    pub fn min(&self) -> Option<Duration> {
        self.rtts.iter().min().copied()
    }

    pub fn max(&self) -> Option<Duration> {
        self.rtts.iter().max().copied()
    }

    pub fn avg(&self) -> Option<Duration> {
        (!self.rtts.is_empty()).then(|| self.rtts.iter().sum::<Duration>() / self.rtts.len() as u32)
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
    let socket = UdpSocket::bind("127.0.0.1:0").await?;
    socket.connect(server).await?;
    let start = Instant::now();
    let mut stats = PingStats::default();
    let mut buf = [0u8; 64];
    for seq in 0..count {
        let sent_nanos = start.elapsed().as_nanos() as u64;
        socket.send(&Ping { seq, sent_nanos }.encode()).await?;
        stats.sent += 1;
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            match tokio::time::timeout_at(deadline, socket.recv(&mut buf)).await {
                Err(_) => break, // timed out: lost
                Ok(Err(e)) if e.kind() == io::ErrorKind::ConnectionRefused => break,
                Ok(Err(e)) => return Err(e),
                Ok(Ok(n)) => {
                    if let Some(pong) = Ping::decode(&buf[..n]).filter(|p| p.seq == seq) {
                        let now = start.elapsed().as_nanos() as u64;
                        stats.received += 1;
                        stats
                            .rtts
                            .push(Duration::from_nanos(now.saturating_sub(pong.sent_nanos)));
                        break;
                    }
                }
            }
        }
        tokio::time::sleep(interval).await;
    }
    Ok(stats)
}
