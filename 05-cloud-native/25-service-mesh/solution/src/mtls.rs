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
        let mut params = CertificateParams::new(Vec::<String>::new()).expect("empty SAN list");
        params.distinguished_name = DistinguishedName::new();
        params
            .distinguished_name
            .push(DnType::CommonName, "rustlearn mesh CA");
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
            KeyUsagePurpose::DigitalSignature,
        ];
        let ca = CertifiedIssuer::self_signed(params, KeyPair::generate().expect("key"))
            .expect("CA certificate");
        Pki { ca }
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
        let mut params =
            CertificateParams::new(dns_name.map(|d| vec![d.to_string()]).unwrap_or_default())
                .expect("valid DNS name");
        params
            .subject_alt_names
            .push(SanType::URI(spiffe_id.try_into().expect("ASCII SPIFFE id")));
        params.distinguished_name = DistinguishedName::new();
        params.extended_key_usages = vec![
            ExtendedKeyUsagePurpose::ServerAuth,
            ExtendedKeyUsagePurpose::ClientAuth,
        ];
        if expired {
            params.not_before = date_time_ymd(2020, 1, 1);
            params.not_after = date_time_ymd(2021, 1, 1);
        }
        let key = KeyPair::generate().expect("key");
        let cert = params
            .signed_by(&key, &self.ca)
            .expect("signed certificate");
        Identity {
            spiffe_id: spiffe_id.into(),
            cert: cert.der().clone(),
            key: PrivatePkcs8KeyDer::from(key.serialize_der()),
        }
    }

    /// Requires clients to present a certificate signed by this CA.
    pub fn server_config(&self, id: &Identity) -> Arc<ServerConfig> {
        let verifier =
            WebPkiClientVerifier::builder_with_provider(Arc::new(self.roots()), provider())
                .build()
                .expect("verifier");
        let config = ServerConfig::builder_with_provider(provider())
            .with_safe_default_protocol_versions()
            .expect("protocol versions")
            .with_client_cert_verifier(verifier)
            .with_single_cert(vec![id.cert.clone()], id.key())
            .expect("server certificate");
        Arc::new(config)
    }

    /// Trusts this CA; presents `id` if given.
    pub fn client_config(&self, id: Option<&Identity>) -> Arc<ClientConfig> {
        let builder = ClientConfig::builder_with_provider(provider())
            .with_safe_default_protocol_versions()
            .expect("protocol versions")
            .with_root_certificates(self.roots());
        let config = match id {
            Some(id) => builder
                .with_client_auth_cert(vec![id.cert.clone()], id.key())
                .expect("client certificate"),
            None => builder.with_no_client_auth(),
        };
        Arc::new(config)
    }
}

/// The SPIFFE id in a certificate's URI SANs.
pub fn spiffe_id(cert: &CertificateDer<'_>) -> Option<String> {
    let (_, parsed) = x509_parser::parse_x509_certificate(cert.as_ref()).ok()?;
    let san = parsed.subject_alternative_name().ok()??;
    san.value.general_names.iter().find_map(|name| match name {
        x509_parser::extensions::GeneralName::URI(uri) if uri.starts_with("spiffe://") => {
            Some(uri.to_string())
        }
        _ => None,
    })
}

/// A tiny mTLS service: reads one line "METHOD /path", authorizes the
/// caller's SPIFFE id against `policy`, answers "200 hello <id>" or "403 ...".
pub async fn serve(listener: TcpListener, config: Arc<ServerConfig>, policy: Arc<Policy>) {
    let acceptor = TlsAcceptor::from(config);
    while let Ok((tcp, _)) = listener.accept().await {
        let (acceptor, policy) = (acceptor.clone(), policy.clone());
        tokio::spawn(async move {
            // A failed handshake (no or untrusted certificate) ends here.
            let Ok(tls) = acceptor.accept(tcp).await else {
                return;
            };
            let peer = tls
                .get_ref()
                .1
                .peer_certificates()
                .and_then(|c| c.first())
                .and_then(spiffe_id);
            let mut tls = BufReader::new(tls);
            let mut line = String::new();
            if tls.read_line(&mut line).await.is_err() {
                return;
            }
            let mut parts = line.split_whitespace();
            let (method, path) = (parts.next().unwrap_or(""), parts.next().unwrap_or("/"));
            let reply = match policy.authorize(peer.as_deref(), method, path) {
                Decision::Allow => format!("200 hello {}\n", peer.unwrap_or_default()),
                Decision::Unauthenticated => "401 no identity\n".to_string(),
                Decision::Deny => {
                    format!("403 {} may not {method} {path}\n", peer.unwrap_or_default())
                }
            };
            let _ = tls.get_mut().write_all(reply.as_bytes()).await;
            let _ = tls.get_mut().shutdown().await;
        });
    }
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
    let tcp = TcpStream::connect(addr).await?;
    let name = ServerName::try_from(server_name.to_string())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let tls = TlsConnector::from(config).connect(name, tcp).await?;
    let mut tls = BufReader::new(tls);
    tls.get_mut()
        .write_all(format!("{method} {path}\n").as_bytes())
        .await?;
    let mut reply = String::new();
    tls.read_line(&mut reply).await?;
    if reply.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::ConnectionAborted,
            "closed without a reply",
        ));
    }
    Ok(reply.trim_end().to_string())
}
