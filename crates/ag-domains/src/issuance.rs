//! Certificate issuance orchestration: SAN-set deduplication and rate limits.
//!
//! Implements the rate-limit protections from the blueprint section 13.4:
//! deduplication by SAN set (so concurrent requests for the same certificate do
//! not start two ACME orders) and per-registered-domain issuance counters (so
//! the control plane stays under the CA's "certificates per registered domain"
//! limit). This is native logic with no external dependency; the actual ACME
//! order is performed by an injected [`CertIssuer`] (production wraps
//! `crate::acme::renewal::issue`; tests use a mock).

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use async_trait::async_trait;

/// An issued certificate: PEM chain + PEM private key.
///
/// Mirrors `acme::renewal::IssuedCert` without coupling this module to the
/// `acme` Cargo feature; the ACME issuer maps one into the other.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuedCertificate {
    /// Certificate chain in PEM format (leaf + intermediates).
    pub cert_chain_pem: String,
    /// Private key in PEM format.
    pub private_key_pem: String,
}

/// Errors from the issuance orchestration.
#[derive(Debug, thiserror::Error)]
pub enum IssuanceError {
    /// The SAN set was empty.
    #[error("empty SAN set")]
    EmptySanSet,
    /// An order for the same SAN set is already running.
    #[error("an order for the same SAN set is already in flight")]
    DuplicateInFlight,
    /// The per-registered-domain issuance limit was reached.
    #[error("issuance rate limit reached for '{domain}' ({count}/{max} in window)")]
    RateLimited {
        /// Registered domain that hit the limit.
        domain: String,
        /// Current count in the window.
        count: u32,
        /// Configured maximum.
        max: u32,
    },
    /// The underlying issuer failed.
    #[error("issuer error: {0}")]
    Issuer(String),
}

/// Produces a certificate for a set of subject alternative names.
///
/// The production implementation wraps the ACME client; tests inject a mock.
#[async_trait]
pub trait CertIssuer: Send + Sync {
    /// Issues a certificate covering exactly `sans`.
    async fn issue(&self, sans: &[String]) -> Result<IssuedCertificate, IssuanceError>;
}

/// Canonical deduplication key for a SAN set: lowercased, de-duplicated, sorted.
///
/// Order- and case-independent, so `[a, B]` and `[b, a]` map to the same key.
pub fn san_key(sans: &[String]) -> String {
    let mut norm: Vec<String> = sans
        .iter()
        .map(|s| s.trim().trim_end_matches('.').to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    norm.sort();
    norm.dedup();
    norm.join(",")
}

/// Registered domain (eTLD+1) used as the per-domain counting key.
///
/// Shares the single eTLD+1 source of the crate ([`crate::registrable`]), so the
/// rate-limit key matches `hostname`'s apex/subdomain classification. PSL-correct
/// with the `psl` feature; a best-effort two-label heuristic otherwise.
fn counting_key(san: &str) -> String {
    crate::registrable::registrable_domain(san)
}

/// Tracks in-flight SAN sets and per-registered-domain issuance counts.
///
/// `max_per_domain` mirrors the CA's certificates-per-registered-domain window
/// limit; counts increment on successful issuance only (matching how CAs count
/// issued certificates), while the in-flight set prevents duplicate concurrent
/// orders.
#[derive(Debug)]
pub struct IssuanceLimiter {
    max_per_domain: u32,
    state: Mutex<LimiterState>,
}

#[derive(Default)]
struct LimiterState {
    in_flight: HashSet<String>,
    issued_counts: HashMap<String, u32>,
}

impl std::fmt::Debug for LimiterState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LimiterState")
            .field("in_flight", &self.in_flight.len())
            .field("tracked_domains", &self.issued_counts.len())
            .finish()
    }
}

