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
        f.write_str("Secret(***)")
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
        match s.to_ascii_lowercase().as_str() {
            "error" => Ok(LogLevel::Error),
            "warn" | "warning" => Ok(LogLevel::Warn),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            "trace" => Ok(LogLevel::Trace),
            _ => Err("expected one of error, warn, info, debug, trace".into()),
        }
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
        writeln!(f, "invalid configuration:")?;
        for e in &self.0 {
            writeln!(f, "  {}: {}", e.var, e.message)?;
        }
        Ok(())
    }
}

impl std::error::Error for ConfigErrors {}

impl Config {
    /// Reads the configuration through `lookup` -- `std::env::var` in
    /// production, a map in tests (changing the real environment from
    /// parallel tests is a data race). Empty values count as unset.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Config, ConfigErrors> {
        let get = |var: &str| lookup(var).filter(|v| !v.trim().is_empty());
        let mut errors = Vec::new();
        let defaults = Config::default();

        let host = parse(&get, "HOST", defaults.host, &mut errors);
        let port: u16 = parse(&get, "PORT", defaults.port, &mut errors);
        if port == 0 {
            error(&mut errors, "PORT", "must be between 1 and 65535");
        }
        let log_level = parse(&get, "LOG_LEVEL", defaults.log_level, &mut errors);
        let database_url = get("DATABASE_URL");

        let api_token = match (get("API_TOKEN"), get("API_TOKEN_FILE")) {
            (Some(_), Some(_)) => {
                error(
                    &mut errors,
                    "API_TOKEN_FILE",
                    "set API_TOKEN or API_TOKEN_FILE, not both",
                );
                None
            }
            (Some(token), None) => Some(Secret::new(token)),
            (None, Some(path)) => match std::fs::read_to_string(&path) {
                // Files written by editors and `echo` end in a newline.
                Ok(contents) => Some(Secret::new(contents.trim_end_matches(['\n', '\r']))),
                Err(e) => {
                    error(
                        &mut errors,
                        "API_TOKEN_FILE",
                        &format!("can't read {path}: {e}"),
                    );
                    None
                }
            },
            (None, None) => None,
        };

        let drain_ms: u64 = parse(
            &get,
            "DRAIN_DELAY_MS",
            defaults.drain_delay.as_millis() as u64,
            &mut errors,
        );
        let timeout_ms: u64 = parse(
            &get,
            "SHUTDOWN_TIMEOUT_MS",
            defaults.shutdown_timeout.as_millis() as u64,
            &mut errors,
        );

        if errors.is_empty() {
            Ok(Config {
                host,
                port,
                log_level,
                database_url,
                api_token,
                drain_delay: Duration::from_millis(drain_ms),
                shutdown_timeout: Duration::from_millis(timeout_ms),
            })
        } else {
            Err(ConfigErrors(errors))
        }
    }

    pub fn from_env() -> Result<Config, ConfigErrors> {
        Config::from_lookup(|var| std::env::var(var).ok())
    }

    pub fn bind_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}

fn error(errors: &mut Vec<ConfigError>, var: &str, message: &str) {
    errors.push(ConfigError {
        var: var.into(),
        message: message.into(),
    });
}

/// The default if `var` is unset; an error (and the default, to keep going)
/// if it doesn't parse.
fn parse<T: FromStr>(
    get: &impl Fn(&str) -> Option<String>,
    var: &str,
    default: T,
    errors: &mut Vec<ConfigError>,
) -> T
where
    T::Err: fmt::Display,
{
    match get(var) {
        None => default,
        Some(raw) => raw.trim().parse().unwrap_or_else(|e| {
            error(errors, var, &format!("{raw:?}: {e}"));
            default
        }),
    }
}
