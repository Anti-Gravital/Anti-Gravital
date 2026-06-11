//! End-to-end DNS-01 issuance orchestration via the provider adapter SDK.
//!
//! Wildcard certificates (`*.example.com`) require the DNS-01 challenge: the CA
//! validates control by reading a `_acme-challenge` TXT record. This module
//! wires that flow on top of the declarative [`ZoneAdapter`] seam: it publishes
//! the challenge TXT records with [`crate::provider::sdk::diff`] + `apply`,
//! finalizes the order, and always tears the ephemeral challenge records down
//! afterwards — success or failure.
//!
//! Manual DNS-01 (publish the TXT record yourself, then issue) stays fully
//! supported through [`crate::acme::challenge`]; this is the *automated* path
//! used only when a provider adapter is configured.
//!
//! The ACME interaction is abstracted behind [`Dns01OrderDriver`] so the
//! orchestration is deterministic and unit-tested with a mock order and a mock
//! adapter, with no network. The live driver against Let's Encrypt staging is a
//! credentialed `#[ignore]` end-to-end test (issue #87).

use async_trait::async_trait;
use tracing::warn;

use crate::{
    error::AgDomainsError,
    issuance::IssuedCertificate,
    provider::sdk::{diff, ZoneAdapter},
    record::{DnsRecord, DnsRecordSpec, RecordType},
};

/// Prefix of the DNS-01 challenge record name (RFC 8555 section 8.4).
const ACME_CHALLENGE_PREFIX: &str = "_acme-challenge.";

/// One `_acme-challenge` TXT record an ACME order needs published.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dns01Desired {
    /// Full record name, e.g. `_acme-challenge.example.com`.
    pub txt_name: String,
    /// The key-authorization digest the CA will read.
    pub txt_value: String,
}

impl Dns01Desired {
    /// The declarative record spec to publish for this challenge (TTL 60s, the
    /// record is short-lived and removed after validation).
    fn to_spec(&self) -> DnsRecordSpec {
        DnsRecordSpec {
            name: self.txt_name.clone(),
            record_type: RecordType::Txt,
            content: self.txt_value.clone(),
            ttl: 60,
            proxied: false,
        }
    }
}

/// Abstracts the ACME side of a DNS-01 order so the orchestration stays
/// testable without a network or the `instant-acme` types.
///
/// The production implementation wraps an `instant_acme::Order` (created against
/// Let's Encrypt); tests inject a deterministic mock.
#[async_trait]
pub trait Dns01OrderDriver: Send {
    /// Begin the order and return the `_acme-challenge` TXT records to publish.
    async fn challenges(&mut self) -> Result<Vec<Dns01Desired>, AgDomainsError>;

    /// After the TXT records are published and propagated, notify the CA, wait
    /// for validation, finalize the CSR and download the certificate.
    async fn finalize(&mut self) -> Result<IssuedCertificate, AgDomainsError>;
}

/// Whether an observed record is an ACME DNS-01 challenge TXT record.
///
/// This is the managed scope for challenge publication: only `_acme-challenge`
/// TXT records are reconciled, so unrelated zone records are never touched (the
/// [`diff`] deletion-scope contract).
fn is_acme_challenge_txt(record: &DnsRecord) -> bool {
    record.record_type == RecordType::Txt
        && record
            .name
            .trim_end_matches('.')
            .to_ascii_lowercase()
            .starts_with(ACME_CHALLENGE_PREFIX)
}

