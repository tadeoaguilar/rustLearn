//! Module 32 -- Network Programming. Reference solution.
//!
//! | File               | Exercise |
//! |--------------------|----------|
//! | `ex01_echo.rs`     | 1  TCP echo: a thread per client, then a task per client |
//! | `ex02_http.rs`     | 2  HTTP/1.1 from scratch: parsing, keep-alive, routing, static files |
//! | `ex03_proxy.rs`    | 3  a TCP proxy and a health-checked round-robin load balancer |
//! | `ex04_chat.rs`     | 4  a framed binary protocol: a chat server |
//! | `ex05_udp.rs`      | 5  UDP ping: sequence numbers, loss, round-trip times |
//! | `bonus_dns.rs`     | bonus: DNS queries and responses, with name compression |

pub mod bonus_dns;
pub mod ex01_echo;
pub mod ex02_http;
pub mod ex03_proxy;
pub mod ex04_chat;
pub mod ex05_udp;
