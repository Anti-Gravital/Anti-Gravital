//! `DnsProvider` trait and contract tests shared by all adapters.
//!
//! The trait is intentionally small: only the fundamental CRUD
//! operations. Adapters must not expose provider-specific primitives
//! through this trait — that would violate the dependency
//! inversion principle of Clean Architecture.

use async_trait::async_trait;

use crate::{
    error::AgDomainsError,
    record::{DnsRecord, DnsRecordSpec},
};

pub mod capabilities;
pub mod sdk;

#[cfg(feature = "cloudflare")]
pub mod cloudflare;

/// Abstraction over any DNS provider.
///
/// Adapters implement this trait. The ecosystem consumes it via
/// generics (`impl DnsProvider`) or trait objects (`Box<dyn DnsProvider>`)
/// depending on the context.
#[async_trait]
pub trait DnsProvider: Send + Sync {
    /// Provider name for logs and error messages.
    fn name(&self) -> &'static str;

    /// Resolves the internal zone identifier for the given domain.
    ///
    /// The zone is the provider's DNS administration unit. On
    /// Cloudflare it corresponds to the Zone ID; on other providers it may
    /// be named differently but the concept is the same.
    async fn zone_id(&self, domain: &str) -> Result<String, AgDomainsError>;

    /// Lists all DNS records of a zone.
    async fn list_records(&self, zone_id: &str) -> Result<Vec<DnsRecord>, AgDomainsError>;

    /// Creates a new DNS record and returns the record with its identity.
    async fn create_record(
        &self,
        zone_id: &str,
        spec: &DnsRecordSpec,
    ) -> Result<DnsRecord, AgDomainsError>;

    /// Replaces an existing DNS record and returns the updated state.
    async fn update_record(
        &self,
        zone_id: &str,
        record_id: &str,
        spec: &DnsRecordSpec,
    ) -> Result<DnsRecord, AgDomainsError>;

    /// Deletes a DNS record from the zone.
    async fn delete_record(&self, zone_id: &str, record_id: &str) -> Result<(), AgDomainsError>;
}

/// Contract test suite that every adapter must pass.
///
/// Invoked from each adapter's tests, passing a provider
/// configured against a test server (wiremock, fake, etc.).
/// Guarantees that no adapter silently drifts from the contract.
#[cfg(test)]
pub mod contract {
    use super::*;
    use crate::record::RecordType;

    /// Verifies that create_record + list_records + delete_record work.
    pub async fn test_create_list_delete<P: DnsProvider>(provider: &P, zone_id: &str) {
        let spec = DnsRecordSpec {
            name: "_ag-contract-test.ejemplo.com".to_owned(),
            record_type: RecordType::Txt,
            content: "ag-domains-contract-test-value".to_owned(),
            ttl: 60,
            proxied: false,
        };

        let created = provider
            .create_record(zone_id, &spec)
            .await
            .expect("create_record debe tener exito");

        assert_eq!(
            created.name, spec.name,
            "nombre del registro creado incorrecto"
        );
        assert_eq!(
            created.content, spec.content,
            "contenido del registro creado incorrecto"
        );
        assert_eq!(
            created.zone_id, zone_id,
            "zone_id del registro creado incorrecto"
        );
        assert!(
            !created.id.is_empty(),
            "id del registro no debe estar vacio"
        );

        let records = provider
            .list_records(zone_id)
            .await
            .expect("list_records debe tener exito");
        assert!(
            records.iter().any(|r| r.id == created.id),
            "el registro recien creado debe aparecer en list_records",
        );

        provider
            .delete_record(zone_id, &created.id)
            .await
            .expect("delete_record debe tener exito");
    }

    /// Verifies that update_record produces the new content.
    pub async fn test_update<P: DnsProvider>(provider: &P, zone_id: &str) {
        let spec = DnsRecordSpec {
            name: "_ag-contract-update.ejemplo.com".to_owned(),
            record_type: RecordType::Txt,
            content: "original".to_owned(),
            ttl: 60,
            proxied: false,
        };

        let created = provider
            .create_record(zone_id, &spec)
            .await
            .expect("create_record debe tener exito");

        let updated_spec = DnsRecordSpec {
            content: "actualizado".to_owned(),
            ..spec
        };
        let updated = provider
            .update_record(zone_id, &created.id, &updated_spec)
            .await
            .expect("update_record debe tener exito");

        assert_eq!(
            updated.content, "actualizado",
            "contenido debe reflejar la actualizacion"
        );
        assert_eq!(
            updated.id, created.id,
            "update no debe cambiar el id del registro"
        );

        provider
            .delete_record(zone_id, &created.id)
            .await
            .expect("delete_record debe tener exito");
    }
}
