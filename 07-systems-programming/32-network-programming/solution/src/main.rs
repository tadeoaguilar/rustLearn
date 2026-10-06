// Reference solution for 32-network-programming.
//
//     cargo run -p m32-network-programming-solution -- <1-5|bonus|all>    demos on local ports
//     cargo run -p m32-network-programming-solution -- http 8080 .        a real HTTP server (curl it)

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use m32_network_programming_solution::ex03_proxy::{Balancer, Stats};
use m32_network_programming_solution::ex04_chat::{ChatClient, Room};
use m32_network_programming_solution::{
    bonus_dns, ex01_echo, ex02_http, ex03_proxy, ex04_chat, ex05_udp,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};

async fn local() -> (TcpListener, SocketAddr) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    (listener, addr)
}

async fn ex1() {
    println!("--- 1: echo");
    let blocking = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let blocking_addr = blocking.local_addr().unwrap();
    let served = Arc::new(AtomicUsize::new(0));
    let counter = served.clone();
    std::thread::spawn(move || ex01_echo::serve_blocking(blocking, counter));
    let (listener, addr) = local().await;
    tokio::spawn(ex01_echo::serve_async(listener));
    for (name, addr) in [
        ("blocking (thread per client)", blocking_addr),
        ("async (task per client)", addr),
    ] {
        let replies = ex01_echo::echo_client(addr, &["hello", "world"])
            .await
            .unwrap();
        println!("{name}: {replies:?}");
    }
    let clients: Vec<_> = (0..100)
        .map(|i| {
            tokio::spawn(
                async move { ex01_echo::echo_client(addr, &[&format!("client {i}")]).await },
            )
        })
        .collect();
    let mut ok = 0;
    for c in clients {
        ok += c.await.unwrap().is_ok() as usize;
    }
    println!("100 concurrent clients on the async server: {ok} answered");
    tokio::time::sleep(Duration::from_millis(50)).await;
    println!(
        "the blocking server finished {} clients",
        served.load(Ordering::SeqCst)
    );
}

async fn raw_http(addr: SocketAddr, request: &str) -> String {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();
    response
}

async fn ex2() {
    println!("--- 2: HTTP/1.1 from scratch");
    let dir = std::env::temp_dir().join("m32-static-demo");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.html"), "<h1>static</h1>\n").unwrap();
    let (listener, addr) = local().await;
    tokio::spawn(ex02_http::serve(listener, dir));
    for request in [
        "GET / HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n",
        "GET /echo?msg=hello+there%21 HTTP/1.1\r\nConnection: close\r\n\r\n",
        "POST /upper HTTP/1.1\r\nContent-Length: 5\r\nConnection: close\r\n\r\nshout",
        "GET /static/index.html HTTP/1.1\r\nConnection: close\r\n\r\n",
        "GET /static/../../etc/passwd HTTP/1.1\r\nConnection: close\r\n\r\n",
        "DELETE / HTTP/1.1\r\nConnection: close\r\n\r\n",
        "NONSENSE\r\n\r\n",
    ] {
        let first_line = request.lines().next().unwrap();
        let response = raw_http(addr, request).await;
        let status = response.lines().next().unwrap_or("");
        let body = response.split("\r\n\r\n").nth(1).unwrap_or("").trim_end();
        println!("{first_line:<45} -> {status} | {body}");
    }
    // keep-alive: two requests on one connection
    let both = raw_http(
        addr,
        "GET / HTTP/1.1\r\n\r\nGET /echo?msg=again HTTP/1.1\r\nConnection: close\r\n\r\n",
    )
    .await;
    println!(
        "two requests, one connection: {} responses",
        both.matches("HTTP/1.1 200 OK").count()
    );
}

async fn ex3() {
    println!("--- 3: proxy and load balancer");
    let mut upstreams = Vec::new();
    for _ in 0..3 {
        let (listener, addr) = local().await;
        tokio::spawn(ex01_echo::serve_async(listener));
        upstreams.push(addr);
    }
    let stats = Arc::new(Stats::default());
    let (listener, proxy) = local().await;
    tokio::spawn(ex03_proxy::serve_proxy(
        listener,
        upstreams[0],
        stats.clone(),
    ));
    let replies = ex01_echo::echo_client(proxy, &["through the proxy"])
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(20)).await;
    println!(
        "{replies:?}; bytes up {}, down {}",
        stats.bytes_up.load(Ordering::SeqCst),
        stats.bytes_down.load(Ordering::SeqCst)
    );

    let balancer = Arc::new(Balancer::new(upstreams.clone()));
    balancer.set_healthy(1, false);
    let picks: Vec<u16> = (0..6).map(|_| balancer.pick().unwrap().port()).collect();
    println!("round-robin with upstream 1 down: ports {picks:?}");
    let (listener, lb) = local().await;
    tokio::spawn(ex03_proxy::serve_balanced(
        listener,
        balancer.clone(),
        stats.clone(),
    ));
    println!(
        "through the balancer: {:?}",
        ex01_echo::echo_client(lb, &["balanced"]).await.unwrap()
    );
}

