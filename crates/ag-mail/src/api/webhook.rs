//! Signed outbound webhooks for the managed email API (Phase 4.6-C, DEBT-021).
//!
//! Each webhook delivery is signed with HMAC-SHA256 over
//! `{id}.{timestamp}.{payload}` using a per-endpoint secret, and the signature
//! travels in a `v1,<base64>` header alongside the id and timestamp. Receivers
//! recompute the MAC and compare in constant time, rejecting deliveries whose
//! timestamp is outside a tolerance window (replay protection). This is the
//! widely-used scheme so existing receiver libraries interoperate.
//!
//! Verification uses the MAC's own constant-time check; a header may carry
//! several space-separated signatures (e.g. during a secret rotation) and the
//! delivery is accepted if any of them matches.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Errors from signing-secret parsing or signature verification.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum WebhookError {
    /// The signing secret could not be decoded or was empty.
    #[error("invalid webhook signing secret")]
    InvalidSecret,
    /// No signature in the header matched.
    #[error("webhook signature does not match")]
    Mismatch,
    /// The signed timestamp was outside the tolerance window.
    #[error("webhook timestamp outside tolerance")]
    Expired,
}

/// A webhook signing secret (the HMAC key).
#[derive(Clone)]
pub struct SigningSecret {
    key: Vec<u8>,
}

impl std::fmt::Debug for SigningSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SigningSecret(<redacted>)")
    }
}

impl SigningSecret {
    /// Wraps raw key bytes.
    pub fn from_key(key: Vec<u8>) -> Self {
        Self { key }
    }

    /// Parses a `whsec_<base64>` secret (the `whsec_` prefix is optional).
    pub fn parse(secret: &str) -> Result<Self, WebhookError> {
        let b64 = secret.strip_prefix("whsec_").unwrap_or(secret);
        let key = STANDARD
            .decode(b64)
            .map_err(|_| WebhookError::InvalidSecret)?;
        if key.is_empty() {
            return Err(WebhookError::InvalidSecret);
        }
        Ok(Self { key })
    }

    /// Renders the secret in `whsec_<base64>` form (return it to the API client
    /// once, at endpoint creation).
    pub fn to_secret_string(&self) -> String {
        format!("whsec_{}", STANDARD.encode(&self.key))
    }

    fn mac(&self, id: &str, timestamp: i64, payload: &[u8]) -> HmacSha256 {
        // HMAC accepts a key of any length, so this never fails.
        let mut mac = HmacSha256::new_from_slice(&self.key).expect("HMAC key length is unbounded");
        mac.update(id.as_bytes());
        mac.update(b".");
        mac.update(timestamp.to_string().as_bytes());
        mac.update(b".");
        mac.update(payload);
        mac
    }
}

/// Signs `{id}.{timestamp}.{payload}`, returning a `v1,<base64>` signature.
pub fn sign(secret: &SigningSecret, id: &str, timestamp: i64, payload: &[u8]) -> String {
    let signature = secret.mac(id, timestamp, payload).finalize().into_bytes();
    format!("v1,{}", STANDARD.encode(signature))
}

/// Verifies a signature `header` against the payload.
///
/// Rejects if the signed `timestamp` is more than `tolerance_secs` away from
/// `now_secs`, or if no `v1,` signature in the (space-separated) header matches.
pub fn verify(
    secret: &SigningSecret,
    id: &str,
    timestamp: i64,
    payload: &[u8],
    header: &str,
    now_secs: i64,
    tolerance_secs: i64,
) -> Result<(), WebhookError> {
    if (now_secs - timestamp).abs() > tolerance_secs {
        return Err(WebhookError::Expired);
    }
    for candidate in header.split_whitespace() {
        let encoded = candidate.strip_prefix("v1,").unwrap_or(candidate);
        let Ok(signature) = STANDARD.decode(encoded) else {
            continue;
        };
        // `verify_slice` compares in constant time.
        if secret
            .mac(id, timestamp, payload)
            .verify_slice(&signature)
            .is_ok()
        {
            return Ok(());
        }
    }
    Err(WebhookError::Mismatch)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secret() -> SigningSecret {
        SigningSecret::from_key(b"super-secret-signing-key-0123456789".to_vec())
    }

    #[test]
    fn sign_then_verify_round_trip() {
        let s = secret();
        let payload = br#"{"type":"email.delivered","data":{"id":"e1"}}"#;
        let sig = sign(&s, "msg_1", 1_700_000_000, payload);
        assert!(sig.starts_with("v1,"));
        assert_eq!(
            verify(
                &s,
                "msg_1",
                1_700_000_000,
                payload,
                &sig,
                1_700_000_010,
                300
            ),
            Ok(())
        );
    }

    #[test]
    fn tampered_payload_fails() {
        let s = secret();
        let sig = sign(&s, "msg_1", 1_700_000_000, b"original");
        assert_eq!(
            verify(
                &s,
                "msg_1",
                1_700_000_000,
                b"tampered",
                &sig,
                1_700_000_000,
                300
            ),
            Err(WebhookError::Mismatch)
        );
    }

    #[test]
    fn wrong_id_or_secret_fails() {
        let s = secret();
        let sig = sign(&s, "msg_1", 1_700_000_000, b"p");
        assert_eq!(
            verify(&s, "msg_2", 1_700_000_000, b"p", &sig, 1_700_000_000, 300),
            Err(WebhookError::Mismatch)
        );
        let other = SigningSecret::from_key(b"different-key-aaaaaaaaaaaaaaaaaaaa".to_vec());
        assert_eq!(
            verify(
                &other,
                "msg_1",
                1_700_000_000,
                b"p",
                &sig,
                1_700_000_000,
                300
            ),
            Err(WebhookError::Mismatch)
        );
    }

    #[test]
    fn expired_timestamp_fails() {
        let s = secret();
        let sig = sign(&s, "msg_1", 1_700_000_000, b"p");
        // 10 minutes later with a 5-minute tolerance.
        assert_eq!(
            verify(&s, "msg_1", 1_700_000_000, b"p", &sig, 1_700_000_600, 300),
            Err(WebhookError::Expired)
        );
    }

    #[test]
    fn multiple_signatures_one_matches() {
        let s = secret();
        let good = sign(&s, "msg_1", 1_700_000_000, b"p");
        let header = format!("v1,AAAA {good} v1,BBBB");
        assert_eq!(
            verify(
                &s,
                "msg_1",
                1_700_000_000,
                b"p",
                &header,
                1_700_000_000,
                300
            ),
            Ok(())
        );
    }

    #[test]
    fn parse_and_render_secret_round_trip() {
        let s = secret();
        let rendered = s.to_secret_string();
        assert!(rendered.starts_with("whsec_"));
        let parsed = SigningSecret::parse(&rendered).unwrap();
        // Same key => same signature.
        assert_eq!(sign(&s, "i", 1, b"x"), sign(&parsed, "i", 1, b"x"));
    }

    #[test]
    fn parse_rejects_garbage() {
        assert_eq!(
            SigningSecret::parse("whsec_!!!notbase64").unwrap_err(),
            WebhookError::InvalidSecret
        );
        assert_eq!(
            SigningSecret::parse("").unwrap_err(),
            WebhookError::InvalidSecret
        );
    }

    #[test]
    fn debug_redacts_key() {
        let rendered = format!("{:?}", secret());
        assert!(rendered.contains("redacted"));
        assert!(!rendered.contains("super-secret"));
    }
}
