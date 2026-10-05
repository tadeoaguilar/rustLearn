use crate::sut::*;
use client::WsClient;
use protocol::{ChatLine, ClientMessage, ServerMessage};
use state::{AppState, HeartbeatConfig};
use std::time::Duration;

const WAIT: Duration = Duration::from_secs(2);

async fn server_with(heartbeat: HeartbeatConfig) -> String {
    crate::sut::spawn(server::router(AppState::new(heartbeat))).await
}

async fn server() -> String {
    server_with(HeartbeatConfig::default()).await
}

async fn connect(addr: &str, path: &str) -> WsClient {
    WsClient::connect(&format!("ws://{addr}{path}"))
        .await
        .expect("websocket handshake")
}

/// Connects to /ws/chat, consumes the Welcome, joins `room`, waits for History.
async fn chatter(addr: &str, name: &str, room: &str) -> (WsClient, Vec<ChatLine>) {
    let mut c = connect(addr, &format!("/ws/chat?name={name}")).await;
    assert_eq!(
        c.recv(WAIT).await,
        Some(ServerMessage::Welcome { name: name.into() })
    );
    c.send(&ClientMessage::Join { room: room.into() })
        .await
        .unwrap();
    match c
        .recv_until(WAIT, |m| matches!(m, ServerMessage::History { .. }))
        .await
    {
        Some(ServerMessage::History { lines, .. }) => (c, lines),
        other => panic!("expected history, got {other:?}"),
    }
}

// ---- Exercise 1: echo ---------------------------------------------------------------------

#[tokio::test]
async fn ex1_echo_text_and_binary() {
    let addr = server().await;
    let mut c = connect(&addr, "/ws/echo").await;
    c.send_text("hello").await.unwrap();
    assert_eq!(c.recv_text(WAIT).await.as_deref(), Some("hello"));
    c.send_binary(vec![1, 2, 3]).await.unwrap();
    match c.next_frame(WAIT).await {
        Some(tokio_tungstenite::tungstenite::Message::Binary(b)) => assert_eq!(&b[..], &[1, 2, 3]),
        other => panic!("expected binary echo, got {other:?}"),
    }
}

// ---- Exercise 2: protocol -------------------------------------------------------------------

