//! Outbound DKIM signing for the native MTA.
//!
//! `ag-mail` consumes DKIM key material; it does not own the DNS side. The
//! key lifecycle and the published `selector._domainkey` TXT record are
//! `ag-domains`' responsibility (cooperation, no dependency cycle). Here we
//! only build a signer from a private key supplied by the caller and sign the
//! message body just before `DATA`, after any future link rewriting, so the
//! signature stays valid.
//!
//! Both Ed25519 (RFC 8463) and RSA (RFC 6376, RSA-SHA256) keys are supported.

use mail_auth::{
    common::{
        crypto::{Ed25519Key, RsaKey, Sha256, SigningKey},
        headers::HeaderWriter,
    },
    dkim::DkimSigner,
};
use rustls_pki_types::{pem::PemObject, PrivateKeyDer, PrivatePkcs8KeyDer};

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

/// DKIM private key material supplied by the caller / `ag-domains`.
///
/// The DNS side publishes the matching public key and advertises the algorithm
/// (`k=ed25519` or `k=rsa`); here we only hold the private key to sign with.
#[derive(Clone)]
pub enum DkimKey {
    /// Ed25519 private key in PKCS#8 DER form (RFC 8463).
    Ed25519Pkcs8Der(Vec<u8>),
    /// RSA private key in PKCS#8 DER form (RSA-SHA256).
    RsaPkcs8Der(Vec<u8>),
    /// RSA private key in PKCS#8 PEM form (RSA-SHA256).
    RsaPkcs8Pem(String),
}

impl std::fmt::Debug for DkimKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never print key material; only the variant/algorithm.
        let kind = match self {
            DkimKey::Ed25519Pkcs8Der(_) => "Ed25519Pkcs8Der",
            DkimKey::RsaPkcs8Der(_) => "RsaPkcs8Der",
            DkimKey::RsaPkcs8Pem(_) => "RsaPkcs8Pem",
        };
        write!(f, "DkimKey::{kind}(<redacted>)")
    }
}

/// DKIM signing configuration for one sending domain.
#[derive(Clone)]
pub struct DkimConfig {
    /// Signing domain (the `d=` tag); must match the `From` domain for DMARC
    /// alignment.
    pub domain: String,
    /// Selector (the `s=` tag); the public key lives at
    /// `selector._domainkey.domain`.
    pub selector: String,
    /// Private key material.
    key: DkimKey,
}

impl std::fmt::Debug for DkimConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DkimConfig")
            .field("domain", &self.domain)
            .field("selector", &self.selector)
            .field("key", &self.key)
            .finish()
    }
}

impl DkimConfig {
    /// Creates a configuration from an Ed25519 PKCS#8 DER key.
    pub fn ed25519(
        domain: impl Into<String>,
        selector: impl Into<String>,
        pkcs8_der: Vec<u8>,
    ) -> Self {
        Self {
            domain: domain.into(),
            selector: selector.into(),
            key: DkimKey::Ed25519Pkcs8Der(pkcs8_der),
        }
    }

    /// Creates a configuration from an RSA PKCS#8 DER key (RSA-SHA256).
    pub fn rsa_pkcs8_der(
        domain: impl Into<String>,
        selector: impl Into<String>,
        pkcs8_der: Vec<u8>,
    ) -> Self {
        Self {
            domain: domain.into(),
            selector: selector.into(),
            key: DkimKey::RsaPkcs8Der(pkcs8_der),
        }
    }

    /// Creates a configuration from an RSA PKCS#8 PEM key (RSA-SHA256).
    pub fn rsa_pkcs8_pem(
        domain: impl Into<String>,
        selector: impl Into<String>,
        pkcs8_pem: impl Into<String>,
    ) -> Self {
        Self {
            domain: domain.into(),
            selector: selector.into(),
            key: DkimKey::RsaPkcs8Pem(pkcs8_pem.into()),
        }
    }

