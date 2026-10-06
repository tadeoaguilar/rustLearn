//! Exercise 4: a custom binary protocol -- a chat server.
//!
//! TCP is a byte stream, not a message stream: one `write` may arrive as two
//! `read`s, two writes as one. So every message is *framed*: a 4-byte
//! big-endian length, then that many bytes. Inside a frame, a message is a
//! one-byte tag and its fields; strings are a 2-byte length and UTF-8 bytes.
//!
//! ```text
//! frame:   [len: u32 BE][payload: len bytes]
//! payload: [tag: u8][fields...]          string = [len: u16 BE][utf-8]
//! ```

use std::collections::BTreeSet;
use std::io;
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

/// Frames larger than this are refused (a length field is attacker-controlled).
pub const MAX_FRAME: usize = 64 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("frame of {0} bytes exceeds the limit")]
    TooLarge(usize),
    #[error("truncated message")]
    Truncated,
    #[error("unknown message tag {0}")]
    UnknownTag(u8),
    #[error("invalid UTF-8")]
    BadUtf8,
    #[error("{0}")]
    Io(#[from] io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientMsg {
    Join { name: String },
    Say { text: String },
    Leave,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerMsg {
    /// Sent to a client that joined: everyone in the room, itself included.
    Welcome {
        users: Vec<String>,
    },
    Joined {
        name: String,
    },
    Left {
        name: String,
    },
    Message {
        from: String,
        text: String,
    },
    Error {
        reason: String,
    },
}

fn put_str(out: &mut Vec<u8>, s: &str) {
    let len = u16::try_from(s.len()).expect("strings fit in a frame");
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(s.as_bytes());
}

/// Reads fields out of a payload.
struct Cursor<'a>(&'a [u8]);

impl Cursor<'_> {
    fn u8(&mut self) -> Result<u8, ProtocolError> {
        let (&b, rest) = self.0.split_first().ok_or(ProtocolError::Truncated)?;
        self.0 = rest;
        Ok(b)
    }
    fn u16(&mut self) -> Result<u16, ProtocolError> {
        Ok(u16::from_be_bytes([self.u8()?, self.u8()?]))
    }
    fn string(&mut self) -> Result<String, ProtocolError> {
        let len = self.u16()? as usize;
        if self.0.len() < len {
            return Err(ProtocolError::Truncated);
        }
        let (s, rest) = self.0.split_at(len);
        self.0 = rest;
        String::from_utf8(s.to_vec()).map_err(|_| ProtocolError::BadUtf8)
    }
}

impl ClientMsg {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        match self {
            ClientMsg::Join { name } => {
                out.push(1);
                put_str(&mut out, name);
            }
            ClientMsg::Say { text } => {
                out.push(2);
                put_str(&mut out, text);
            }
            ClientMsg::Leave => out.push(3),
        }
        out
    }

    pub fn decode(payload: &[u8]) -> Result<Self, ProtocolError> {
        let mut c = Cursor(payload);
        match c.u8()? {
            1 => Ok(ClientMsg::Join { name: c.string()? }),
            2 => Ok(ClientMsg::Say { text: c.string()? }),
            3 => Ok(ClientMsg::Leave),
            tag => Err(ProtocolError::UnknownTag(tag)),
        }
    }
}

impl ServerMsg {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        match self {
            ServerMsg::Welcome { users } => {
                out.push(10);
                out.extend_from_slice(&(users.len() as u16).to_be_bytes());
                for user in users {
                    put_str(&mut out, user);
                }
            }
            ServerMsg::Joined { name } => {
                out.push(11);
                put_str(&mut out, name);
            }
            ServerMsg::Left { name } => {
                out.push(12);
                put_str(&mut out, name);
            }
            ServerMsg::Message { from, text } => {
                out.push(13);
                put_str(&mut out, from);
                put_str(&mut out, text);
            }
            ServerMsg::Error { reason } => {
                out.push(14);
                put_str(&mut out, reason);
            }
        }
        out
    }

    pub fn decode(payload: &[u8]) -> Result<Self, ProtocolError> {
        let mut c = Cursor(payload);
        match c.u8()? {
            10 => {
                let n = c.u16()?;
                let users = (0..n).map(|_| c.string()).collect::<Result<_, _>>()?;
                Ok(ServerMsg::Welcome { users })
            }
            11 => Ok(ServerMsg::Joined { name: c.string()? }),
            12 => Ok(ServerMsg::Left { name: c.string()? }),
            13 => Ok(ServerMsg::Message {
                from: c.string()?,
                text: c.string()?,
            }),
            14 => Ok(ServerMsg::Error {
                reason: c.string()?,
            }),
            tag => Err(ProtocolError::UnknownTag(tag)),
        }
    }
}

/// Write one frame.
pub async fn write_frame<W: AsyncWrite + Unpin>(writer: &mut W, payload: &[u8]) -> io::Result<()> {
    writer
        .write_all(&(payload.len() as u32).to_be_bytes())
        .await?;
    writer.write_all(payload).await?;
    writer.flush().await
}

