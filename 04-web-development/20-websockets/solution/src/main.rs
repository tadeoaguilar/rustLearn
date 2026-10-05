// Reference solution for 20-websockets.
//
//     cargo run -p m20-websockets-solution -- demo    # clients exercise every endpoint
//     cargo run -p m20-websockets-solution -- serve   # ws://127.0.0.1:3000 -- try it with websocat

use m20_websockets_solution::client::{Backoff, WsClient, collect_with_reconnects};
use m20_websockets_solution::protocol::{ClientMessage, ServerMessage};
use m20_websockets_solution::server::router;
use m20_websockets_solution::state::{AppState, HeartbeatConfig};
use std::time::Duration;

const WAIT: Duration = Duration::from_secs(2);

async fn demo() {
    let addr =
        m20_websockets_solution::spawn(router(AppState::new(HeartbeatConfig::default()))).await;
    let ws = |path: &str| format!("ws://{addr}{path}");

    let mut echo = WsClient::connect(&ws("/ws/echo")).await.unwrap();
    echo.send_text("hello").await.unwrap();
    println!("echo -> {:?}", echo.recv_text(WAIT).await);

    let mut alice = WsClient::connect(&ws("/ws/chat?name=alice")).await.unwrap();
    let mut bob = WsClient::connect(&ws("/ws/chat?name=bob")).await.unwrap();
    println!("alice <- {:?}", alice.recv(WAIT).await);
    bob.recv(WAIT).await;
    // Messages from *different* connections are handled concurrently, so
    // wait for each step's confirmation before the next client acts.
    alice
        .send(&ClientMessage::Join {
            room: "rust".into(),
        })
        .await
        .unwrap();
    alice
        .recv_until(WAIT, |m| matches!(m, ServerMessage::History { .. }))
        .await;
    alice
        .send(&ClientMessage::Say {
            text: "anyone here?".into(),
        })
        .await
        .unwrap();
    alice
        .recv_until(WAIT, |m| matches!(m, ServerMessage::Message { .. }))
        .await; // our own message, back from the room
    bob.send(&ClientMessage::Join {
        room: "rust".into(),
    })
    .await
    .unwrap();
    println!(
        "bob   <- {:?}",
        bob.recv_until(WAIT, |m| matches!(m, ServerMessage::History { .. }))
            .await
    );
    println!(
        "alice <- {:?}",
        alice
            .recv_until(
                WAIT,
                |m| matches!(m, ServerMessage::Joined { name, .. } if name == "bob")
            )
            .await
    );
    bob.send(&ClientMessage::Say {
        text: "hi alice".into(),
    })
    .await
    .unwrap();
    println!(
        "alice <- {:?}",
        alice
            .recv_until(WAIT, |m| matches!(m, ServerMessage::Message { .. }))
            .await
    );
    alice.send_text("{not json").await.unwrap();
    println!("alice <- {:?}", alice.recv(WAIT).await);

    let mut inbox = WsClient::connect(&ws("/ws/notifications?user=bob"))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await; // let the subscription register
    let r: serde_json::Value = reqwest::Client::new()
        .post(format!("http://{addr}/notify/bob"))
        .json(&serde_json::json!({"title": "Build finished", "body": "all green"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    println!(
        "POST /notify/bob -> {r}; inbox <- {:?}",
        inbox.recv(WAIT).await
    );

    let mut ed1 = WsClient::connect(&ws("/ws/doc?name=ed1")).await.unwrap();
    let mut ed2 = WsClient::connect(&ws("/ws/doc?name=ed2")).await.unwrap();
    ed1.recv(WAIT).await;
    ed2.recv(WAIT).await;
    ed1.send(&ClientMessage::Edit {
        base_version: 0,
        text: "Hello".into(),
    })
    .await
    .unwrap();
    println!("ed2 <- {:?}", ed2.recv(WAIT).await);
    ed2.send(&ClientMessage::Edit {
        base_version: 0,
        text: "stale edit".into(),
    })
    .await
    .unwrap();
    println!("ed2 <- {:?}", ed2.recv(WAIT).await);

    let mut dash = WsClient::connect(&ws("/ws/dashboard?every_ms=100"))
        .await
        .unwrap();
    for _ in 0..3 {
        println!("dashboard <- {:?}", dash.recv(WAIT).await);
    }

    let got = collect_with_reconnects(
        &ws("/ws/flaky"),
        3,
        Backoff {
            base: Duration::from_millis(50),
            max: Duration::from_secs(1),
        },
    )
    .await;
    println!("reconnecting client collected {got:?}");
}

#[tokio::main]
async fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("demo") | Some("all") => demo().await,
        Some("serve") => {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
                .await
                .unwrap();
            println!(
                "WebSocket server on ws://127.0.0.1:3000 -- see GETTING_STARTED.md for websocat commands"
            );
            axum::serve(listener, router(AppState::new(HeartbeatConfig::default())))
                .await
                .unwrap();
        }
        _ => {
            println!("20-websockets -- reference solution\n");
            println!(
                "  cargo run -p m20-websockets-solution -- demo    clients exercise every endpoint"
            );
            println!("  cargo run -p m20-websockets-solution -- serve   ws://127.0.0.1:3000");
        }
    }
}
