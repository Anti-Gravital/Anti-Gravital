//! Provider adapter SDK seam: plan/diff/apply/verify/rollback (blueprint section 11).
//!
//! The base [`DnsProvider`] trait is intentionally a thin CRUD surface. Real
//! provider automation needs a richer, *declarative* contract: compute the
//! difference between a desired record set and what the zone currently holds,
//! apply that difference as a single reviewed plan, verify the result, and roll
//! it back if needed. This module adds that seam without changing `DnsProvider`.
//!
//! Manual mode stays the default (RFC-0011 / ADR-0009): nothing here runs unless
//! a provider adapter is explicitly configured. The seam is built *on top of*
//! the existing CRUD trait, so the Cloudflare adapter (and any future adapter)
//! participates through [`ProviderAdapter`] with zero adapter-specific code.
//!
//! The core of the seam is the pure [`diff`] function, which is fully
//! deterministic and unit-tested independently of any network.

use async_trait::async_trait;

use crate::{
    error::AgDomainsError,
    provider::DnsProvider,
    record::{DnsRecord, DnsRecordSpec, RecordType},
};

/// Identity of a single declarative change against a zone.
///
/// `diff` produces these; `apply` executes them; `rollback` executes their
/// inverse. The variants carry enough state to be both human-reviewable (a
/// dry-run) and reversible.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ZoneChange {
    /// Create a record that is desired but absent from the zone.
    Create(DnsRecordSpec),
    /// Replace an existing record whose content/TTL/proxy state drifted.
    Update {
        /// Provider-assigned identity of the record to replace.
        record_id: String,
        /// Observed state before the change (used to build the inverse).
        from: DnsRecordSpec,
        /// Desired state after the change.
        to: DnsRecordSpec,
    },
    /// Delete a record present in the zone scope but no longer desired.
    Delete {
        /// Provider-assigned identity of the record to delete.
        record_id: String,
        /// Observed state before deletion (used to build the inverse).
        record: DnsRecordSpec,
    },
}

impl ZoneChange {
    /// The inverse change that undoes this one.
    ///
    /// `Create` is undone by deleting the record that `apply` produced; the
    /// caller supplies its `record_id` (unknown until creation). `Update` and
    /// `Delete` already carry the prior state needed to reverse them.
    fn inverse(&self, created_id: Option<&str>) -> Option<ZoneChange> {
        match self {
            ZoneChange::Create(spec) => created_id.map(|id| ZoneChange::Delete {
                record_id: id.to_owned(),
                record: spec.clone(),
            }),
            ZoneChange::Update {
                record_id,
                from,
                to,
            } => Some(ZoneChange::Update {
                record_id: record_id.clone(),
                from: to.clone(),
                to: from.clone(),
            }),
            ZoneChange::Delete { record, .. } => Some(ZoneChange::Create(record.clone())),
        }
    }
}

/// An ordered set of changes that reconciles a zone scope to the desired state.
///
/// A plan is a dry-run artifact first and an execution unit second: print it,
/// review it, then `apply` it. An empty plan means the zone already matches.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ZonePlan {
    /// Changes to execute, in apply order (deletes and updates before creates
    /// is not required; the operations are independent per record identity).
    pub changes: Vec<ZoneChange>,
}

impl ZonePlan {
    /// A plan with no changes: the zone scope already matches the desired set.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Number of records to create.
    pub fn creates(&self) -> usize {
        self.changes
            .iter()
            .filter(|c| matches!(c, ZoneChange::Create(_)))
            .count()
    }

    /// Number of records to update.
    pub fn updates(&self) -> usize {
        self.changes
            .iter()
            .filter(|c| matches!(c, ZoneChange::Update { .. }))
            .count()
    }

    /// Number of records to delete.
    pub fn deletes(&self) -> usize {
        self.changes
            .iter()
            .filter(|c| matches!(c, ZoneChange::Delete { .. }))
            .count()
    }

    /// One-line, human-readable summary for dry-run output and logs.
    pub fn summary(&self) -> String {
        format!(
            "{} change(s): +{} create, ~{} update, -{} delete",
            self.changes.len(),
            self.creates(),
            self.updates(),
            self.deletes()
        )
    }
}