/// Issue a (wildcard or multi-SAN) certificate end to end via DNS-01.
///
/// Steps:
/// 1. Read the order's required `_acme-challenge` TXT records.
/// 2. Discover the zone and read only its existing `_acme-challenge` TXT
///    records (the managed scope).
/// 3. `diff` desired vs observed and `apply` the plan to publish the records.
/// 4. `finalize` the order.
/// 5. Always tear the `_acme-challenge` subtree down, removing the challenge
///    records whether issuance succeeded or failed.
///
/// The `_acme-challenge` subtree is treated as ephemeral and fully managed by
/// this orchestration: stale challenge records left by an earlier run are
/// reconciled away during publication, and every challenge record is removed at
/// teardown. The managed scope is strictly `_acme-challenge` TXT records, so no
/// other record in the zone is ever touched. (This is why cleanup is a teardown
/// rather than a generic rollback, which would restore stale challenge records.)
///
/// `base_domain` is the registrable domain whose zone hosts the challenge
/// records (e.g. `example.com` for `*.example.com`).
pub async fn issue_dns01_with_adapter<A, D>(
    adapter: &A,
    base_domain: &str,
    order: &mut D,
) -> Result<IssuedCertificate, AgDomainsError>
where
    A: ZoneAdapter + ?Sized,
    D: Dns01OrderDriver + ?Sized,
{
    let desired_challenges = order.challenges().await?;
    if desired_challenges.is_empty() {
        return Err(AgDomainsError::Acme(
            "ACME order produced no DNS-01 challenges".to_owned(),
        ));
    }

    let zone = adapter.discover(base_domain).await?;

    // Scope the diff to the `_acme-challenge` subtree so nothing else is touched.
    let observed = read_challenge_scope(adapter, &zone).await?;
    let desired: Vec<DnsRecordSpec> = desired_challenges
        .iter()
        .map(Dns01Desired::to_spec)
        .collect();

    let plan = diff(&desired, &observed);
    adapter.apply(&zone, &plan).await?;

    // Finalize, then always tear down the ephemeral challenge records.
    let result = order.finalize().await;
    teardown_challenge_scope(adapter, &zone).await;

    result
}

/// Read only the `_acme-challenge` TXT records in the zone (the managed scope).
async fn read_challenge_scope<A: ZoneAdapter + ?Sized>(
    adapter: &A,
    zone: &crate::provider::sdk::ZoneRef,
) -> Result<Vec<DnsRecord>, AgDomainsError> {
    Ok(adapter
        .read(zone)
        .await?
        .into_iter()
        .filter(is_acme_challenge_txt)
        .collect())
}