async fn ex4() {
    println!("--- 4: chat over a framed protocol");
    let (listener, addr) = local().await;
    tokio::spawn(ex04_chat::serve_chat(listener, Arc::new(Room::new())));
    let (mut ana, welcome) = ChatClient::join(addr, "ana").await.unwrap();
    println!("ana joins: {welcome:?}");
    let (mut ben, welcome) = ChatClient::join(addr, "ben").await.unwrap();
    println!("ben joins: {welcome:?}");
    println!("ana sees: {:?}", ana.next().await.unwrap());
    let (_, refused) = ChatClient::join(addr, "ana").await.unwrap();
    println!("a second 'ana': {refused:?}");
    ben.say("hi ana!").await.unwrap();
    println!("ana sees: {:?}", ana.next().await.unwrap());
    drop(ben);
    println!("ben disconnects; ana sees: {:?}", ana.next().await.unwrap());
}

async fn ex5() {
    println!("--- 5: UDP ping");
    for drop_every in [None, Some(4)] {
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let addr = socket.local_addr().unwrap();
        tokio::spawn(ex05_udp::serve_pong(socket, drop_every));
        let stats = ex05_udp::ping(
            addr,
            8,
            Duration::from_millis(5),
            Duration::from_millis(100),
        )
        .await
        .unwrap();
        println!(
            "dropping every {drop_every:?}: {}/{} received, {:.1}% loss, rtt min/avg/max {:?}/{:?}/{:?}",
            stats.received,
            stats.sent,
            stats.loss_percent(),
            stats.min().unwrap(),
            stats.avg().unwrap(),
            stats.max().unwrap()
        );
    }
}

fn bonus() {
    println!("--- bonus: DNS");
    let query = bonus_dns::build_query(0xBEEF, "example.com").unwrap();
    println!(
        "query for example.com: {} bytes: {:02x?}",
        query.len(),
        query
    );
    // A response as a server would send it: the question, then one A record
    // whose name is a pointer (0xC00C) back to the question's name.
    let mut response = query.clone();
    response[2..4].copy_from_slice(&0x8180u16.to_be_bytes()); // response, RD, RA
    response[6..8].copy_from_slice(&1u16.to_be_bytes()); // one answer
    response.extend_from_slice(&[
        0xC0, 0x0C, 0, 1, 0, 1, 0, 0, 0x0E, 0x10, 0, 4, 93, 184, 215, 14,
    ]);
    let parsed = bonus_dns::parse_response(&response).unwrap();
    println!(
        "parsed: id {:#06x}, {:?} -> {:?} (ttl {} s)",
        parsed.id,
        parsed.answers[0].name,
        parsed.a_records(),
        parsed.answers[0].ttl
    );
}

async fn http_server(port: u16, dir: String) {
    let listener = TcpListener::bind(("127.0.0.1", port)).await.expect("bind");
    println!(
        "serving http://127.0.0.1:{port}/  (/echo?msg=hi, POST /upper, /static/<file> from {dir})"
    );
    ex02_http::serve(listener, dir.into()).await.unwrap();
}

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("1") => rt.block_on(ex1()),
        Some("2") => rt.block_on(ex2()),
        Some("3") => rt.block_on(ex3()),
        Some("4") => rt.block_on(ex4()),
        Some("5") => rt.block_on(ex5()),
        Some("bonus") => bonus(),
        Some("all") => rt.block_on(async {
            ex1().await;
            ex2().await;
            ex3().await;
            ex4().await;
            ex5().await;
            bonus();
        }),
        Some("http") => {
            let port = args.get(2).and_then(|p| p.parse().ok()).unwrap_or(8080);
            let dir = args.get(3).cloned().unwrap_or_else(|| ".".into());
            rt.block_on(http_server(port, dir));
        }
        _ => println!(
            "32-network-programming -- reference solution\n\n  cargo run -p m32-network-programming-solution -- <1-5|bonus|all>\n  cargo run -p m32-network-programming-solution -- http [port] [static dir]"
        ),
    }
}