/// A resolved zone the adapter operates on (the provider's administration unit).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ZoneRef {
    /// The registrable domain the zone serves (e.g. `example.com`).
    pub domain: String,
    /// Provider-internal zone identifier resolved by `discover`.
    pub zone_id: String,
}

/// Handle returned by `apply`, carrying the inverse plan for `rollback`.
///
/// Rolling back is `apply(inverse)`: this records exactly what to do to undo a
/// committed change set, including the identities of records that `apply`
/// created (unknown before execution).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChangeRef {
    /// The plan that reverses the applied changes.
    pub inverse: ZonePlan,
}

/// Outcome of `verify`: whether the zone now matches the plan's intent.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum VerifyOutcome {
    /// Every change's post-condition holds in the observed zone.
    Applied,
    /// One or more changes did not take effect; the residual drift is attached.
    Drift {
        /// Human-readable description of what is still wrong.
        detail: String,
    },
}

/// Whether a record type holds a single value per name or multiple values.
///
/// `A`/`AAAA`/`CNAME` are single-valued per name, so a changed address is an
/// *update* of the same record identity. `TXT`/`MX` are multi-valued: each
/// distinct value is its own record, so a changed value is a delete + create.
fn is_singleton(record_type: &RecordType) -> bool {
    matches!(
        record_type,
        RecordType::A | RecordType::Aaaa | RecordType::Cname
    )
}

/// Normalized matching key for a record, distinguishing singleton vs multi-value
/// types. Names are lowercased and stripped of a trailing dot.
fn record_key(name: &str, record_type: &RecordType, content: &str) -> (String, RecordType, String) {
    let name = name.trim_end_matches('.').to_ascii_lowercase();
    if is_singleton(record_type) {
        (name, record_type.clone(), String::new())
    } else {
        (name, record_type.clone(), content.to_owned())
    }
}

/// Whether two specs are observably equal for reconciliation purposes.
fn spec_matches(desired: &DnsRecordSpec, observed: &DnsRecord) -> bool {
    desired.content == observed.content
        && desired.ttl == observed.ttl
        && desired.proxied == observed.proxied
}

/// Compute the plan that reconciles `observed` to `desired` (pure, no I/O).
///
/// Contract: `desired` is treated as the **authoritative set for the scope that
/// `observed` represents**. Observed records absent from `desired` are scheduled
/// for deletion. Callers that do not own the whole zone MUST scope `observed`
/// to the records `ag-domains` manages (e.g. the `_acme-challenge` subtree)
/// before calling this; otherwise unrelated records would be planned for
/// deletion. This keeps the function total and deterministic while leaving the
/// safety boundary explicit at the call site.
pub fn diff(desired: &[DnsRecordSpec], observed: &[DnsRecord]) -> ZonePlan {
    let mut changes = Vec::new();
    let mut matched_observed = vec![false; observed.len()];

    for want in desired {
        let want_key = record_key(&want.name, &want.record_type, &want.content);
        let found = observed.iter().enumerate().find(|(i, have)| {
            !matched_observed[*i]
                && record_key(&have.name, &have.record_type, &have.content) == want_key
        });

        match found {
            Some((idx, have)) => {
                matched_observed[idx] = true;
                if !spec_matches(want, have) {
                    changes.push(ZoneChange::Update {
                        record_id: have.id.clone(),
                        from: have.spec(),
                        to: want.clone(),
                    });
                }
            }
            None => changes.push(ZoneChange::Create(want.clone())),
        }
    }

    for (i, have) in observed.iter().enumerate() {
        if !matched_observed[i] {
            changes.push(ZoneChange::Delete {
                record_id: have.id.clone(),
                record: have.spec(),
            });
        }
    }

    ZonePlan { changes }
}

/// Declarative provider adapter SDK (blueprint section 11).
///
/// Adapters either implement this directly or, more commonly, obtain it for
/// free by wrapping a [`DnsProvider`] in [`ProviderAdapter`]. The default
/// `apply`/`verify`/`rollback` implementations are written purely in terms of
/// `read` plus the per-record operations, so a CRUD adapter needs no extra code.
#[async_trait]
pub trait ZoneAdapter: Send + Sync {
    /// Adapter name for logs and diagnostics.
    fn name(&self) -> &'static str;

