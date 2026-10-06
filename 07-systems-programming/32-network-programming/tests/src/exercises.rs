use std::future::Future;
use std::io::{BufRead, BufReader, Write};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use crate::sut::bonus_dns::{self as dns, DnsError};
use crate::sut::ex02_http::{self as http, HttpError, Request, Response};
use crate::sut::ex03_proxy::{self as proxy, Balancer, Stats};
use crate::sut::ex04_chat::{self as chat, ChatClient, ClientMsg, ProtocolError, Room, ServerMsg};
use crate::sut::{ex01_echo as echo, ex05_udp as udp};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tokio::net::{TcpListener, TcpStream, UdpSocket};

/// Every network test runs under a watchdog: a hang is a failure, not a stuck test run.
async fn deadline<T>(future: impl Future<Output = T>) -> T {
    tokio::time::timeout(Duration::from_secs(10), future)
        .await
        .expect("timed out")
}

async fn local() -> (TcpListener, SocketAddr) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    (listener, addr)
}

// ---------------------------------------------------------------- Exercise 1

#[test]
fn ex1_reply() {
    assert_eq!(echo::reply("hi"), Some("echo: hi\n".into()));
    assert_eq!(echo::reply("hi\r\n"), Some("echo: hi\n".into()));
    assert_eq!(echo::reply(""), Some("echo: \n".into()));
    assert_eq!(echo::reply("bye"), None);
    assert_eq!(echo::reply("bye\r\n"), None);
}

#[test]
fn ex1_blocking_server() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let served = Arc::new(AtomicUsize::new(0));
    let counter = served.clone();
    std::thread::spawn(move || echo::serve_blocking(listener, counter));
    let mut clients = Vec::new();
    for i in 0..5 {
        clients.push(std::thread::spawn(move || {
            let mut stream = std::net::TcpStream::connect(addr).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            writeln!(stream, "client {i}").unwrap();
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            writeln!(stream, "bye").unwrap();
            line
        }));
    }
    for (i, c) in clients.into_iter().enumerate() {
        assert_eq!(c.join().unwrap(), format!("echo: client {i}\n"));
    }
    let start = std::time::Instant::now();
    while served.load(Ordering::SeqCst) < 5 && start.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        served.load(Ordering::SeqCst),
        5,
        "each client finished after bye"
    );
}

#[tokio::test]
async fn ex1_async_server_handles_many_clients() {
    deadline(async {
        let (listener, addr) = local().await;
        tokio::spawn(echo::serve_async(listener));
        assert_eq!(
            echo::echo_client(addr, &["a", "b"]).await.unwrap(),
            ["echo: a", "echo: b"]
        );
        let clients: Vec<_> = (0..200)
            .map(|i| tokio::spawn(async move { echo::echo_client(addr, &[&i.to_string()]).await }))
            .collect();
        for (i, c) in clients.into_iter().enumerate() {
            assert_eq!(c.await.unwrap().unwrap(), [format!("echo: {i}")]);
        }
        // `bye` closes the connection from the server's side
        let mut stream = TcpStream::connect(addr).await.unwrap();
        stream.write_all(b"bye\n").await.unwrap();
        let mut rest = Vec::new();
        stream.read_to_end(&mut rest).await.unwrap();
        assert!(rest.is_empty());
    })
    .await;
}

// ---------------------------------------------------------------- Exercise 2

#[test]
fn ex2_percent_decoding_and_targets() {
    assert_eq!(
        http::percent_decode("a%20b%21", false).as_deref(),
        Some("a b!")
    );
    assert_eq!(http::percent_decode("a+b", true).as_deref(), Some("a b"));
    assert_eq!(http::percent_decode("a+b", false).as_deref(), Some("a+b"));
    assert_eq!(
        http::percent_decode("%e2%9c%93", false).as_deref(),
        Some("✓")
    );
    assert_eq!(http::percent_decode("%zz", false), None);
    assert_eq!(http::percent_decode("%4", false), None);
    assert_eq!(http::percent_decode("%ff", false), None, "not UTF-8");
    let (path, query) = http::split_target("/a%20b?x=1&y=two+words&flag").unwrap();
    assert_eq!(path, "/a b");
    assert_eq!(
        query,
        [
            ("x".into(), "1".into()),
            ("y".into(), "two words".into()),
            ("flag".into(), String::new())
        ]
    );
    assert!(matches!(
        http::split_target("no-slash"),
        Err(HttpError::Malformed(_))
    ));
}

