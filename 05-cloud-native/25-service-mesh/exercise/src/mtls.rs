//! Exercise 5: mutual TLS with workload identities.
//!
//! In ordinary TLS only the server proves who it is. In **mutual** TLS the
//! client presents a certificate too, so both sides know who they're talking
//! to -- and the traffic is encrypted. A mesh gives every workload a
//! certificate whose URI SAN is a **SPIFFE id**:
//!
//! ```text
//!   spiffe://cluster.local/ns/shop/sa/orders
//!            trust domain  namespace  service account
//! ```
//!
//! and rotates it every day or so. Here: our own CA (rcgen), certificates
//! for workloads, and rustls on both ends.

use crate::policy::{Decision, Policy};
use rcgen::{
    BasicConstraints, CertificateParams, CertifiedIssuer, DistinguishedName, DnType,
    ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose, SanType, date_time_ymd,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName};
use rustls::server::WebPkiClientVerifier;
use rustls::{ClientConfig, RootCertStore, ServerConfig};
use std::io;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{TlsAcceptor, TlsConnector};

/// A workload's certificate and private key.
pub struct Identity {
    pub spiffe_id: String,
    pub cert: CertificateDer<'static>,
    key: PrivatePkcs8KeyDer<'static>,
}

impl Identity {
    fn key(&self) -> PrivateKeyDer<'static> {
        PrivateKeyDer::Pkcs8(self.key.clone_key())
    }
}

/// The mesh's certificate authority.
pub struct Pki {
    ca: CertifiedIssuer<'static, KeyPair>,
}

impl Default for Pki {
    fn default() -> Pki {
        Pki::new()
    }
}

fn provider() -> Arc<rustls::crypto::CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

impl Pki {
    pub fn new() -> Pki {
        todo!("Exercise 5")
    }

    pub fn ca_cert(&self) -> CertificateDer<'static> {
        self.ca.der().clone()
    }

    fn roots(&self) -> RootCertStore {
        let mut roots = RootCertStore::empty();
        roots.add(self.ca_cert()).expect("valid CA certificate");
        roots
    }

    /// A certificate for `spiffe_id` (as a URI SAN), plus `dns_name` for
    /// servers that clients reach by name. `expired` issues one that ran out
    /// in 2021, for testing.
    pub fn issue(&self, spiffe_id: &str, dns_name: Option<&str>, expired: bool) -> Identity {
        todo!("Exercise 5")
    }

    /// Requires clients to present a certificate signed by this CA.
    pub fn server_config(&self, id: &Identity) -> Arc<ServerConfig> {
        todo!("Exercise 5")
    }

    /// Trusts this CA; presents `id` if given.
    pub fn client_config(&self, id: Option<&Identity>) -> Arc<ClientConfig> {
        todo!("Exercise 5")
    }
}

/// The SPIFFE id in a certificate's URI SANs.
pub fn spiffe_id(cert: &CertificateDer<'_>) -> Option<String> {
    todo!("Exercise 5")
}

/// A tiny mTLS service: reads one line "METHOD /path", authorizes the
/// caller's SPIFFE id against `policy`, answers "200 hello <id>" or "403 ...".
pub async fn serve(listener: TcpListener, config: Arc<ServerConfig>, policy: Arc<Policy>) {
    todo!("Exercise 5")
}

/// Connects with mTLS, sends "METHOD /path", returns the reply line.
/// Handshake and certificate failures come back as errors.
pub async fn call(
    addr: &str,
    config: Arc<ClientConfig>,
    server_name: &str,
    method: &str,
    path: &str,
) -> io::Result<String> {
    todo!("Exercise 5")
}
