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
    crypto::{Ed25519Key, RsaKey, Sha256, SigningKey},
    dkim::{Canonicalization, DkimSigner},
    headers::HeaderWriter,
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
        // Pin relaxed/relaxed explicitly: the pre-upgrade stack used it as its
        // implicit default, and relying on a library default would make the
        // wire format a silent dependency on a future mail-auth change.
        let signer = DkimSigner::from_key(key)
            .domain(&self.domain)
            .selector(&self.selector)
            .headers(SIGNED_HEADERS)
            .header_canonicalization(Canonicalization::Relaxed)
            .body_canonicalization(Canonicalization::Relaxed);
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

    // ---- cross-version known-answer vector ---------------------------------
    //
    // Recorded from mail-auth 0.9 (the pre-upgrade stack) for the fixed key and
    // message below, with domain `send.example` and selector `sel`. DKIM is
    // deterministic for a fixed key and timestamp, so the new stack must still
    // produce a signature that verifies, and must also verify this old one.
    // This guards the 0.9 -> 0.15 signing/verifying migration against silent
    // canonicalization or algorithm changes.

    const KA_ED25519_PKCS8_DER: &[u8] = &[
        48, 81, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 17, 3, 231, 95, 91, 66, 162, 59,
        122, 193, 2, 82, 53, 179, 194, 23, 234, 104, 165, 3, 72, 121, 47, 216, 189, 191, 233, 84,
        177, 241, 201, 22, 129, 33, 0, 162, 108, 193, 14, 91, 157, 191, 30, 160, 163, 99, 195, 193,
        250, 213, 2, 254, 98, 249, 42, 58, 5, 205, 255, 24, 54, 160, 83, 43, 34, 187, 196,
    ];
    /// `p=` tag published in `sel._domainkey.send.example`: the public key of
    /// `KA_ED25519_PKCS8_DER`.
    const KA_PUBLIC_KEY_B64: &str = "omzBDludvx6go2PDwfrVAv5i+So6Bc3/GDagUysiu8Q=";
    const KA_MESSAGE: &[u8] = b"From: from@send.example\r\n\
        To: rcpt@example.com\r\n\
        Subject: hi\r\n\
        Date: Tue, 10 Nov 2026 12:00:00 +0000\r\n\
        Message-ID: <ka@send.example>\r\n\
        MIME-Version: 1.0\r\n\
        Content-Type: text/plain\r\n\
        \r\n\
        body\r\n";
    const KA_SIGNATURE_0_9: &str =
        "DKIM-Signature: v=1; a=ed25519-sha256; s=sel; d=send.example; c=relaxed/relaxed;\r\n\
        \th=Content-Type:MIME-Version:Message-ID:Date:Subject:To:From; t=1795308595; bh=C\r\n\
        \tk5SoRNWUpSR4X0COv7R5ub2pUTtl6xz4dTFz++ji4M=; b=q1w8p3SqEoPW6XRva6KWYWCvisK5\r\n\
        \t+5J7QpxYMRciZrVk3b+fezU1PXXQiAhKwcaPW4fPhjAocFeB7gVnfsdFDQ==;\r\n";

    /// Minimal DNS cache that answers the single `_domainkey` TXT query with
    /// the fixed public key. Every other record type stays uncached (and is
    /// never queried by DKIM verification for this signature).
    mod ka_dns {
        use std::{
            borrow::Borrow,
            collections::HashMap,
            hash::Hash,
            sync::{Arc, Mutex},
        };

        use mail_auth::dkim::DomainKey;
        use mail_auth::dns::{DnsCache, NoCache, ResolverCache, TxtRecord, TxtRecordParser};

        pub struct TxtCache(Mutex<HashMap<Box<str>, TxtRecord>>);

        impl TxtCache {
            pub fn new(key: &str, value: TxtRecord) -> Self {
                Self(Mutex::new(HashMap::from([(key.into(), value)])))
            }
        }

        impl ResolverCache<Box<str>, TxtRecord> for TxtCache {
            fn get<Q>(&self, name: &Q) -> Option<TxtRecord>
            where
                Box<str>: Borrow<Q>,
                Q: Hash + Eq + ?Sized,
            {
                self.0.lock().unwrap().get(name).cloned()
            }

            fn remove<Q>(&self, name: &Q) -> Option<TxtRecord>
            where
                Box<str>: Borrow<Q>,
                Q: Hash + Eq + ?Sized,
            {
                self.0.lock().unwrap().remove(name)
            }

            fn insert(&self, key: Box<str>, value: TxtRecord, _valid_until: std::time::Instant) {
                self.0.lock().unwrap().insert(key, value);
            }
        }

        pub struct FixedDns {
            txt: TxtCache,
        }

        impl FixedDns {
            pub fn with_public_key(record: &str, name: &str) -> Self {
                let key = <DomainKey as TxtRecordParser>::parse(record.as_bytes())
                    .expect("fixed key record parses");
                Self {
                    txt: TxtCache::new(name, TxtRecord::DomainKey(Arc::new(key))),
                }
            }
        }

        impl DnsCache for FixedDns {
            type Txt = TxtCache;
            type Mx = NoCache;
            type Ipv4 = NoCache;
            type Ipv6 = NoCache;
            type Ptr = NoCache;

            fn txt(&self) -> Option<&Self::Txt> {
                Some(&self.txt)
            }
            fn mx(&self) -> Option<&Self::Mx> {
                None
            }
            fn ipv4(&self) -> Option<&Self::Ipv4> {
                None
            }
            fn ipv6(&self) -> Option<&Self::Ipv6> {
                None
            }
            fn ptr(&self) -> Option<&Self::Ptr> {
                None
            }
        }
    }

    #[tokio::test]
    async fn dkim_known_answer_old_signature_verifies_and_new_signature_verifies() {
        use ka_dns::FixedDns;
        use mail_auth::dns::Parameters;
        use mail_auth::{AuthenticatedMessage, DkimResult, MessageAuthenticator};

        let cfg = DkimConfig::ed25519("send.example", "sel", KA_ED25519_PKCS8_DER.to_vec());

        // The same fixed key and message must still be signable by the new stack.
        let new_signed = cfg
            .sign(KA_MESSAGE)
            .expect("new stack signs the fixed vector");
        assert!(new_signed.starts_with(b"DKIM-Signature:"));

        let record = format!("v=DKIM1; k=ed25519; p={KA_PUBLIC_KEY_B64}");
        let dns = FixedDns::with_public_key(&record, "sel._domainkey.send.example.");
        let authenticator = MessageAuthenticator::new_system_conf().expect("resolver");

        // A signature produced by mail-auth 0.9 verifies under 0.15: no silent
        // change to canonicalization, `a=` algorithm or `bh=` body hashing.
        let mut old_signed = KA_SIGNATURE_0_9.as_bytes().to_vec();
        old_signed.extend_from_slice(KA_MESSAGE);
        let old_message =
            AuthenticatedMessage::parse(&old_signed).expect("parse old signed message");
        let old_results = authenticator
            .verify_dkim(Parameters::new(&old_message).with_cache(&dns))
            .await;
        assert_eq!(old_results.len(), 1, "one signature from the old vector");
        assert_eq!(
            old_results[0].result(),
            &DkimResult::Pass,
            "pre-upgrade DKIM signature must still verify"
        );

        // A signature produced by the new stack also verifies with the same
        // verifier and published key.
        let new_message =
            AuthenticatedMessage::parse(&new_signed).expect("parse new signed message");
        let new_results = authenticator
            .verify_dkim(Parameters::new(&new_message).with_cache(&dns))
            .await;
        assert_eq!(new_results.len(), 1, "one signature from the new stack");
        assert_eq!(
            new_results[0].result(),
            &DkimResult::Pass,
            "new DKIM signature must verify"
        );
    }

    // ---- independent verification with dkimpy -------------------------------
    //
    // The cross-version test above is migration regression coverage: both
    // versions share canonicalization, hashing and header-writing lineage
    // with the signer, so it is not independent evidence. This test runs an
    // external implementation (dkimpy) over the embedded 0.9 vector and a
    // fresh 0.15 signature. It is #[ignore] because dkimpy is not a Cargo
    // dependency; the exact invocation and recorded output are in
    // docs/security/DKIM_INDEPENDENT_VERIFICATION.md.
    #[test]
    #[ignore = "requires external dkimpy + pynacl; see docs/security/DKIM_INDEPENDENT_VERIFICATION.md"]
    fn dkim_signatures_verify_with_dkimpy() {
        use std::{
            path::Path,
            process::{Command, Stdio},
        };

        let cfg = DkimConfig::ed25519("send.example", "sel", KA_ED25519_PKCS8_DER.to_vec());
        let fresh = cfg
            .sign(KA_MESSAGE)
            .expect("new stack signs the fixed vector");

        let mut old = KA_SIGNATURE_0_9.as_bytes().to_vec();
        old.extend_from_slice(KA_MESSAGE);

        let dir = std::env::temp_dir().join(format!("ag-mail-dkim-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temporary directory");
        let old_path = dir.join("old-0-9.eml");
        let fresh_path = dir.join("fresh-0-15.eml");
        std::fs::write(&old_path, &old).expect("write old vector");
        std::fs::write(&fresh_path, &fresh).expect("write fresh signature");

        let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/dkim_verify.py");
        assert!(
            script.exists(),
            "dkimpy verifier script missing: {}",
            script.display()
        );

        let output = Command::new("python3")
            .arg(&script)
            .arg(&old_path)
            .arg(&fresh_path)
            .stdin(Stdio::null())
            .output()
            .expect("run dkimpy verifier");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("{stdout}");
        assert!(
            output.status.success(),
            "dkimpy rejected a signature: {stdout}{stderr}"
        );
    }
}