    /// Signs the RFC 5322 message bytes and returns the signed message: the
    /// `DKIM-Signature` header followed by the original content. This mirrors
    /// what an SMTP client does before `DATA`, and is applied last so the
    /// signature covers the final bytes.
    pub fn sign(&self, content: &[u8]) -> Result<Vec<u8>, AgMailError> {
        match &self.key {
            DkimKey::Ed25519Pkcs8Der(der) => {
                let key = Ed25519Key::from_pkcs8_der(der).map_err(dkim_err)?;
                self.sign_with(key, content)
            }
            DkimKey::RsaPkcs8Der(der) => {
                let pk = PrivatePkcs8KeyDer::from(der.as_slice());
                let key =
                    RsaKey::<Sha256>::from_key_der(PrivateKeyDer::Pkcs8(pk)).map_err(dkim_err)?;
                self.sign_with(key, content)
            }
            DkimKey::RsaPkcs8Pem(pem) => {
                let pk = PrivatePkcs8KeyDer::from_pem_slice(pem.as_bytes()).map_err(dkim_err)?;
                let key =
                    RsaKey::<Sha256>::from_key_der(PrivateKeyDer::Pkcs8(pk)).map_err(dkim_err)?;
                self.sign_with(key, content)
            }
        }
    }

    /// Builds the signer from a concrete key and produces the signed bytes.
    fn sign_with<K: SigningKey>(&self, key: K, content: &[u8]) -> Result<Vec<u8>, AgMailError> {
        let signer = DkimSigner::from_key(key)
            .domain(&self.domain)
            .selector(&self.selector)
            .headers(SIGNED_HEADERS);
        let signature = signer.sign(content).map_err(dkim_err)?;
        let mut signed = Vec::with_capacity(content.len() + 256);
        signature.write_header(&mut signed);
        signed.extend_from_slice(content);
        Ok(signed)
    }
}

