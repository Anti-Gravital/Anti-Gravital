//! SNI certificate selection and restricted on-demand issuance.
//!
//! The edge selects a certificate for a TLS ClientHello SNI value (RFC 6066).
//! To match certificates this module stores opaque certificate references
//! (handles), never private key material (CLAUDE.md security rule 16; blueprint
//! section 15.5: certificate references in logs, not raw keys).
//!
//! On-demand TLS is restricted: a certificate may only be ordered for an SNI
//! that already exists as a verified attachment (blueprint section 13.6). The
//! edge never issues for an arbitrary SNI seen during a handshake.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// An opaque reference to a stored certificate (not the key material).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertHandle(pub String);

/// Maps SNI hostnames to certificate handles, with wildcard support.
#[derive(Debug, Default)]
pub struct SniCertStore {
    exact: HashMap<String, CertHandle>,
    /// (base, handle) for `*.base` certificates.
    wildcard: Vec<(String, CertHandle)>,
}

impl SniCertStore {
    /// Creates an empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a certificate for an exact hostname or a `*.base` wildcard.
    pub fn insert(&mut self, hostname_ascii: &str, handle: CertHandle) {
        if let Some(base) = hostname_ascii.strip_prefix("*.") {
            self.wildcard.push((base.to_owned(), handle));
        } else {
            self.exact.insert(hostname_ascii.to_owned(), handle);
        }
    }

    /// Selects the certificate for an SNI value: exact match first, then a
    /// single-label wildcard match.
    pub fn select(&self, sni: &str) -> Option<&CertHandle> {
        let sni = sni.trim_end_matches('.').to_ascii_lowercase();
        if let Some(h) = self.exact.get(&sni) {
            return Some(h);
        }
        for (base, handle) in &self.wildcard {
            if let Some(prefix) = sni.strip_suffix(base) {
                let prefix = prefix.strip_suffix('.').unwrap_or(prefix);
                if !prefix.is_empty() && !prefix.contains('.') {
                    return Some(handle);
                }
            }
        }
        None
    }
}

/// Whether on-demand TLS issuance is permitted for an SNI value.
///
/// Returns `true` only when `is_known_verified` confirms the hostname is an
/// existing, ownership-verified attachment. This prevents unbounded issuance
/// for arbitrary handshake SNI values.
pub fn allow_on_demand(sni: &str, is_known_verified: impl Fn(&str) -> bool) -> bool {
    let sni = sni.trim_end_matches('.').to_ascii_lowercase();
    if sni.is_empty() || sni.contains('/') || sni.contains(' ') {
        return false;
    }
    is_known_verified(&sni)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_sni_selects_cert() {
        let mut store = SniCertStore::new();
        store.insert("api.example.com", CertHandle("cert_1".to_owned()));
        assert_eq!(
            store.select("api.example.com"),
            Some(&CertHandle("cert_1".to_owned()))
        );
        assert_eq!(
            store.select("API.Example.com."),
            Some(&CertHandle("cert_1".to_owned()))
        );
    }

    #[test]
    fn wildcard_sni_selects_cert() {
        let mut store = SniCertStore::new();
        store.insert("*.example.com", CertHandle("cert_wild".to_owned()));
        assert_eq!(
            store.select("anything.example.com"),
            Some(&CertHandle("cert_wild".to_owned()))
        );
        assert_eq!(store.select("a.b.example.com"), None);
    }

    #[test]
    fn unknown_sni_has_no_cert() {
        let store = SniCertStore::new();
        assert_eq!(store.select("nope.example.com"), None);
    }

    #[test]
    fn on_demand_only_for_known_verified() {
        let known = |h: &str| h == "api.example.com";
        assert!(allow_on_demand("api.example.com", known));
        assert!(!allow_on_demand("evil.example.com", known));
    }

    #[test]
    fn on_demand_rejects_malformed_sni() {
        let always = |_: &str| true;
        assert!(!allow_on_demand("", always));
        assert!(!allow_on_demand("a/b", always));
    }
}
