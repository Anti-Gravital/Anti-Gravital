//! Types and helpers for ACME challenges.
//!
//! Encapsulates the logic of preparing a DNS-01 challenge:
//! building the TXT record name (`_acme-challenge.{domain}`) and
//! delegating creation to a `DnsProvider`.

use crate::{
    error::AgDomainsError,
    provider::DnsProvider,
    record::{DnsRecordSpec, RecordType},
};

/// DNS name of the TXT record for the DNS-01 challenge.
///
/// RFC 8555 section 8.4: the name is `_acme-challenge.{domain}`.
pub fn dns01_record_name(domain: &str) -> String {
    // Strip trailing dot if present for consistency.
    let domain = domain.trim_end_matches('.');
    format!("_acme-challenge.{domain}")
}

/// Creates the DNS-01 challenge TXT record in the provider.
///
/// Returns the `id` of the created record — needed to remove it once
/// Let's Encrypt validates the challenge.
pub async fn set_dns01_challenge<P: DnsProvider>(
    provider: &P,
    zone_id: &str,
    domain: &str,
    dns_value: &str,
) -> Result<String, AgDomainsError> {
    let name = dns01_record_name(domain);
    let spec = DnsRecordSpec {
        name,
        record_type: RecordType::Txt,
        content: dns_value.to_owned(),
        // Short TTL: the record is removed after validation.
        ttl: 60,
        proxied: false,
    };

    let record = provider.create_record(zone_id, &spec).await?;
    tracing::debug!(
        provider = provider.name(),
        domain,
        record_id = record.id,
        "challenge DNS-01 creado"
    );
    Ok(record.id)
}

/// Removes the DNS-01 challenge TXT record from the provider.
///
/// Always called after validation, whether it succeeded or failed.
pub async fn remove_dns01_challenge<P: DnsProvider>(
    provider: &P,
    zone_id: &str,
    record_id: &str,
) -> Result<(), AgDomainsError> {
    provider.delete_record(zone_id, record_id).await?;
    tracing::debug!(
        provider = provider.name(),
        record_id,
        "challenge DNS-01 eliminado"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dns01_record_name_basic() {
        assert_eq!(
            dns01_record_name("ejemplo.com"),
            "_acme-challenge.ejemplo.com"
        );
        assert_eq!(
            dns01_record_name("sub.ejemplo.com"),
            "_acme-challenge.sub.ejemplo.com"
        );
    }

    #[test]
    fn dns01_record_name_strips_trailing_dot() {
        assert_eq!(
            dns01_record_name("ejemplo.com."),
            "_acme-challenge.ejemplo.com"
        );
    }
}