#[test]
fn ex2_parse_head() {
    let req =
        http::parse_head("GET /echo?msg=hi HTTP/1.1\r\nHost: example.com\r\nX-Thing:  spaced  ")
            .unwrap();
    assert_eq!(
        (req.method.as_str(), req.path.as_str(), req.version.as_str()),
        ("GET", "/echo", "HTTP/1.1")
    );
    assert_eq!(req.query_param("msg"), Some("hi"));
    assert_eq!(req.header("host"), Some("example.com"), "case-insensitive");
    assert_eq!(req.header("X-THING"), Some("spaced"), "trimmed");
    for bad in [
        "GET / HTTP/2.0",
        "GET /",
        "get / HTTP/1.1",
        "GET / HTTP/1.1\r\nno colon",
        "GET / HTTP/1.1\r\nBad Name: x",
    ] {
        assert!(
            matches!(http::parse_head(bad), Err(HttpError::Malformed(_))),
            "{bad:?}"
        );
    }
}

#[test]
fn ex2_keep_alive_rules() {
    let req = |version: &str, connection: Option<&str>| Request {
        version: version.into(),
        headers: connection
            .map(|c| vec![("Connection".to_string(), c.to_string())])
            .unwrap_or_default(),
        ..Default::default()
    };
    assert!(req("HTTP/1.1", None).keep_alive());
    assert!(!req("HTTP/1.1", Some("close")).keep_alive());
    assert!(!req("HTTP/1.1", Some("Close")).keep_alive());
    assert!(!req("HTTP/1.0", None).keep_alive());
    assert!(req("HTTP/1.0", Some("keep-alive")).keep_alive());
}

#[test]
fn ex2_response_bytes() {
    let bytes = Response::text(200, "hi").to_bytes(true);
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(text.contains("Content-Type: text/plain; charset=utf-8\r\n"));
    assert!(text.contains("Content-Length: 2\r\n") && text.contains("Connection: keep-alive\r\n"));
    assert!(text.ends_with("\r\n\r\nhi"));
    assert!(
        String::from_utf8(Response::new(404).to_bytes(false))
            .unwrap()
            .contains("Connection: close")
    );
}

#[tokio::test]
async fn ex2_read_request() {
    let raw = b"POST /upper HTTP/1.1\r\nContent-Length: 5\r\n\r\nhello";
    let req = http::read_request(&mut AsyncBufReader::new(&raw[..]))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        (req.method.as_str(), req.body.as_slice()),
        ("POST", &b"hello"[..])
    );
    let bare_lf = b"GET / HTTP/1.1\nHost: x\n\n";
    assert_eq!(
        http::read_request(&mut AsyncBufReader::new(&bare_lf[..]))
            .await
            .unwrap()
            .unwrap()
            .header("host"),
        Some("x")
    );
    assert!(
        http::read_request(&mut AsyncBufReader::new(&b""[..]))
            .await
            .unwrap()
            .is_none(),
        "clean EOF"
    );
    let cut = b"GET / HTTP/1.1\r\nHost: x\r\n";
    assert!(matches!(
        http::read_request(&mut AsyncBufReader::new(&cut[..])).await,
        Err(HttpError::Malformed(_))
    ));
    let huge_header = format!(
        "GET / HTTP/1.1\r\nX: {}\r\n\r\n",
        "a".repeat(http::MAX_HEAD)
    );
    assert!(matches!(
        http::read_request(&mut AsyncBufReader::new(huge_header.as_bytes())).await,
        Err(HttpError::HeadTooLarge)
    ));
    let huge_body = format!(
        "POST / HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
        http::MAX_BODY + 1
    );
    assert!(matches!(
        http::read_request(&mut AsyncBufReader::new(huge_body.as_bytes())).await,
        Err(HttpError::BodyTooLarge)
    ));
    let bad_length = b"POST / HTTP/1.1\r\nContent-Length: lots\r\n\r\n";
    assert!(matches!(
        http::read_request(&mut AsyncBufReader::new(&bad_length[..])).await,
        Err(HttpError::Malformed(_))
    ));
}