fn dkim_err(e: impl std::fmt::Display) -> AgMailError {
    AgMailError::Dkim(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ed25519_key() -> Vec<u8> {
        Ed25519Key::generate_pkcs8().expect("key generation succeeds")
    }

    // Throwaway 2048-bit RSA key for tests only (PKCS#8). Not used anywhere
    // else and never published; it exercises the RSA-SHA256 signing path.
    const TEST_RSA_PKCS8_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvAIBADANBgkqhkiG9w0BAQEFAASCBKYwggSiAgEAAoIBAQDJD13O3/aRkjsR
NuoNGJndkz6w93+61Foz5RhR7LaZw8ct0j+AQTwiEbSOr3Bgbv2lhEywkoWIFKfB
xllCnr0wB9j3y47F3XdBgBVPH1PpvPeLz6ZJFC3u48gStfxJI592Fb468oefIGMt
Opv7HYFGbLR7a4tynP7tPaZT7vrLM8uf648Vx1zONXkRMUoBTW320vlXwlCEvbHk
eKeUUKUUw8IWCbwfoLwEequn9WZisfjerCWoCmkiLV+2wTtrxUn6EuAcY9GWaP2m
E+ZWXxkyn6mqesxx11UmASHUbNUyNfqak4i60pfAzt1M6aPGCiUokL8G0+m5xGaa
jDDjdaFRAgMBAAECggEAC5U/G7NBP6YNkTaBPbmHaTmBssUiaplv3CYgTF1zUFjU
G2idtApGECYmGVOD+UlBo3jDiIdbaiFbAAXI2AbAOlF0a2OirTVe2MMbzynsThae
6gL/pt2Czyy4FrRdZTsINW/HnJqi66zca9NQ38LVkxChhev2XdcrtFAROkwF7qF6
KYTAsoSpgkqqw+5Hg6cHaQHOK7B0pyO8GkWCbSycreZpty2dbiAAo5EMF5ZUeRRC
SqtqYIyN0wrXWeNszOZh1vMk5oiDgFII89nhHDr1UzP84zBOHJI0S4tAVtrTn2mm
uE1x7ocms82w6LjGELCeGIBT/qAW3jt9JmrGP9iPAQKBgQD8UBHufrpjMaeMjjrW
Sd2Pen1yHIUlCCvEn8Zi62PNB7cjG7hAUQtzkaC3/i1hHM++348sU3UiLBtCRlig
0vbdGIS6RaPMvv9AM3mtZzUdZV4a4xbYiTgSjkN+94KRVud6AHXFs2NMfuCB2Onf
3/dcoQ0RWL76Gg1ejgLGwUm7AQKBgQDL/43gBQsVSZEV67adFPdRJROJyGy5Zi0/
BwMaZqb0uRR2J0El6pUcz9DM1xwSp1DR9ayffexO/mZEfvAssp52fJ718+C+tGo+
JfTmmXaFyjWHbVQtJN/nX+nRlwkb8RZTadl3g36NIwt+mC564Uj1/h9MZpScvrlZ
WCxtKe92UQJ/D8TxWfaRr6Fq114MT1hOgKknLfqaV7YrBh72RzhvBGF3ELqqyVpH
XPG9GcLJE0i5szRFvn940qnvCjVbVLHau/bMVNgFZMaLLXgKIsxQhYXQW93Vj7MD
ziUWEtllT7Aex4E/2xtbGWzUm0jzVQqqax+lsW/af8chDsfRZJqwAQKBgQDFr4e4
IIsF/P2ZJYwYmxmzy98sNUqPO6H7Ivtx2SpPmw9mFFJAwRvxCL5J0ucEeJmMbavz
/x5ByqwmEnECj7GaFtggdk5D4CfYxAWYdfzyV+sZquvYt5Q27q5yig+swSumd7vi
4k1/zgc0jZr1P13p7HI+ImY/o1+QYi7jNrE0kQKBgQCswicqZ/w9FNlfdJQyFos5
Wlu4HR/OzN95ko8hhCqX22s2h1eR+N+0uAPZ8WnDDqs9tnepCXI0eGQStU7VToXq
bPR7xJgRIhO+q0Zvqre9v81EvF0KqdtBIg5UP52hcPTY11SSDbZ2xTX4pLtyWFJC
Xi6iNG5RqHMCBtI7A0uu8w==
-----END PRIVATE KEY-----"#;

    fn signed_text(cfg: &DkimConfig) -> String {
        let content = b"From: a@example.com\r\nSubject: hi\r\n\r\nbody\r\n";
        let signed = cfg.sign(content).expect("signing succeeds");
        assert!(signed.ends_with(content), "original content preserved");
        String::from_utf8_lossy(&signed).into_owned()
    }

    #[test]
    fn ed25519_sign_prepends_signature() {
        let cfg = DkimConfig::ed25519("example.com", "ag2026", ed25519_key());
        let text = signed_text(&cfg);
        assert!(text.starts_with("DKIM-Signature:"));
        assert!(text.contains("d=example.com"));
        assert!(text.contains("s=ag2026"));
        assert!(text.contains("a=ed25519-sha256"));
    }

    #[test]
    fn rsa_sign_prepends_signature() {
        let cfg = DkimConfig::rsa_pkcs8_pem("example.com", "ag2026", TEST_RSA_PKCS8_PEM);
        let text = signed_text(&cfg);
        assert!(text.starts_with("DKIM-Signature:"));
        assert!(text.contains("d=example.com"));
        assert!(text.contains("a=rsa-sha256"));
    }

    #[test]
    fn ed25519_rejects_invalid_key() {
        let cfg = DkimConfig::ed25519("example.com", "ag2026", vec![0u8; 8]);
        assert!(matches!(cfg.sign(b"x"), Err(AgMailError::Dkim(_))));
    }

    #[test]
    fn rsa_rejects_invalid_pem() {
        let cfg = DkimConfig::rsa_pkcs8_pem("example.com", "ag2026", "not a pem");
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