/// Read one frame; `Ok(None)` on a clean close between frames.
pub async fn read_frame<R: AsyncRead + Unpin>(
    reader: &mut R,
) -> Result<Option<Vec<u8>>, ProtocolError> {
    let mut len = [0u8; 4];
    match reader.read_exact(&mut len).await {
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e.into()),
    }
    let len = u32::from_be_bytes(len) as usize;
    if len > MAX_FRAME {
        return Err(ProtocolError::TooLarge(len));
    }
    let mut payload = vec![0; len];
    reader
        .read_exact(&mut payload)
        .await
        .map_err(|e| match e.kind() {
            io::ErrorKind::UnexpectedEof => ProtocolError::Truncated,
            _ => e.into(),
        })?;
    Ok(Some(payload))
}

/// 1 to 16 ASCII letters, digits or underscores.
pub fn valid_name(name: &str) -> bool {
    (1..=16).contains(&name.len()) && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// The room: who's in it, and the channel every message is broadcast on.
pub struct Room {
    users: Mutex<BTreeSet<String>>,
    tx: broadcast::Sender<ServerMsg>,
}

impl Default for Room {
    fn default() -> Self {
        Self::new()
    }
}

impl Room {
    pub fn new() -> Self {
        Room {
            users: Mutex::new(BTreeSet::new()),
            tx: broadcast::channel(256).0,
        }
    }

    pub fn users(&self) -> Vec<String> {
        self.users.lock().expect("lock").iter().cloned().collect()
    }
}

async fn send<W: AsyncWrite + Unpin>(writer: &mut W, msg: &ServerMsg) -> io::Result<()> {
    write_frame(writer, &msg.encode()).await
}

/// One client: the first message must be a `Join` with a valid, unused
/// name (else an `Error` and the connection closes). Then `Welcome`, a
/// `Joined` to everyone else, and messages both ways until `Leave` or
/// disconnect, after which everyone gets `Left`.
pub async fn handle_client(stream: TcpStream, room: Arc<Room>) -> Result<(), ProtocolError> {
    let (mut reader, mut writer) = stream.into_split();
    let name = match read_frame(&mut reader)
        .await?
        .map(|f| ClientMsg::decode(&f))
    {
        Some(Ok(ClientMsg::Join { name })) => name,
        _ => {
            send(
                &mut writer,
                &ServerMsg::Error {
                    reason: "join first".into(),
                },
            )
            .await?;
            return Ok(());
        }
    };
    let reason = if !valid_name(&name) {
        Some("invalid name")
    } else if !room.users.lock().expect("lock").insert(name.clone()) {
        Some("name taken")
    } else {
        None
    };
    if let Some(reason) = reason {
        send(
            &mut writer,
            &ServerMsg::Error {
                reason: reason.into(),
            },
        )
        .await?;
        return Ok(());
    }

    // Subscribe before announcing, so nothing sent after our Welcome is missed.
    let mut rx = room.tx.subscribe();
    send(
        &mut writer,
        &ServerMsg::Welcome {
            users: room.users(),
        },
    )
    .await?;
    let _ = room.tx.send(ServerMsg::Joined { name: name.clone() });

    let result = async {
        loop {
            tokio::select! {
                frame = read_frame(&mut reader) => match frame?.map(|f| ClientMsg::decode(&f)).transpose()? {
                    Some(ClientMsg::Say { text }) => {
                        let _ = room.tx.send(ServerMsg::Message { from: name.clone(), text });
                    }
                    Some(ClientMsg::Leave) | None => return Ok(()),
                    Some(ClientMsg::Join { .. }) => send(&mut writer, &ServerMsg::Error { reason: "already joined".into() }).await?,
                },
                msg = rx.recv() => match msg {
                    Ok(ServerMsg::Joined { name: joined }) if joined == name => {}
                    Ok(msg) => send(&mut writer, &msg).await?,
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => return Ok(()),
                },
            }
        }
    }
    .await;

    room.users.lock().expect("lock").remove(&name);
    let _ = room.tx.send(ServerMsg::Left { name });
    result
}

pub async fn serve_chat(listener: TcpListener, room: Arc<Room>) -> io::Result<()> {
    loop {
        let (stream, _) = listener.accept().await?;
        let room = room.clone();
        tokio::spawn(async move {
            let _ = handle_client(stream, room).await;
        });
    }
}

/// A client, for tests and the demo.
pub struct ChatClient {
    reader: tokio::net::tcp::OwnedReadHalf,
    writer: tokio::net::tcp::OwnedWriteHalf,
}

impl ChatClient {
    /// Connect and join; returns the client and the server's first reply
    /// (`Welcome`, or `Error`).
    pub async fn join(
        addr: std::net::SocketAddr,
        name: &str,
    ) -> Result<(ChatClient, ServerMsg), ProtocolError> {
        let (reader, writer) = TcpStream::connect(addr).await?.into_split();
        let mut client = ChatClient { reader, writer };
        client.send(&ClientMsg::Join { name: name.into() }).await?;
        let first = client.next().await?.ok_or(ProtocolError::Truncated)?;
        Ok((client, first))
    }

    pub async fn send(&mut self, msg: &ClientMsg) -> io::Result<()> {
        write_frame(&mut self.writer, &msg.encode()).await
    }

    pub async fn say(&mut self, text: &str) -> io::Result<()> {
        self.send(&ClientMsg::Say { text: text.into() }).await
    }

    /// The next message from the server (`None` when it closed).
    pub async fn next(&mut self) -> Result<Option<ServerMsg>, ProtocolError> {
        match read_frame(&mut self.reader).await? {
            Some(frame) => Ok(Some(ServerMsg::decode(&frame)?)),
            None => Ok(None),
        }
    }
}