#[test]
fn ex2_routing() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("page.html"), "<p>hi</p>").unwrap();
    let get = |target: &str| {
        let (path, query) = http::split_target(target).unwrap();
        http::route(
            &Request {
                method: "GET".into(),
                path,
                query,
                version: "HTTP/1.1".into(),
                ..Default::default()
            },
            dir.path(),
        )
    };
    assert_eq!(
        (get("/").status, get("/").body),
        (200, b"Hello, HTTP!\n".to_vec())
    );
    assert_eq!(get("/echo?msg=a+b").body, b"a b");
    let page = get("/static/page.html");
    assert_eq!(
        (page.status, page.body.as_slice()),
        (200, &b"<p>hi</p>"[..])
    );
    assert!(
        page.headers
            .contains(&("Content-Type".into(), "text/html; charset=utf-8".into()))
    );
    assert_eq!(get("/static/missing.html").status, 404);
    assert_eq!(get("/static/../secret").status, 403);
    assert_eq!(
        get("/static/%2e%2e/secret").status,
        403,
        "decoded before checking"
    );
    assert_eq!(get("/nowhere").status, 404);
    let post = |path: &str, body: &[u8]| {
        http::route(
            &Request {
                method: "POST".into(),
                path: path.into(),
                version: "HTTP/1.1".into(),
                body: body.to_vec(),
                ..Default::default()
            },
            dir.path(),
        )
    };
    assert_eq!(post("/upper", b"shout").body, b"SHOUT");
    assert_eq!(post("/upper", &[0xff]).status, 400);
    let wrong = post("/", b"");
    assert_eq!(wrong.status, 405);
    assert!(wrong.headers.contains(&("Allow".into(), "GET".into())));
    assert_eq!(
        http::content_type(std::path::Path::new("x.css")),
        "text/css"
    );
    assert_eq!(
        http::content_type(std::path::Path::new("x.bin")),
        "application/octet-stream"
    );
    assert!(
        http::static_file(dir.path(), "/etc/passwd").is_none(),
        "absolute paths too"
    );
}

async fn http_server() -> (SocketAddr, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.txt"), "file contents").unwrap();
    let (listener, addr) = local().await;
    tokio::spawn(http::serve(listener, dir.path().to_path_buf()));
    (addr, dir)
}

async fn raw(addr: SocketAddr, request: &[u8]) -> String {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream.write_all(request).await.unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();
    String::from_utf8_lossy(&response).into_owned()
}

#[tokio::test]
async fn ex2_server_keep_alive_and_errors() {
    deadline(async {
        let (addr, _dir) = http_server().await;
        let two = raw(
            addr,
            b"GET / HTTP/1.1\r\n\r\nGET /echo?msg=x HTTP/1.1\r\nConnection: close\r\n\r\n",
        )
        .await;
        assert_eq!(two.matches("HTTP/1.1 200 OK").count(), 2, "{two}");
        assert!(two.contains("Connection: keep-alive") && two.trim_end().ends_with('x'));
        let bad = raw(addr, b"BROKEN\r\n\r\n").await;
        assert!(bad.starts_with("HTTP/1.1 400 Bad Request"), "{bad}");
        let big = raw(
            addr,
            format!("GET / HTTP/1.1\r\nX: {}\r\n\r\n", "a".repeat(9000)).as_bytes(),
        )
        .await;
        assert!(big.starts_with("HTTP/1.1 431"), "{big}");
        let http10 = raw(addr, b"GET / HTTP/1.0\r\n\r\n").await;
        assert!(
            http10.contains("Connection: close"),
            "HTTP/1.0 closes by default"
        );
    })
    .await;
}