    /// Resolve the provider zone that administers `domain`.
    async fn discover(&self, domain: &str) -> Result<ZoneRef, AgDomainsError>;

    /// Read the records currently present in the zone.
    async fn read(&self, zone: &ZoneRef) -> Result<Vec<DnsRecord>, AgDomainsError>;

    /// Apply a plan, returning a handle that can roll it back.
    async fn apply(&self, zone: &ZoneRef, plan: &ZonePlan) -> Result<ChangeRef, AgDomainsError>;

    /// Re-read the zone and confirm every change in `plan` took effect.
    async fn verify(
        &self,
        zone: &ZoneRef,
        plan: &ZonePlan,
    ) -> Result<VerifyOutcome, AgDomainsError>;

    /// Undo a previously applied change set.
    async fn rollback(&self, zone: &ZoneRef, change: &ChangeRef) -> Result<(), AgDomainsError>;
}

/// Bridges any [`DnsProvider`] CRUD adapter to the declarative [`ZoneAdapter`].
///
/// This is what lets the existing Cloudflare adapter (and any future one)
/// participate in plan/apply/verify/rollback without provider-specific code:
/// the plan is executed through the CRUD trait, and the inverse plan is built
/// from the records the provider returns.
pub struct ProviderAdapter<P: DnsProvider> {
    provider: P,
}

impl<P: DnsProvider> ProviderAdapter<P> {
    /// Wrap a CRUD provider in the declarative SDK seam.
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    /// Borrow the underlying CRUD provider.
    pub fn provider(&self) -> &P {
        &self.provider
    }
}

#[async_trait]
impl<P: DnsProvider> ZoneAdapter for ProviderAdapter<P> {
    fn name(&self) -> &'static str {
        self.provider.name()
    }

    async fn discover(&self, domain: &str) -> Result<ZoneRef, AgDomainsError> {
        let zone_id = self.provider.zone_id(domain).await?;
        Ok(ZoneRef {
            domain: domain.to_owned(),
            zone_id,
        })
    }

    async fn read(&self, zone: &ZoneRef) -> Result<Vec<DnsRecord>, AgDomainsError> {
        self.provider.list_records(&zone.zone_id).await
    }

    async fn apply(&self, zone: &ZoneRef, plan: &ZonePlan) -> Result<ChangeRef, AgDomainsError> {
        // Build the inverse as we go: creates need the id the provider assigns.
        let mut inverse = Vec::with_capacity(plan.changes.len());
        for change in &plan.changes {
            match change {
                ZoneChange::Create(spec) => {
                    let created = self.provider.create_record(&zone.zone_id, spec).await?;
                    if let Some(inv) = change.inverse(Some(&created.id)) {
                        inverse.push(inv);
                    }
                }
                ZoneChange::Update { record_id, to, .. } => {
                    self.provider
                        .update_record(&zone.zone_id, record_id, to)
                        .await?;
                    if let Some(inv) = change.inverse(None) {
                        inverse.push(inv);
                    }
                }
                ZoneChange::Delete { record_id, .. } => {
                    self.provider
                        .delete_record(&zone.zone_id, record_id)
                        .await?;
                    if let Some(inv) = change.inverse(None) {
                        inverse.push(inv);
                    }
                }
            }
        }
        // Rollback undoes changes in reverse application order.
        inverse.reverse();
        Ok(ChangeRef {
            inverse: ZonePlan { changes: inverse },
        })
    }

    async fn verify(
        &self,
        zone: &ZoneRef,
        plan: &ZonePlan,
    ) -> Result<VerifyOutcome, AgDomainsError> {
        let observed = self.read(zone).await?;
        for change in &plan.changes {
            let ok = match change {
                ZoneChange::Create(spec) | ZoneChange::Update { to: spec, .. } => observed
                    .iter()
                    .any(|r| spec_matches(spec, r) && same_record(spec, r)),
                ZoneChange::Delete { record, .. } => {
                    !observed.iter().any(|r| same_record(record, r))
                }
            };
            if !ok {
                return Ok(VerifyOutcome::Drift {
                    detail: format!("change not realized: {change:?}"),
                });
            }
        }
        Ok(VerifyOutcome::Applied)
    }

    async fn rollback(&self, zone: &ZoneRef, change: &ChangeRef) -> Result<(), AgDomainsError> {
        self.apply(zone, &change.inverse).await.map(|_| ())
    }
}