#[test]
fn ex2_wire_format() {
    let msg: ClientMessage = serde_json::from_str(r#"{"type":"join","room":"rust"}"#).unwrap();
    assert_eq!(
        msg,
        ClientMessage::Join {
            room: "rust".into()
        }
    );
    let edit: ClientMessage =
        serde_json::from_str(r#"{"type":"edit","base_version":3,"text":"x"}"#).unwrap();
    assert_eq!(
        edit,
        ClientMessage::Edit {
            base_version: 3,
            text: "x".into()
        }
    );
    let out = ServerMessage::Message {
        room: "r".into(),
        from: "a".into(),
        text: "t".into(),
    }
    .to_json();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&out).unwrap(),
        serde_json::json!({"type":"message","room":"r","from":"a","text":"t"})
    );
    assert!(matches!(
        protocol::parse_client(r#"{"type":"dance"}"#),
        Err(ServerMessage::Error { .. })
    ));
}

#[tokio::test]
async fn ex2_bad_frames_get_error_replies_not_disconnects() {
    let addr = server().await;
    let mut c = connect(&addr, "/ws/chat?name=x").await;
    c.recv(WAIT).await; // welcome
    c.send_text("{not json").await.unwrap();
    assert!(matches!(
        c.recv(WAIT).await,
        Some(ServerMessage::Error { .. })
    ));
    c.send_binary(vec![0]).await.unwrap();
    assert!(matches!(
        c.recv(WAIT).await,
        Some(ServerMessage::Error { .. })
    ));
    c.send(&ClientMessage::Ping).await.unwrap();
    assert_eq!(
        c.recv(WAIT).await,
        Some(ServerMessage::Pong),
        "still connected"
    );
}

#[tokio::test]
async fn ex2_name_is_validated_before_upgrading() {
    let addr = server().await;
    let err = WsClient::connect(&format!("ws://{addr}/ws/chat"))
        .await
        .err()
        .expect("handshake must fail");
    match err {
        tokio_tungstenite::tungstenite::Error::Http(resp) => assert_eq!(resp.status(), 400),
        other => panic!("expected an HTTP 400, got {other:?}"),
    }
}

// ---- Exercise 3: chat rooms ------------------------------------------------------------------

#[tokio::test]
async fn ex3_broadcast_history_and_presence() {
    let addr = server().await;
    let (mut alice, history) = chatter(&addr, "alice", "rust").await;
    assert!(history.is_empty());
    alice
        .send(&ClientMessage::Say {
            text: "first!".into(),
        })
        .await
        .unwrap();
    assert_eq!(
        alice
            .recv_until(WAIT, |m| matches!(m, ServerMessage::Message { .. }))
            .await,
        Some(ServerMessage::Message {
            room: "rust".into(),
            from: "alice".into(),
            text: "first!".into()
        }),
        "senders get their own message back"
    );

    let (mut bob, history) = chatter(&addr, "bob", "rust").await;
    assert_eq!(
        history,
        vec![ChatLine {
            from: "alice".into(),
            text: "first!".into()
        }]
    );
    assert_eq!(
        alice
            .recv_until(WAIT, |m| matches!(m, ServerMessage::Joined { .. }))
            .await,
        Some(ServerMessage::Joined {
            room: "rust".into(),
            name: "bob".into(),
            members: vec!["alice".into(), "bob".into()]
        })
    );
    bob.send(&ClientMessage::Say { text: "hi".into() })
        .await
        .unwrap();
    assert!(
        matches!(alice.recv_until(WAIT, |m| matches!(m, ServerMessage::Message { .. })).await, Some(ServerMessage::Message { from, .. }) if from == "bob")
    );

    bob.close().await; // a disconnect counts as leaving
    assert_eq!(
        alice
            .recv_until(WAIT, |m| matches!(m, ServerMessage::Left { .. }))
            .await,
        Some(ServerMessage::Left {
            room: "rust".into(),
            name: "bob".into()
        })
    );
}

#[tokio::test]
async fn ex3_rooms_are_isolated_and_switchable() {
    let addr = server().await;
    let (mut a, _) = chatter(&addr, "a", "one").await;
    let (mut b, _) = chatter(&addr, "b", "two").await;
    b.send(&ClientMessage::Say {
        text: "in two".into(),
    })
    .await
    .unwrap();
    assert!(
        a.recv_until(Duration::from_millis(300), |m| matches!(
            m,
            ServerMessage::Message { .. }
        ))
        .await
        .is_none(),
        "room one hears nothing from room two"
    );
    a.send(&ClientMessage::Join { room: "two".into() })
        .await
        .unwrap();
    match a
        .recv_until(WAIT, |m| matches!(m, ServerMessage::History { .. }))
        .await
    {
        Some(ServerMessage::History { room, lines }) => {
            assert_eq!(room, "two");
            assert_eq!(lines.len(), 1);
        }
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn ex3_validation_and_history_limit() {
    let addr = server().await;
    let mut c = connect(&addr, "/ws/chat?name=solo").await;
    c.recv(WAIT).await;
    c.send(&ClientMessage::Say {
        text: "hello?".into(),
    })
    .await
    .unwrap();
    assert!(
        matches!(c.recv(WAIT).await, Some(ServerMessage::Error { message }) if message.contains("join"))
    );
    let (mut c, _) = chatter(&addr, "talker", "big").await;
    c.send(&ClientMessage::Say { text: "   ".into() })
        .await
        .unwrap();
    assert!(
        c.recv_until(WAIT, |m| matches!(m, ServerMessage::Error { .. }))
            .await
            .is_some()
    );
    for i in 0..25 {
        c.send(&ClientMessage::Say {
            text: format!("m{i}"),
        })
        .await
        .unwrap();
    }
    for _ in 0..25 {
        c.recv_until(WAIT, |m| matches!(m, ServerMessage::Message { .. }))
            .await
            .unwrap();
    }
    let (_, history) = chatter(&addr, "late", "big").await;
    assert_eq!(history.len(), state::HISTORY);
    assert_eq!(
        history.first().unwrap().text,
        "m5",
        "only the most recent 20 are kept"
    );
}

// ---- Exercise 4: notifications ----------------------------------------------------------------

async fn notify(addr: &str, user: &str) -> u64 {
    let r = reqwest::Client::new()
        .post(format!("http://{addr}/notify/{user}"))
        .json(&serde_json::json!({"title": "Deploy", "body": "done"}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 202);
    r.json::<serde_json::Value>().await.unwrap()["delivered"]
        .as_u64()
        .unwrap()
}

#[tokio::test]
async fn ex4_http_pushes_to_every_tab_of_that_user_only() {
    let addr = server().await;
    assert_eq!(notify(&addr, "bob").await, 0, "nobody listening");
    let mut tab1 = connect(&addr, "/ws/notifications?user=bob").await;
    let mut tab2 = connect(&addr, "/ws/notifications?user=bob").await;
    let mut other = connect(&addr, "/ws/notifications?user=carol").await;
    tokio::time::sleep(Duration::from_millis(100)).await; // let the subscriptions register
    assert_eq!(notify(&addr, "bob").await, 2);
    let expected = Some(ServerMessage::Notification {
        title: "Deploy".into(),
        body: "done".into(),
    });
    assert_eq!(tab1.recv(WAIT).await, expected);
    assert_eq!(tab2.recv(WAIT).await, expected);
    assert_eq!(other.recv(Duration::from_millis(300)).await, None);
}

// ---- Exercise 5: heartbeats -------------------------------------------------------------------

fn fast_heartbeat() -> HeartbeatConfig {
    HeartbeatConfig {
        interval: Duration::from_millis(50),
        timeout: Duration::from_millis(200),
    }
}

#[tokio::test]
async fn ex5_a_client_that_answers_pings_stays_connected() {
    let addr = server_with(fast_heartbeat()).await;
    let mut c = connect(&addr, "/ws/chat?name=alive").await;
    c.recv(WAIT).await;
    // Reading lets tungstenite answer the server's pings automatically.
    assert_eq!(
        c.recv(Duration::from_millis(600)).await,
        None,
        "nothing but pings for 600ms"
    );
    c.send(&ClientMessage::Ping).await.unwrap();
    assert_eq!(
        c.recv(WAIT).await,
        Some(ServerMessage::Pong),
        "still connected after 3x the timeout"
    );
}

#[tokio::test]
async fn ex5_a_silent_client_is_disconnected() {
    let addr = server_with(fast_heartbeat()).await;
    let mut c = connect(&addr, "/ws/chat?name=ghost").await;
    c.recv(WAIT).await;
    // Pings are only answered while the client *reads*. Not reading for 600ms
    // looks exactly like a dead connection to the server.
    tokio::time::sleep(Duration::from_millis(600)).await;
    assert!(
        c.is_closed_within(WAIT).await,
        "the server should have closed the connection"
    );
}

// ---- Exercise 6: collaborative document -------------------------------------------------------

#[tokio::test]
async fn ex6_versions_updates_and_conflicts() {
    let addr = server().await;
    let mut ed1 = connect(&addr, "/ws/doc?name=ed1").await;
    let mut ed2 = connect(&addr, "/ws/doc?name=ed2").await;
    assert_eq!(
        ed1.recv(WAIT).await,
        Some(ServerMessage::Document {
            version: 0,
            text: String::new()
        })
    );
    ed2.recv(WAIT).await;

    ed1.send(&ClientMessage::Edit {
        base_version: 0,
        text: "Hello".into(),
    })
    .await
    .unwrap();
    let update = Some(ServerMessage::Updated {
        version: 1,
        text: "Hello".into(),
        by: "ed1".into(),
    });
    assert_eq!(
        ed1.recv(WAIT).await,
        update,
        "the editor sees its own accepted edit"
    );
    assert_eq!(ed2.recv(WAIT).await, update);

    ed2.send(&ClientMessage::Edit {
        base_version: 0,
        text: "stale".into(),
    })
    .await
    .unwrap();
    assert_eq!(
        ed2.recv(WAIT).await,
        Some(ServerMessage::Conflict {
            your_base: 0,
            current_version: 1,
            text: "Hello".into()
        })
    );
    assert_eq!(
        ed1.recv(Duration::from_millis(300)).await,
        None,
        "a rejected edit isn't broadcast"
    );

    ed2.send(&ClientMessage::Edit {
        base_version: 1,
        text: "Hello, world".into(),
    })
    .await
    .unwrap();
    assert!(matches!(
        ed1.recv(WAIT).await,
        Some(ServerMessage::Updated { version: 2, .. })
    ));

    let mut late = connect(&addr, "/ws/doc?name=late").await;
    assert_eq!(
        late.recv(WAIT).await,
        Some(ServerMessage::Document {
            version: 2,
            text: "Hello, world".into()
        })
    );
}

// ---- Exercise 7: dashboard ---------------------------------------------------------------------

#[tokio::test]
async fn ex7_dashboard_pushes_metrics_periodically() {
    let addr = server().await;
    let _a = connect(&addr, "/ws/echo").await;
    let _b = connect(&addr, "/ws/echo").await;
    let mut dash = connect(&addr, "/ws/dashboard?every_ms=50").await;
    let mut uptimes = Vec::new();
    for _ in 0..3 {
        match dash.recv(WAIT).await {
            Some(ServerMessage::Metrics {
                connections,
                uptime_ms,
                ..
            }) => {
                assert_eq!(connections, 3, "two echo clients + the dashboard");
                uptimes.push(uptime_ms);
            }
            other => panic!("expected metrics, got {other:?}"),
        }
    }
    assert!(uptimes.windows(2).all(|w| w[1] > w[0]), "{uptimes:?}");
}

// ---- Bonus: reconnecting client -----------------------------------------------------------------

#[test]
fn bonus_backoff_doubles_and_caps() {
    let b = client::Backoff {
        base: Duration::from_millis(100),
        max: Duration::from_secs(1),
    };
    let delays: Vec<u128> = (0..6).map(|a| b.delay(a).as_millis()).collect();
    assert_eq!(delays, vec![100, 200, 400, 800, 1000, 1000]);
    assert_eq!(b.delay(1_000), Duration::from_secs(1), "no overflow");
}

#[tokio::test]
async fn bonus_client_reconnects_after_the_server_hangs_up() {
    let addr = server().await;
    let backoff = client::Backoff {
        base: Duration::from_millis(20),
        max: Duration::from_millis(200),
    };
    let got = client::collect_with_reconnects(&format!("ws://{addr}/ws/flaky"), 3, backoff).await;
    assert_eq!(got, vec!["hello #1", "hello #2", "hello #3"]);
}