impl IssuanceLimiter {
    /// Creates a limiter that allows `max_per_domain` issued certificates per
    /// registered domain.
    pub fn new(max_per_domain: u32) -> Self {
        Self {
            max_per_domain,
            state: Mutex::new(LimiterState::default()),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, LimiterState> {
        self.state.lock().expect("issuance limiter lock poisoned")
    }

    /// Reserves an in-flight slot for `sans` after checking dedup and rate.
    ///
    /// Returns the SAN key to pass to [`IssuanceLimiter::release`].
    fn reserve(&self, sans: &[String], key: &str) -> Result<(), IssuanceError> {
        let mut st = self.lock();
        if st.in_flight.contains(key) {
            return Err(IssuanceError::DuplicateInFlight);
        }
        for san in sans {
            let domain = counting_key(san);
            let count = st.issued_counts.get(&domain).copied().unwrap_or(0);
            if count >= self.max_per_domain {
                return Err(IssuanceError::RateLimited {
                    domain,
                    count,
                    max: self.max_per_domain,
                });
            }
        }
        st.in_flight.insert(key.to_owned());
        Ok(())
    }

    /// Records a successful issuance, incrementing per-domain counters once per
    /// distinct registered domain in the SAN set.
    fn record_success(&self, sans: &[String]) {
        let mut st = self.lock();
        let mut seen = HashSet::new();
        for san in sans {
            let domain = counting_key(san);
            if seen.insert(domain.clone()) {
                *st.issued_counts.entry(domain).or_insert(0) += 1;
            }
        }
    }

    /// Releases the in-flight reservation for `key`.
    fn release(&self, key: &str) {
        self.lock().in_flight.remove(key);
    }

    /// Current issued count for a registered domain (test/diagnostics).
    pub fn issued_count(&self, registered_domain: &str) -> u32 {
        self.lock()
            .issued_counts
            .get(&counting_key(registered_domain))
            .copied()
            .unwrap_or(0)
    }
}

/// Issues a certificate for `sans`, honoring deduplication and rate limits.
///
/// Reserves an in-flight slot (rejecting duplicates and rate-limited domains),
/// delegates to `issuer`, increments per-domain counters on success, and always
/// releases the in-flight slot.
pub async fn issue_certificate(
    limiter: &IssuanceLimiter,
    issuer: &dyn CertIssuer,
    sans: &[String],
) -> Result<IssuedCertificate, IssuanceError> {
    if sans.iter().all(|s| s.trim().is_empty()) {
        return Err(IssuanceError::EmptySanSet);
    }
    let key = san_key(sans);
    limiter.reserve(sans, &key)?;

    let result = issuer.issue(sans).await;

    if result.is_ok() {
        limiter.record_success(sans);
    }
    limiter.release(&key);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockIssuer {
        calls: AtomicUsize,
        fail: bool,
    }
    impl MockIssuer {
        fn ok() -> Self {
            Self {
                calls: AtomicUsize::new(0),
                fail: false,
            }
        }
        fn failing() -> Self {
            Self {
                calls: AtomicUsize::new(0),
                fail: true,
            }
        }
    }
    #[async_trait]
    impl CertIssuer for MockIssuer {
        async fn issue(&self, sans: &[String]) -> Result<IssuedCertificate, IssuanceError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.fail {
                return Err(IssuanceError::Issuer("boom".to_owned()));
            }
            Ok(IssuedCertificate {
                cert_chain_pem: format!("CERT for {}", san_key(sans)),
                private_key_pem: "KEY".to_owned(),
            })
        }
    }

    #[test]
    fn san_key_is_order_and_case_independent() {
        let a = san_key(&["B.example.com".to_owned(), "a.example.com".to_owned()]);
        let b = san_key(&["a.example.com".to_owned(), "b.example.com.".to_owned()]);
        assert_eq!(a, b);
        assert_eq!(a, "a.example.com,b.example.com");
    }

    #[test]
    fn counting_key_strips_wildcard_and_keeps_etld1() {
        assert_eq!(counting_key("*.example.com"), "example.com");
        assert_eq!(counting_key("api.example.com"), "example.com");
        assert_eq!(counting_key("example.com"), "example.com");
    }

    #[tokio::test]
    async fn issues_and_counts_success() {
        let limiter = IssuanceLimiter::new(5);
        let issuer = MockIssuer::ok();
        let cert = issue_certificate(&limiter, &issuer, &["api.example.com".to_owned()])
            .await
            .unwrap();
        assert!(cert.cert_chain_pem.contains("api.example.com"));
        assert_eq!(limiter.issued_count("example.com"), 1);
    }

    #[tokio::test]
    async fn empty_san_set_rejected() {
        let limiter = IssuanceLimiter::new(5);
        let issuer = MockIssuer::ok();
        let err = issue_certificate(&limiter, &issuer, &["  ".to_owned()])
            .await
            .unwrap_err();
        assert!(matches!(err, IssuanceError::EmptySanSet));
        assert_eq!(issuer.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn duplicate_in_flight_rejected() {
        let limiter = IssuanceLimiter::new(5);
        let key = san_key(&["api.example.com".to_owned()]);
        limiter
            .reserve(&["api.example.com".to_owned()], &key)
            .unwrap();
        // A second reserve for the same key fails until released.
        let err = limiter
            .reserve(&["api.example.com".to_owned()], &key)
            .unwrap_err();
        assert!(matches!(err, IssuanceError::DuplicateInFlight));
        limiter.release(&key);
        assert!(limiter
            .reserve(&["api.example.com".to_owned()], &key)
            .is_ok());
    }

    #[tokio::test]
    async fn rate_limit_enforced_per_domain() {
        let limiter = IssuanceLimiter::new(2);
        let issuer = MockIssuer::ok();
        for sub in ["a.example.com", "b.example.com"] {
            issue_certificate(&limiter, &issuer, &[sub.to_owned()])
                .await
                .unwrap();
        }
        assert_eq!(limiter.issued_count("example.com"), 2);
        let err = issue_certificate(&limiter, &issuer, &["c.example.com".to_owned()])
            .await
            .unwrap_err();
        assert!(matches!(err, IssuanceError::RateLimited { max: 2, .. }));
    }

    #[tokio::test]
    async fn failure_does_not_count_and_releases() {
        let limiter = IssuanceLimiter::new(2);
        let issuer = MockIssuer::failing();
        let err = issue_certificate(&limiter, &issuer, &["api.example.com".to_owned()])
            .await
            .unwrap_err();
        assert!(matches!(err, IssuanceError::Issuer(_)));
        // Not counted, and the in-flight slot was released (can retry).
        assert_eq!(limiter.issued_count("example.com"), 0);
        let ok_issuer = MockIssuer::ok();
        assert!(
            issue_certificate(&limiter, &ok_issuer, &["api.example.com".to_owned()])
                .await
                .is_ok()
        );
    }
}