#[tokio::test]
async fn ex2_a_real_http_client_agrees() {
    deadline(async {
        let (addr, _dir) = http_server().await;
        let client = reqwest::Client::new();
        let res = client
            .get(format!("http://{addr}/echo?msg=caf%C3%A9"))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(res.text().await.unwrap(), "café");
        let res = client
            .post(format!("http://{addr}/upper"))
            .body("from reqwest")
            .send()
            .await
            .unwrap();
        assert_eq!(res.text().await.unwrap(), "FROM REQWEST");
        let res = client
            .get(format!("http://{addr}/static/a.txt"))
            .send()
            .await
            .unwrap();
        assert_eq!(res.text().await.unwrap(), "file contents");
        assert_eq!(
            client
                .get(format!("http://{addr}/missing"))
                .send()
                .await
                .unwrap()
                .status(),
            404
        );
    })
    .await;
}

// ---------------------------------------------------------------- Exercise 3

async fn echo_upstream() -> SocketAddr {
    let (listener, addr) = local().await;
    tokio::spawn(echo::serve_async(listener));
    addr
}

#[tokio::test]
async fn ex3_proxy_forwards_and_counts() {
    deadline(async {
        let upstream = echo_upstream().await;
        let stats = Arc::new(Stats::default());
        let (listener, addr) = local().await;
        tokio::spawn(proxy::serve_proxy(listener, upstream, stats.clone()));
        assert_eq!(
            echo::echo_client(addr, &["one", "two"]).await.unwrap(),
            ["echo: one", "echo: two"]
        );
        let start = tokio::time::Instant::now();
        while stats.bytes_down.load(Ordering::SeqCst) == 0
            && start.elapsed() < Duration::from_secs(5)
        {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        assert_eq!(stats.connections.load(Ordering::SeqCst), 1);
        assert_eq!(
            stats.bytes_up.load(Ordering::SeqCst),
            "one\ntwo\nbye\n".len() as u64
        );
        assert_eq!(
            stats.bytes_down.load(Ordering::SeqCst),
            "echo: one\necho: two\n".len() as u64
        );
    })
    .await;
}

#[test]
fn ex3_round_robin_skips_unhealthy() {
    let addrs: Vec<SocketAddr> = (1..=3)
        .map(|p| SocketAddr::from(([127, 0, 0, 1], p)))
        .collect();
    let balancer = Balancer::new(addrs.clone());
    let picks: Vec<SocketAddr> = (0..6).map(|_| balancer.pick().unwrap()).collect();
    assert_eq!(
        picks,
        [addrs[0], addrs[1], addrs[2], addrs[0], addrs[1], addrs[2]]
    );
    balancer.set_healthy(1, false);
    assert!(!balancer.is_healthy(1));
    assert!(
        (0..6)
            .map(|_| balancer.pick().unwrap())
            .all(|a| a != addrs[1])
    );
    balancer.set_healthy(0, false);
    balancer.set_healthy(2, false);
    assert_eq!(balancer.pick(), None);
}

#[tokio::test]
async fn ex3_health_checks_and_failover() {
    deadline(async {
        let up = echo_upstream().await;
        // a port with nothing listening: bind, note the address, close
        let down = {
            let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
            l.local_addr().unwrap()
        };
        let balancer = Arc::new(Balancer::new(vec![down, up]));
        balancer.check_all(Duration::from_millis(500)).await;
        assert!(!balancer.is_healthy(0) && balancer.is_healthy(1));

        // even if the dead one is believed healthy, a connection fails over
        balancer.set_healthy(0, true);
        let (listener, addr) = local().await;
        tokio::spawn(proxy::serve_balanced(
            listener,
            balancer.clone(),
            Arc::new(Stats::default()),
        ));
        for _ in 0..3 {
            assert_eq!(
                echo::echo_client(addr, &["hi"]).await.unwrap(),
                ["echo: hi"]
            );
        }
        assert!(!balancer.is_healthy(0), "marked down after refusing");
    })
    .await;
}

// ---------------------------------------------------------------- Exercise 4

#[test]
fn ex4_encoding() {
    assert_eq!(
        ClientMsg::Join { name: "ab".into() }.encode(),
        [1, 0, 2, b'a', b'b']
    );
    assert_eq!(ClientMsg::Leave.encode(), [3]);
    for msg in [
        ClientMsg::Join { name: "ana".into() },
        ClientMsg::Say {
            text: "héllo".into(),
        },
        ClientMsg::Leave,
    ] {
        assert_eq!(ClientMsg::decode(&msg.encode()).unwrap(), msg);
    }
    for msg in [
        ServerMsg::Welcome {
            users: vec!["a".into(), "bb".into()],
        },
        ServerMsg::Welcome { users: vec![] },
        ServerMsg::Joined { name: "x".into() },
        ServerMsg::Left { name: "y".into() },
        ServerMsg::Message {
            from: "a".into(),
            text: "".into(),
        },
        ServerMsg::Error {
            reason: "no".into(),
        },
    ] {
        assert_eq!(ServerMsg::decode(&msg.encode()).unwrap(), msg);
    }
    assert!(matches!(
        ClientMsg::decode(&[9]),
        Err(ProtocolError::UnknownTag(9))
    ));
    assert!(matches!(
        ClientMsg::decode(&[2, 0, 5, b'a']),
        Err(ProtocolError::Truncated)
    ));
    assert!(matches!(
        ClientMsg::decode(&[]),
        Err(ProtocolError::Truncated)
    ));
    assert!(matches!(
        ClientMsg::decode(&[2, 0, 1, 0xff]),
        Err(ProtocolError::BadUtf8)
    ));
}

#[tokio::test]
async fn ex4_frames() {
    let mut buf = Vec::new();
    chat::write_frame(&mut buf, b"abc").await.unwrap();
    chat::write_frame(&mut buf, b"").await.unwrap();
    assert_eq!(buf, [0, 0, 0, 3, b'a', b'b', b'c', 0, 0, 0, 0]);
    let mut reader = &buf[..];
    assert_eq!(
        chat::read_frame(&mut reader).await.unwrap(),
        Some(b"abc".to_vec())
    );
    assert_eq!(chat::read_frame(&mut reader).await.unwrap(), Some(vec![]));
    assert_eq!(
        chat::read_frame(&mut reader).await.unwrap(),
        None,
        "clean end"
    );
    let too_big = (chat::MAX_FRAME as u32 + 1).to_be_bytes();
    assert!(matches!(
        chat::read_frame(&mut &too_big[..]).await,
        Err(ProtocolError::TooLarge(_))
    ));
    assert!(matches!(
        chat::read_frame(&mut &[0u8, 0, 0, 5, 1][..]).await,
        Err(ProtocolError::Truncated)
    ));
    assert!(
        chat::valid_name("ana_2")
            && !chat::valid_name("")
            && !chat::valid_name("a b")
            && !chat::valid_name(&"x".repeat(17))
    );
}

async fn chat_server() -> SocketAddr {
    let (listener, addr) = local().await;
    tokio::spawn(chat::serve_chat(listener, Arc::new(Room::new())));
    addr
}

#[tokio::test]
async fn ex4_chat_room() {
    deadline(async {
        let addr = chat_server().await;
        let (mut ana, welcome) = ChatClient::join(addr, "ana").await.unwrap();
        assert_eq!(
            welcome,
            ServerMsg::Welcome {
                users: vec!["ana".into()]
            }
        );
        let (mut ben, welcome) = ChatClient::join(addr, "ben").await.unwrap();
        assert_eq!(
            welcome,
            ServerMsg::Welcome {
                users: vec!["ana".into(), "ben".into()]
            }
        );
        assert_eq!(
            ana.next().await.unwrap(),
            Some(ServerMsg::Joined { name: "ben".into() })
        );

        ben.say("hello").await.unwrap();
        let expected = ServerMsg::Message {
            from: "ben".into(),
            text: "hello".into(),
        };
        assert_eq!(ana.next().await.unwrap(), Some(expected.clone()));
        assert_eq!(
            ben.next().await.unwrap(),
            Some(expected),
            "senders see their own messages"
        );

        ben.send(&ClientMsg::Leave).await.unwrap();
        assert_eq!(
            ana.next().await.unwrap(),
            Some(ServerMsg::Left { name: "ben".into() })
        );
        assert_eq!(
            ben.next().await.unwrap(),
            None,
            "the server closed ben's connection"
        );
        // the name is free again
        let (_carl, welcome) = ChatClient::join(addr, "ben").await.unwrap();
        assert!(matches!(welcome, ServerMsg::Welcome { .. }));
    })
    .await;
}

#[tokio::test]
async fn ex4_joining_rules() {
    deadline(async {
        let addr = chat_server().await;
        let (_ana, _) = ChatClient::join(addr, "ana").await.unwrap();
        let (_, taken) = ChatClient::join(addr, "ana").await.unwrap();
        assert_eq!(
            taken,
            ServerMsg::Error {
                reason: "name taken".into()
            }
        );
        let (_, invalid) = ChatClient::join(addr, "no spaces").await.unwrap();
        assert_eq!(
            invalid,
            ServerMsg::Error {
                reason: "invalid name".into()
            }
        );
        // saying something before joining
        let mut stream = TcpStream::connect(addr).await.unwrap();
        chat::write_frame(&mut stream, &ClientMsg::Say { text: "hi".into() }.encode())
            .await
            .unwrap();
        let frame = chat::read_frame(&mut stream).await.unwrap().unwrap();
        assert_eq!(
            ServerMsg::decode(&frame).unwrap(),
            ServerMsg::Error {
                reason: "join first".into()
            }
        );
        // a disconnect counts as leaving
        let (mut watcher, _) = ChatClient::join(addr, "watcher").await.unwrap();
        let (dropper, _) = ChatClient::join(addr, "dropper").await.unwrap();
        assert_eq!(
            watcher.next().await.unwrap(),
            Some(ServerMsg::Joined {
                name: "dropper".into()
            })
        );
        drop(dropper);
        assert_eq!(
            watcher.next().await.unwrap(),
            Some(ServerMsg::Left {
                name: "dropper".into()
            })
        );
    })
    .await;
}

// ---------------------------------------------------------------- Exercise 5

#[test]
fn ex5_packets() {
    let ping = udp::Ping {
        seq: 0x0102,
        sent_nanos: 0x0304050607080910,
    };
    assert_eq!(
        ping.encode(),
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 0x10],
        "big-endian"
    );
    assert_eq!(udp::Ping::decode(&ping.encode()), Some(ping));
    assert_eq!(udp::Ping::decode(&[1, 2, 3]), None);
    assert_eq!(udp::Ping::decode(&[0; 11]), None);
}

