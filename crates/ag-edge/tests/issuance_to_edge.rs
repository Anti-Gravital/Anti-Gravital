//! End-to-end glue: control-plane issuance -> edge certificate store -> SNI.
//!
//! Proves that a certificate produced through the `ag-domains` issuance
//! orchestration (deduplicated + rate-limited) can be loaded into the edge
//! `CertStore` via its PEM bridge and then selected by SNI. The issuer here is
//! a mock that mints a real self-signed certificate with `rcgen`, standing in
//! for the production ACME issuer.
#![cfg(feature = "tls")]

use std::sync::Arc;

use ag_domains::issuance::{
    issue_certificate, CertIssuer, IssuanceError, IssuanceLimiter, IssuedCertificate,
};
use ag_edge::cert::{server_config, CertStore};
use async_trait::async_trait;

struct RcgenIssuer;

#[async_trait]
impl CertIssuer for RcgenIssuer {
    async fn issue(&self, sans: &[String]) -> Result<IssuedCertificate, IssuanceError> {
        let ck = rcgen::generate_simple_self_signed(sans.to_vec())
            .map_err(|e| IssuanceError::Issuer(e.to_string()))?;
        Ok(IssuedCertificate {
            cert_chain_pem: ck.cert.pem(),
            private_key_pem: ck.key_pair.serialize_pem(),
        })
    }
}

#[tokio::test]
async fn issued_certificate_serves_over_sni() {
    let limiter = IssuanceLimiter::new(50);
    let issuer = RcgenIssuer;
    let host = "api.example.com".to_owned();

    // Issue through the control-plane orchestration (dedup + rate limit).
    let cert = issue_certificate(&limiter, &issuer, std::slice::from_ref(&host))
        .await
        .expect("issuance succeeds");
    assert_eq!(limiter.issued_count("example.com"), 1);

    // Load the issued PEM into the edge certificate store and select by SNI.
    let mut store = CertStore::new();
    store
        .insert_pem(&host, &cert.cert_chain_pem, &cert.private_key_pem)
        .expect("PEM loads into the edge cert store");
    assert!(store.resolve(&host).is_some());
    assert!(store.resolve("other.example.com").is_none());

    // The store builds a usable rustls server config (handshake-ready).
    assert!(server_config(Arc::new(store)).is_ok());
}
