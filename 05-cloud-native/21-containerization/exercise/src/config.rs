//! Exercise 1: configuration from the environment.
//!
//! Twelve-factor rule III: config that varies between deploys lives in the
//! environment, not in the image. The same image then runs in dev, staging
//! and production. Two habits make that pleasant:
//!
//! - report *every* problem at once, not the first one -- a crash loop that
//!   reveals one typo per restart is miserable;
//! - read secrets from files (`API_TOKEN_FILE=/run/secrets/api_token`), which
//!   is how Docker and Kubernetes mount them, and never print them.

use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::time::Duration;

/// A value that must not end up in logs. `Debug` prints `Secret(***)`.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: impl Into<String>) -> Secret {
        Secret(value.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Exercise 1")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<LogLevel, String> {
        todo!("Exercise 1")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// HOST, default 0.0.0.0. Inside a container, 127.0.0.1 is the
    /// container's *own* loopback: nothing outside could connect.
    pub host: IpAddr,
    /// PORT, default 8080.
    pub port: u16,
    /// LOG_LEVEL, default info.
    pub log_level: LogLevel,
    /// DATABASE_URL, optional.
    pub database_url: Option<String>,
    /// API_TOKEN or API_TOKEN_FILE (not both), optional.
    pub api_token: Option<Secret>,
    /// DRAIN_DELAY_MS, default 5000: how long to keep serving after SIGTERM
    /// while load balancers notice we're no longer ready.
    pub drain_delay: Duration,
    /// SHUTDOWN_TIMEOUT_MS, default 20000: how long in-flight requests get
    /// to finish. Drain + timeout must fit in the orchestrator's grace
    /// period (Kubernetes: 30 s by default) or we get SIGKILLed.
    pub shutdown_timeout: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            host: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            port: 8080,
            log_level: LogLevel::Info,
            database_url: None,
            api_token: None,
            drain_delay: Duration::from_millis(5000),
            shutdown_timeout: Duration::from_millis(20000),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError {
    pub var: String,
    pub message: String,
}

/// Every problem found, in variable order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigErrors(pub Vec<ConfigError>);

impl fmt::Display for ConfigErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Exercise 1")
    }
}

impl std::error::Error for ConfigErrors {}

impl Config {
    /// Reads the configuration through `lookup` -- `std::env::var` in
    /// production, a map in tests (changing the real environment from
    /// parallel tests is a data race). Empty values count as unset.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Config, ConfigErrors> {
        todo!("Exercise 1")
    }

    pub fn from_env() -> Result<Config, ConfigErrors> {
        Config::from_lookup(|var| std::env::var(var).ok())
    }

    pub fn bind_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}