#[test]
fn ex5_stats() {
    let ms = Duration::from_millis;
    let stats = udp::PingStats {
        sent: 4,
        received: 3,
        rtts: vec![ms(2), ms(4), ms(9)],
    };
    assert_eq!(stats.loss_percent(), 25.0);
    assert_eq!(
        (stats.min(), stats.avg(), stats.max()),
        (Some(ms(2)), Some(ms(5)), Some(ms(9)))
    );
    let empty = udp::PingStats::default();
    assert_eq!((empty.loss_percent(), empty.avg()), (0.0, None));
}

async fn pong_server(drop_every: Option<u32>) -> SocketAddr {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let addr = socket.local_addr().unwrap();
    tokio::spawn(udp::serve_pong(socket, drop_every));
    addr
}

#[tokio::test]
async fn ex5_ping() {
    deadline(async {
        let addr = pong_server(None).await;
        let stats = udp::ping(
            addr,
            10,
            Duration::from_millis(1),
            Duration::from_millis(500),
        )
        .await
        .unwrap();
        assert_eq!((stats.sent, stats.received, stats.rtts.len()), (10, 10, 10));
        assert!(stats.max().unwrap() < Duration::from_millis(500));
        let lossy = pong_server(Some(3)).await;
        let stats = udp::ping(
            lossy,
            9,
            Duration::from_millis(1),
            Duration::from_millis(100),
        )
        .await
        .unwrap();
        assert_eq!(
            (stats.sent, stats.received),
            (9, 6),
            "every 3rd pong dropped"
        );
        assert!((stats.loss_percent() - 100.0 / 3.0).abs() < 0.01);
    })
    .await;
}

