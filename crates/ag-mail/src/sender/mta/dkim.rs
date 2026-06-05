//! Outbound DKIM signing for the native MTA.
//!
//! `ag-mail` consumes DKIM key material; it does not own the DNS side. The
//! key lifecycle and the published `selector._domainkey` TXT record are
//! `ag-domains`' responsibility (cooperation, no dependency cycle). Here we
//! only build a signer from a private key supplied by the caller and sign the
//! message body just before `DATA`, after any future link rewriting, so the
//! signature stays valid.
//!
//! Phase 4.6-A supports Ed25519 (RFC 8463) keys in PKCS#8 DER form. RSA
//! support is tracked as follow-up work in `docs/DEBT.md`.

use mail_auth::{
    common::{crypto::Ed25519Key, headers::HeaderWriter},
    dkim::{DkimSigner, Done},
};

use crate::error::AgMailError;

/// Headers covered by the DKIM signature. Order matters and follows common
/// practice for transactional mail.
const SIGNED_HEADERS: [&str; 7] = [
    "From",
    "To",
    "Subject",
    "Date",
    "Message-ID",
    "MIME-Version",
    "Content-Type",
];

/// DKIM signing configuration for one sending domain.
#[derive(Clone)]
pub struct DkimConfig {
    /// Signing domain (the `d=` tag); must match the `From` domain for DMARC
    /// alignment.
    pub domain: String,
    /// Selector (the `s=` tag); the public key lives at
    /// `selector._domainkey.domain`.
    pub selector: String,
    /// Ed25519 private key in PKCS#8 DER form.
    pub ed25519_pkcs8_der: Vec<u8>,
}

impl std::fmt::Debug for DkimConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never print key material.
        f.debug_struct("DkimConfig")
            .field("domain", &self.domain)
            .field("selector", &self.selector)
            .field("ed25519_pkcs8_der", &"<redacted>")
            .finish()
    }
}

impl DkimConfig {
    /// Creates a configuration from an Ed25519 PKCS#8 DER key.
    pub fn ed25519(
        domain: impl Into<String>,
        selector: impl Into<String>,
        ed25519_pkcs8_der: Vec<u8>,
    ) -> Self {
        Self {
            domain: domain.into(),
            selector: selector.into(),
            ed25519_pkcs8_der,
        }
    }

    /// Builds a ready-to-use DKIM signer from this configuration.
    ///
    /// The signer is built per send rather than cached so the key bytes are
    /// the single source of truth and rotation by the caller takes effect
    /// immediately.
    pub fn signer(&self) -> Result<DkimSigner<Ed25519Key, Done>, AgMailError> {
        let key = Ed25519Key::from_pkcs8_der(&self.ed25519_pkcs8_der)
            .map_err(|e| AgMailError::Dkim(e.to_string()))?;
        Ok(DkimSigner::from_key(key)
            .domain(&self.domain)
            .selector(&self.selector)
            .headers(SIGNED_HEADERS))
    }

    /// Signs the RFC 5322 message bytes and returns the signed message: the
    /// `DKIM-Signature` header followed by the original content. This mirrors
    /// what an SMTP client does before `DATA`, and is applied last so the
    /// signature covers the final bytes.
    pub fn sign(&self, content: &[u8]) -> Result<Vec<u8>, AgMailError> {
        let signer = self.signer()?;
        let signature = signer
            .sign(content)
            .map_err(|e| AgMailError::Dkim(e.to_string()))?;
        let mut signed = Vec::with_capacity(content.len() + 128);
        signature.write_header(&mut signed);
        signed.extend_from_slice(content);
        Ok(signed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generated_key() -> Vec<u8> {
        Ed25519Key::generate_pkcs8().expect("key generation succeeds")
    }

    #[test]
    fn builds_signer_from_generated_key() {
        let cfg = DkimConfig::ed25519("example.com", "ag2026", generated_key());
        assert!(cfg.signer().is_ok());
    }

    #[test]
    fn rejects_invalid_key_bytes() {
        let cfg = DkimConfig::ed25519("example.com", "ag2026", vec![0u8; 8]);
        // The Ok variant (DkimSigner) is not Debug, so assert via matches!.
        assert!(matches!(cfg.signer(), Err(AgMailError::Dkim(_))));
    }

    #[test]
    fn sign_prepends_dkim_signature_header() {
        let cfg = DkimConfig::ed25519("example.com", "ag2026", generated_key());
        let content = b"From: a@example.com\r\nSubject: hi\r\n\r\nbody\r\n";
        let signed = cfg.sign(content).expect("signing succeeds");
        let text = String::from_utf8_lossy(&signed);
        assert!(text.starts_with("DKIM-Signature:"));
        assert!(text.contains("d=example.com"));
        assert!(text.contains("s=ag2026"));
        // Original content is preserved after the header.
        assert!(signed.ends_with(content));
    }

    #[test]
    fn sign_rejects_invalid_key() {
        let cfg = DkimConfig::ed25519("example.com", "ag2026", vec![0u8; 8]);
        assert!(matches!(cfg.sign(b"x"), Err(AgMailError::Dkim(_))));
    }

    #[test]
    fn debug_redacts_key_material() {
        let cfg = DkimConfig::ed25519("example.com", "ag2026", vec![1, 2, 3, 4]);
        let rendered = format!("{cfg:?}");
        assert!(rendered.contains("redacted"));
        assert!(!rendered.contains("[1, 2, 3, 4]"));
    }
}