/// Best-effort removal of every `_acme-challenge` TXT record in the zone.
///
/// Never fails the issuance: errors are logged, since the certificate (if
/// issued) is already valid regardless of leftover challenge records.
async fn teardown_challenge_scope<A: ZoneAdapter + ?Sized>(
    adapter: &A,
    zone: &crate::provider::sdk::ZoneRef,
) {
    match read_challenge_scope(adapter, zone).await {
        Ok(remaining) => {
            // Empty desired set => the plan deletes every scoped record.
            let teardown = diff(&[], &remaining);
            if !teardown.is_empty() {
                if let Err(e) = adapter.apply(zone, &teardown).await {
                    warn!(
                        zone = zone.domain,
                        error = %e,
                        "failed to clean up DNS-01 challenge records"
                    );
                }
            }
        }
        Err(e) => warn!(
            zone = zone.domain,
            error = %e,
            "could not re-read zone to clean up DNS-01 challenge records"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::sdk::ProviderAdapter;
    use crate::provider::DnsProvider;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// In-memory `DnsProvider` for orchestration tests (no network).
    ///
    /// A cheaply-clonable handle sharing inner state, so the adapter and the
    /// mock order observe the same zone.
    #[derive(Clone)]
    struct MockProvider {
        inner: Arc<MockInner>,
    }

    struct MockInner {
        records: Mutex<HashMap<String, DnsRecord>>,
        seq: Mutex<u64>,
    }

    impl MockProvider {
        fn new() -> Self {
            Self {
                inner: Arc::new(MockInner {
                    records: Mutex::new(HashMap::new()),
                    seq: Mutex::new(0),
                }),
            }
        }

        fn seed(&self, rec: DnsRecord) {
            self.inner
                .records
                .lock()
                .unwrap()
                .insert(rec.id.clone(), rec);
        }

        fn names(&self) -> Vec<String> {
            let mut v: Vec<String> = self
                .inner
                .records
                .lock()
                .unwrap()
                .values()
                .map(|r| format!("{}={}", r.name, r.content))
                .collect();
            v.sort();
            v
        }
    }

    #[async_trait]
    impl DnsProvider for MockProvider {
        fn name(&self) -> &'static str {
            "mock"
        }
        async fn zone_id(&self, _domain: &str) -> Result<String, AgDomainsError> {
            Ok("zone-1".to_owned())
        }
        async fn list_records(&self, _zone_id: &str) -> Result<Vec<DnsRecord>, AgDomainsError> {
            Ok(self
                .inner
                .records
                .lock()
                .unwrap()
                .values()
                .cloned()
                .collect())
        }
        async fn create_record(
            &self,
            zone_id: &str,
            spec: &DnsRecordSpec,
        ) -> Result<DnsRecord, AgDomainsError> {
            let mut seq = self.inner.seq.lock().unwrap();
            *seq += 1;
            let id = format!("rec-{seq}");
            let rec = DnsRecord {
                id: id.clone(),
                zone_id: zone_id.to_owned(),
                name: spec.name.clone(),
                record_type: spec.record_type.clone(),
                content: spec.content.clone(),
                ttl: spec.ttl,
                proxied: spec.proxied,
            };
            self.inner.records.lock().unwrap().insert(id, rec.clone());
            Ok(rec)
        }
        async fn update_record(
            &self,
            _zone_id: &str,
            record_id: &str,
            spec: &DnsRecordSpec,
        ) -> Result<DnsRecord, AgDomainsError> {
            let mut records = self.inner.records.lock().unwrap();
            let rec = records
                .get_mut(record_id)
                .ok_or_else(|| AgDomainsError::RecordNotFound(record_id.to_owned()))?;
            rec.content = spec.content.clone();
            rec.ttl = spec.ttl;
            Ok(rec.clone())
        }
        async fn delete_record(
            &self,
            _zone_id: &str,
            record_id: &str,
        ) -> Result<(), AgDomainsError> {
            self.inner
                .records
                .lock()
                .unwrap()
                .remove(record_id)
                .map(|_| ())
                .ok_or_else(|| AgDomainsError::RecordNotFound(record_id.to_owned()))
        }
    }

    /// Mock ACME order: yields fixed challenges and a configurable finalize.
    struct MockOrder {
        challenges: Vec<Dns01Desired>,
        fail_finalize: bool,
        /// Records what the zone held at finalize time, to assert the challenge
        /// TXT records were actually published before the CA was notified.
        observed_at_finalize: Mutex<Option<Vec<String>>>,
        adapter_records: MockProvider,
    }

    #[async_trait]
    impl Dns01OrderDriver for MockOrder {
        async fn challenges(&mut self) -> Result<Vec<Dns01Desired>, AgDomainsError> {
            Ok(self.challenges.clone())
        }
        async fn finalize(&mut self) -> Result<IssuedCertificate, AgDomainsError> {
            *self.observed_at_finalize.lock().unwrap() = Some(self.adapter_records.names());
            if self.fail_finalize {
                return Err(AgDomainsError::Acme("order invalid".to_owned()));
            }
            Ok(IssuedCertificate {
                cert_chain_pem: "CERT".to_owned(),
                private_key_pem: "KEY".to_owned(),
            })
        }
    }

    fn challenge(name: &str, value: &str) -> Dns01Desired {
        Dns01Desired {
            txt_name: name.to_owned(),
            txt_value: value.to_owned(),
        }
    }

    #[tokio::test]
    async fn wildcard_issuance_publishes_then_cleans_up() {
        let provider = MockProvider::new();
        let adapter = ProviderAdapter::new(provider.clone());
        // Wildcard + apex share the `_acme-challenge.example.com` name with two
        // distinct TXT values (RFC 8555 allows multiple values at one name).
        let mut order = MockOrder {
            challenges: vec![
                challenge("_acme-challenge.example.com", "value-apex"),
                challenge("_acme-challenge.example.com", "value-wildcard"),
            ],
            fail_finalize: false,
            observed_at_finalize: Mutex::new(None),
            adapter_records: provider.clone(),
        };

        let cert = issue_dns01_with_adapter(&adapter, "example.com", &mut order)
            .await
            .unwrap();
        assert_eq!(cert.cert_chain_pem, "CERT");

        // Both TXT values were present in the zone when finalize ran.
        let at_finalize = order.observed_at_finalize.lock().unwrap().clone().unwrap();
        assert!(at_finalize.contains(&"_acme-challenge.example.com=value-apex".to_owned()));
        assert!(at_finalize.contains(&"_acme-challenge.example.com=value-wildcard".to_owned()));

        // After issuance the challenge records are cleaned up (rollback).
        assert!(provider.names().is_empty(), "challenge TXT must be removed");
    }

    #[tokio::test]
    async fn cleanup_runs_even_when_finalize_fails() {
        let provider = MockProvider::new();
        let adapter = ProviderAdapter::new(provider.clone());
        let mut order = MockOrder {
            challenges: vec![challenge("_acme-challenge.example.com", "v")],
            fail_finalize: true,
            observed_at_finalize: Mutex::new(None),
            adapter_records: provider.clone(),
        };

        let err = issue_dns01_with_adapter(&adapter, "example.com", &mut order)
            .await
            .unwrap_err();
        assert!(matches!(err, AgDomainsError::Acme(_)));
        // Even on failure the challenge TXT is cleaned up.
        assert!(provider.names().is_empty());
    }

    #[tokio::test]
    async fn unrelated_records_are_never_touched() {
        let provider = MockProvider::new();
        // A normal A record and a stale challenge TXT from a prior run.
        provider.seed(DnsRecord {
            id: "a1".to_owned(),
            zone_id: "zone-1".to_owned(),
            name: "api.example.com".to_owned(),
            record_type: RecordType::A,
            content: "192.0.2.1".to_owned(),
            ttl: 300,
            proxied: false,
        });
        provider.seed(DnsRecord {
            id: "stale".to_owned(),
            zone_id: "zone-1".to_owned(),
            name: "_acme-challenge.example.com".to_owned(),
            record_type: RecordType::Txt,
            content: "old-value".to_owned(),
            ttl: 60,
            proxied: false,
        });
        let adapter = ProviderAdapter::new(provider.clone());
        let mut order = MockOrder {
            challenges: vec![challenge("_acme-challenge.example.com", "fresh-value")],
            fail_finalize: false,
            observed_at_finalize: Mutex::new(None),
            adapter_records: provider.clone(),
        };

        issue_dns01_with_adapter(&adapter, "example.com", &mut order)
            .await
            .unwrap();

        // The unrelated A record survives; only challenge TXTs were managed.
        let names = provider.names();
        assert!(
            names.contains(&"api.example.com=192.0.2.1".to_owned()),
            "unrelated A record must be preserved: {names:?}"
        );
        // The stale challenge value is gone (reconciled away, then cleaned up).
        assert!(!names
            .iter()
            .any(|n| n.starts_with("_acme-challenge.example.com")));
    }

    #[tokio::test]
    async fn empty_challenge_set_is_rejected() {
        let provider = MockProvider::new();
        let adapter = ProviderAdapter::new(provider.clone());
        let mut order = MockOrder {
            challenges: vec![],
            fail_finalize: false,
            observed_at_finalize: Mutex::new(None),
            adapter_records: provider.clone(),
        };
        let err = issue_dns01_with_adapter(&adapter, "example.com", &mut order)
            .await
            .unwrap_err();
        assert!(matches!(err, AgDomainsError::Acme(_)));
    }
}