/// Whether an observed record has the same name+type+content as a spec.
fn same_record(spec: &DnsRecordSpec, observed: &DnsRecord) -> bool {
    record_key(&spec.name, &spec.record_type, &spec.content)
        == record_key(&observed.name, &observed.record_type, &observed.content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    fn spec(name: &str, rt: RecordType, content: &str, ttl: u32) -> DnsRecordSpec {
        DnsRecordSpec {
            name: name.to_owned(),
            record_type: rt,
            content: content.to_owned(),
            ttl,
            proxied: false,
        }
    }

    fn record(id: &str, name: &str, rt: RecordType, content: &str, ttl: u32) -> DnsRecord {
        DnsRecord {
            id: id.to_owned(),
            zone_id: "zone-1".to_owned(),
            name: name.to_owned(),
            record_type: rt,
            content: content.to_owned(),
            ttl,
            proxied: false,
        }
    }

    #[test]
    fn diff_empty_when_matching() {
        let desired = vec![spec("api.example.com", RecordType::A, "192.0.2.1", 300)];
        let observed = vec![record(
            "r1",
            "api.example.com",
            RecordType::A,
            "192.0.2.1",
            300,
        )];
        assert!(diff(&desired, &observed).is_empty());
    }

    #[test]
    fn diff_creates_missing() {
        let desired = vec![spec("api.example.com", RecordType::A, "192.0.2.1", 300)];
        let plan = diff(&desired, &[]);
        assert_eq!(plan.creates(), 1);
        assert_eq!(plan.changes.len(), 1);
    }

    #[test]
    fn diff_updates_singleton_content_change() {
        let desired = vec![spec("api.example.com", RecordType::A, "192.0.2.2", 300)];
        let observed = vec![record(
            "r1",
            "api.example.com",
            RecordType::A,
            "192.0.2.1",
            300,
        )];
        let plan = diff(&desired, &observed);
        assert_eq!(plan.updates(), 1);
        assert_eq!(plan.creates(), 0);
        assert_eq!(plan.deletes(), 0);
    }

    #[test]
    fn diff_updates_on_ttl_only_change() {
        let desired = vec![spec("api.example.com", RecordType::A, "192.0.2.1", 60)];
        let observed = vec![record(
            "r1",
            "api.example.com",
            RecordType::A,
            "192.0.2.1",
            300,
        )];
        assert_eq!(diff(&desired, &observed).updates(), 1);
    }

    #[test]
    fn diff_deletes_unmanaged_in_scope() {
        let observed = vec![record(
            "r1",
            "old.example.com",
            RecordType::A,
            "192.0.2.9",
            300,
        )];
        let plan = diff(&[], &observed);
        assert_eq!(plan.deletes(), 1);
    }

    #[test]
    fn diff_txt_value_change_is_delete_plus_create() {
        // TXT is multi-valued: a changed value is a new record, not an update.
        let desired = vec![spec(
            "_acme-challenge.example.com",
            RecordType::Txt,
            "new",
            60,
        )];
        let observed = vec![record(
            "r1",
            "_acme-challenge.example.com",
            RecordType::Txt,
            "old",
            60,
        )];
        let plan = diff(&desired, &observed);
        assert_eq!(plan.creates(), 1, "new TXT value created");
        assert_eq!(plan.deletes(), 1, "old TXT value removed");
        assert_eq!(plan.updates(), 0);
    }

    #[test]
    fn diff_name_is_case_and_trailing_dot_insensitive() {
        let desired = vec![spec("API.Example.com", RecordType::A, "192.0.2.1", 300)];
        let observed = vec![record(
            "r1",
            "api.example.com.",
            RecordType::A,
            "192.0.2.1",
            300,
        )];
        assert!(diff(&desired, &observed).is_empty());
    }

    /// Minimal in-memory `DnsProvider` for SDK seam tests (no network).
    struct MockProvider {
        records: Mutex<HashMap<String, DnsRecord>>,
        seq: Mutex<u64>,
    }

    impl MockProvider {
        fn new() -> Self {
            Self {
                records: Mutex::new(HashMap::new()),
                seq: Mutex::new(0),
            }
        }

        fn seed(&self, rec: DnsRecord) {
            self.records.lock().unwrap().insert(rec.id.clone(), rec);
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
            Ok(self.records.lock().unwrap().values().cloned().collect())
        }

        async fn create_record(
            &self,
            zone_id: &str,
            spec: &DnsRecordSpec,
        ) -> Result<DnsRecord, AgDomainsError> {
            let mut seq = self.seq.lock().unwrap();
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
            self.records.lock().unwrap().insert(id, rec.clone());
            Ok(rec)
        }

        async fn update_record(
            &self,
            zone_id: &str,
            record_id: &str,
            spec: &DnsRecordSpec,
        ) -> Result<DnsRecord, AgDomainsError> {
            let mut records = self.records.lock().unwrap();
            let rec = records
                .get_mut(record_id)
                .ok_or_else(|| AgDomainsError::RecordNotFound(record_id.to_owned()))?;
            rec.zone_id = zone_id.to_owned();
            rec.name = spec.name.clone();
            rec.record_type = spec.record_type.clone();
            rec.content = spec.content.clone();
            rec.ttl = spec.ttl;
            rec.proxied = spec.proxied;
            Ok(rec.clone())
        }

        async fn delete_record(
            &self,
            _zone_id: &str,
            record_id: &str,
        ) -> Result<(), AgDomainsError> {
            self.records
                .lock()
                .unwrap()
                .remove(record_id)
                .map(|_| ())
                .ok_or_else(|| AgDomainsError::RecordNotFound(record_id.to_owned()))
        }
    }

    #[tokio::test]
    async fn apply_verify_roundtrip_through_provider_adapter() {
        let adapter = ProviderAdapter::new(MockProvider::new());
        let zone = adapter.discover("example.com").await.unwrap();

        let desired = vec![
            spec("api.example.com", RecordType::A, "192.0.2.1", 300),
            spec("_acme-challenge.example.com", RecordType::Txt, "token", 60),
        ];
        let observed = adapter.read(&zone).await.unwrap();
        let plan = diff(&desired, &observed);
        assert_eq!(plan.creates(), 2);

        let change_ref = adapter.apply(&zone, &plan).await.unwrap();
        assert_eq!(
            adapter.verify(&zone, &plan).await.unwrap(),
            VerifyOutcome::Applied
        );

        // Second diff is a no-op: the zone now matches desired.
        let observed = adapter.read(&zone).await.unwrap();
        assert!(diff(&desired, &observed).is_empty());

        // Rollback restores the original (empty) state.
        adapter.rollback(&zone, &change_ref).await.unwrap();
        assert!(adapter.read(&zone).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn rollback_restores_prior_record_content() {
        let provider = MockProvider::new();
        provider.seed(record(
            "r1",
            "api.example.com",
            RecordType::A,
            "192.0.2.1",
            300,
        ));
        let adapter = ProviderAdapter::new(provider);
        let zone = adapter.discover("example.com").await.unwrap();

        let desired = vec![spec("api.example.com", RecordType::A, "192.0.2.2", 300)];
        let observed = adapter.read(&zone).await.unwrap();
        let plan = diff(&desired, &observed);
        assert_eq!(plan.updates(), 1);

        let change_ref = adapter.apply(&zone, &plan).await.unwrap();
        let after = adapter.read(&zone).await.unwrap();
        assert_eq!(after[0].content, "192.0.2.2");

        adapter.rollback(&zone, &change_ref).await.unwrap();
        let restored = adapter.read(&zone).await.unwrap();
        assert_eq!(
            restored[0].content, "192.0.2.1",
            "rollback restores content"
        );
    }

    #[test]
    fn plan_summary_is_human_readable() {
        let plan = ZonePlan {
            changes: vec![
                ZoneChange::Create(spec("a.example.com", RecordType::A, "192.0.2.1", 60)),
                ZoneChange::Delete {
                    record_id: "r1".to_owned(),
                    record: spec("b.example.com", RecordType::A, "192.0.2.2", 60),
                },
            ],
        };
        assert_eq!(
            plan.summary(),
            "2 change(s): +1 create, ~0 update, -1 delete"
        );
    }
}