#[tokio::test]
async fn ex5_garbage_is_ignored() {
    deadline(async {
        let addr = pong_server(None).await;
        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        client.send_to(b"garbage, 17 bytes", addr).await.unwrap();
        let mut buf = [0u8; 64];
        let reply =
            tokio::time::timeout(Duration::from_millis(200), client.recv_from(&mut buf)).await;
        assert!(reply.is_err(), "no reply to garbage");
        client
            .send_to(
                &udp::Ping {
                    seq: 7,
                    sent_nanos: 1,
                }
                .encode(),
                addr,
            )
            .await
            .unwrap();
        let (n, _) = client.recv_from(&mut buf).await.unwrap();
        assert_eq!(udp::Ping::decode(&buf[..n]).unwrap().seq, 7);
    })
    .await;
}

// --------------------------------------------------------------------- Bonus

#[test]
fn bonus_names_and_queries() {
    assert_eq!(
        dns::encode_name("www.example.com").unwrap(),
        b"\x03www\x07example\x03com\x00"
    );
    assert_eq!(
        dns::encode_name("example.com.").unwrap(),
        dns::encode_name("example.com").unwrap(),
        "trailing dot"
    );
    assert_eq!(dns::encode_name("a..b"), Err(DnsError::BadLabel));
    assert_eq!(dns::encode_name(&"x".repeat(64)), Err(DnsError::BadLabel));
    assert_eq!(
        dns::encode_name(&vec!["abcdefgh"; 30].join(".")),
        Err(DnsError::NameTooLong)
    );
    let q = dns::build_query(0x1234, "a.io").unwrap();
    assert_eq!(&q[..12], [0x12, 0x34, 0x01, 0x00, 0, 1, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&q[12..], b"\x01a\x02io\x00\x00\x01\x00\x01");
}

#[test]
fn bonus_parse_response_with_compression() {
    let mut packet = dns::build_query(7, "example.com").unwrap();
    packet[2..4].copy_from_slice(&0x8180u16.to_be_bytes());
    packet[6..8].copy_from_slice(&2u16.to_be_bytes());
    // two A records, both naming the question (offset 12) by pointer
    packet.extend_from_slice(&[0xC0, 12, 0, 1, 0, 1, 0, 0, 0, 60, 0, 4, 1, 2, 3, 4]);
    packet.extend_from_slice(&[0xC0, 12, 0, 1, 0, 1, 0, 1, 0, 0, 0, 4, 5, 6, 7, 8]);
    let response = dns::parse_response(&packet).unwrap();
    assert_eq!((response.id, response.rcode), (7, 0));
    assert_eq!(response.answers[0].name, "example.com");
    assert_eq!(
        (response.answers[0].ttl, response.answers[1].ttl),
        (60, 65536)
    );
    assert_eq!(
        response.a_records(),
        [
            std::net::Ipv4Addr::new(1, 2, 3, 4),
            std::net::Ipv4Addr::new(5, 6, 7, 8)
        ]
    );
    assert_eq!(
        dns::decode_name(&packet, 12).unwrap(),
        ("example.com".to_string(), 25)
    );
    assert_eq!(
        dns::decode_name(&packet, 29).unwrap(),
        ("example.com".to_string(), 31),
        "a pointer is two bytes"
    );
}

#[test]
fn bonus_malicious_packets() {
    // a pointer to itself, and one pointing forward
    let mut looping = vec![0u8; 12];
    looping.extend_from_slice(&[0xC0, 12]);
    assert_eq!(dns::decode_name(&looping, 12), Err(DnsError::BadPointer));
    let mut forward = vec![0u8; 12];
    forward.extend_from_slice(&[0xC0, 20]);
    assert_eq!(dns::decode_name(&forward, 12), Err(DnsError::BadPointer));
    assert_eq!(dns::decode_name(b"\x05ab", 0), Err(DnsError::Truncated));
    assert_eq!(dns::parse_response(&[0, 1, 2]), Err(DnsError::Truncated));
}
