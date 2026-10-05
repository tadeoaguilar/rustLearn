//! A small WebSocket client (tokio-tungstenite), used by the demo and the
//! tests -- and the bonus: reconnecting with exponential backoff.

use crate::protocol::{ClientMessage, ServerMessage};
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

pub type Error = tokio_tungstenite::tungstenite::Error;

pub struct WsClient {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WsClient {
    /// `url` like "ws://127.0.0.1:3000/ws/chat?name=alice".
    pub async fn connect(url: &str) -> Result<WsClient, Error> {
        let (stream, _response) = connect_async(url).await?;
        Ok(WsClient { stream })
    }

    pub async fn send(&mut self, msg: &ClientMessage) -> Result<(), Error> {
        self.send_text(&serde_json::to_string(msg).expect("serialisable"))
            .await
    }

    pub async fn send_text(&mut self, text: &str) -> Result<(), Error> {
        self.stream.send(Message::Text(text.into())).await
    }

    pub async fn send_binary(&mut self, bytes: Vec<u8>) -> Result<(), Error> {
        self.stream.send(Message::Binary(bytes.into())).await
    }

    /// Next frame of data. Pings are answered automatically by tungstenite
    /// while we read; they're skipped here. None when the connection closed
    /// or nothing arrived within `wait`.
    pub async fn next_frame(&mut self, wait: Duration) -> Option<Message> {
        // One deadline for the whole call. (A fresh `timeout(wait, ..)` per
        // iteration would never expire while the server keeps pinging.)
        let deadline = tokio::time::Instant::now() + wait;
        loop {
            match tokio::time::timeout_at(deadline, self.stream.next()).await {
                Ok(Some(Ok(Message::Ping(_) | Message::Pong(_) | Message::Frame(_)))) => continue,
                Ok(Some(Ok(Message::Close(_)))) | Ok(Some(Err(_))) | Ok(None) | Err(_) => {
                    return None;
                }
                Ok(Some(Ok(msg))) => return Some(msg),
            }
        }
    }

    pub async fn recv_text(&mut self, wait: Duration) -> Option<String> {
        match self.next_frame(wait).await? {
            Message::Text(t) => Some(t.to_string()),
            _ => None,
        }
    }

    pub async fn recv(&mut self, wait: Duration) -> Option<ServerMessage> {
        serde_json::from_str(&self.recv_text(wait).await?).ok()
    }

    /// Skips messages until one matches.
    pub async fn recv_until(
        &mut self,
        wait: Duration,
        pred: impl Fn(&ServerMessage) -> bool,
    ) -> Option<ServerMessage> {
        while let Some(m) = self.recv(wait).await {
            if pred(&m) {
                return Some(m);
            }
        }
        None
    }

    /// True if the server has closed the connection (reads until it ends).
    pub async fn is_closed_within(&mut self, wait: Duration) -> bool {
        matches!(
            tokio::time::timeout(wait, async {
                while let Some(Ok(msg)) = self.stream.next().await {
                    if let Message::Close(_) = msg {
                        break;
                    }
                }
            })
            .await,
            Ok(())
        )
    }

    pub async fn close(mut self) {
        let _ = self.stream.close(None).await;
    }

    /// Exposes the raw stream, e.g. to simulate a client that stops reading.
    pub fn into_inner(self) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
        self.stream
    }
}

/// Bonus: exponential backoff. Attempt 0 waits `base`, then 2x, 4x...
/// capped at `max`. Production clients add random *jitter* so thousands of
/// clients disconnected by the same outage don't reconnect in lockstep.
#[derive(Debug, Clone, Copy)]
pub struct Backoff {
    pub base: Duration,
    pub max: Duration,
}

impl Backoff {
    pub fn delay(&self, attempt: u32) -> Duration {
        self.base
            .saturating_mul(2u32.saturating_pow(attempt))
            .min(self.max)
    }
}

/// Connects, reads text messages until the server hangs up, waits, and
/// reconnects -- `connections` times in total. Failed connection attempts
/// count against the backoff too. Returns every text message received.
pub async fn collect_with_reconnects(
    url: &str,
    connections: usize,
    backoff: Backoff,
) -> Vec<String> {
    let mut received = Vec::new();
    let mut attempt = 0u32;
    let mut done = 0;
    while done < connections {
        match WsClient::connect(url).await {
            Ok(mut client) => {
                attempt = 0; // a successful connection resets the backoff
                while let Some(text) = client.recv_text(Duration::from_secs(2)).await {
                    received.push(text);
                }
                done += 1;
            }
            Err(_) => attempt += 1,
        }
        if done < connections {
            tokio::time::sleep(backoff.delay(attempt)).await;
        }
    }
    received
}
