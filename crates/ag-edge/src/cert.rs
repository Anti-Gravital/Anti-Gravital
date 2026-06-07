//! TLS certificate store and SNI resolver (real key material).
//!
//! Holds parsed `rustls` certified keys indexed by hostname and selects one for
//! a TLS ClientHello SNI value (RFC 6066). The `insert_pem` bridge accepts the
//! PEM chain + key produced by the ACME issuance flow in `ag-domains`, so an
//! issued certificate can be served by the edge without an extra format step.
//!
//! Private keys live only inside `rustls` signing keys here; they are not
//! logged (CLAUDE.md security rule 16).

use std::collections::HashMap;
use std::sync::Arc;

use rustls::server::{ClientHello, ResolvesServerCert};
use rustls::sign::CertifiedKey;
use rustls::ServerConfig;
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};

/// Errors building or loading a certificate.
#[derive(Debug, thiserror::Error)]
pub enum TlsError {
    /// PEM could not be parsed.
    #[error("invalid PEM: {0}")]
    Pem(String),
    /// No certificate found in the chain.
    #[error("empty certificate chain")]
    EmptyChain,
    /// The private key was not usable by the crypto provider.
    #[error("unusable private key: {0}")]
    Key(String),
    /// rustls server configuration error.
    #[error("rustls config error: {0}")]
    Config(String),
}

/// A store of certified keys indexed by hostname, with wildcard support.
#[derive(Default)]
pub struct CertStore {
    exact: HashMap<String, Arc<CertifiedKey>>,
    /// (base, key) for `*.base` certificates.
    wildcard: Vec<(String, Arc<CertifiedKey>)>,
}

impl std::fmt::Debug for CertStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CertStore")
            .field("exact", &self.exact.keys().collect::<Vec<_>>())
            .field(
                "wildcard",
                &self.wildcard.iter().map(|(b, _)| b).collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl CertStore {
    /// Creates an empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a certificate from DER components.
    pub fn insert_der(
        &mut self,
        hostname: &str,
        chain: Vec<CertificateDer<'static>>,
        key: PrivateKeyDer<'static>,
    ) -> Result<(), TlsError> {
        if chain.is_empty() {
            return Err(TlsError::EmptyChain);
        }
        let signing_key = rustls::crypto::ring::sign::any_supported_type(&key)
            .map_err(|e| TlsError::Key(e.to_string()))?;
        let certified = Arc::new(CertifiedKey::new(chain, signing_key));
        self.insert_certified(hostname, certified);
        Ok(())
    }

    /// Inserts a certificate from a PEM chain and PEM private key.
    ///
    /// This is the bridge from the `ag-domains` ACME issuance output
    /// (`IssuedCert { cert_chain_pem, private_key_pem }`).
    pub fn insert_pem(
        &mut self,
        hostname: &str,
        cert_chain_pem: &str,
        private_key_pem: &str,
    ) -> Result<(), TlsError> {
        let chain = CertificateDer::pem_slice_iter(cert_chain_pem.as_bytes())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TlsError::Pem(e.to_string()))?;
        let key = PrivateKeyDer::from_pem_slice(private_key_pem.as_bytes())
            .map_err(|e| TlsError::Pem(e.to_string()))?;
        self.insert_der(hostname, chain, key)
    }

    fn insert_certified(&mut self, hostname: &str, certified: Arc<CertifiedKey>) {
        let hostname = hostname.trim_end_matches('.').to_ascii_lowercase();
        if let Some(base) = hostname.strip_prefix("*.") {
            self.wildcard.push((base.to_owned(), certified));
        } else {
            self.exact.insert(hostname, certified);
        }
    }

    /// Resolves a certified key for an SNI value: exact match, then a
    /// single-label wildcard match.
    pub fn resolve(&self, sni: &str) -> Option<Arc<CertifiedKey>> {
        let sni = sni.trim_end_matches('.').to_ascii_lowercase();
        if let Some(k) = self.exact.get(&sni) {
            return Some(k.clone());
        }
        for (base, key) in &self.wildcard {
            if let Some(prefix) = sni.strip_suffix(base) {
                let prefix = prefix.strip_suffix('.').unwrap_or(prefix);
                if !prefix.is_empty() && !prefix.contains('.') {
                    return Some(key.clone());
                }
            }
        }
        None
    }

    /// Number of certificates held.
    pub fn len(&self) -> usize {
        self.exact.len() + self.wildcard.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A `rustls` certificate resolver backed by a [`CertStore`].
#[derive(Debug)]
pub struct SniCertResolver {
    store: Arc<CertStore>,
}

impl SniCertResolver {
    /// Wraps a shared certificate store.
    pub fn new(store: Arc<CertStore>) -> Self {
        Self { store }
    }
}

impl ResolvesServerCert for SniCertResolver {
    fn resolve(&self, client_hello: ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
        let sni = client_hello.server_name()?;
        self.store.resolve(sni)
    }
}

/// Builds a `rustls` server configuration that selects certificates by SNI
/// from the given store, using the ring crypto provider explicitly (so it does
/// not depend on a process-global default provider).
pub fn server_config(store: Arc<CertStore>) -> Result<Arc<ServerConfig>, TlsError> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| TlsError::Config(e.to_string()))?
        .with_no_client_auth()
        .with_cert_resolver(Arc::new(SniCertResolver::new(store)));
    Ok(Arc::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn self_signed(names: &[&str]) -> (String, String) {
        let names: Vec<String> = names.iter().map(|s| s.to_string()).collect();
        let ck = rcgen::generate_simple_self_signed(names).unwrap();
        (ck.cert.pem(), ck.key_pair.serialize_pem())
    }

    #[test]
    fn insert_pem_and_resolve_exact() {
        let (cert, key) = self_signed(&["api.example.com"]);
        let mut store = CertStore::new();
        store.insert_pem("api.example.com", &cert, &key).unwrap();
        assert!(store.resolve("api.example.com").is_some());
        assert!(store.resolve("API.Example.com.").is_some());
        assert!(store.resolve("other.example.com").is_none());
    }

    #[test]
    fn wildcard_resolution() {
        let (cert, key) = self_signed(&["*.example.com"]);
        let mut store = CertStore::new();
        store.insert_pem("*.example.com", &cert, &key).unwrap();
        assert!(store.resolve("anything.example.com").is_some());
        assert!(store.resolve("a.b.example.com").is_none());
    }

    #[test]
    fn invalid_pem_rejected() {
        let mut store = CertStore::new();
        let err = store.insert_pem("x.example.com", "not pem", "not pem");
        assert!(err.is_err());
    }

    #[test]
    fn server_config_builds() {
        let (cert, key) = self_signed(&["api.example.com"]);
        let mut store = CertStore::new();
        store.insert_pem("api.example.com", &cert, &key).unwrap();
        let cfg = server_config(Arc::new(store));
        assert!(cfg.is_ok());
    }
}
