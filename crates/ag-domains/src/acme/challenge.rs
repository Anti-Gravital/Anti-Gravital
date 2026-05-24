//! Tipos y helpers para challenges ACME.
//!
//! Encapsula la logica de preparacion de un challenge DNS-01:
//! construir el nombre del registro TXT (`_acme-challenge.{domain}`) y
//! delegar la creacion a un `DnsProvider`.

use crate::{
    error::AgDomainsError,
    provider::DnsProvider,
    record::{DnsRecordSpec, RecordType},
};

/// Nombre DNS del registro TXT para el challenge DNS-01.
///
/// RFC 8555 secion 8.4: el nombre es `_acme-challenge.{domain}`.
pub fn dns01_record_name(domain: &str) -> String {
    // Strip trailing dot if present for consistency.
    let domain = domain.trim_end_matches('.');
    format!("_acme-challenge.{domain}")
}

/// Crea el registro TXT de challenge DNS-01 en el proveedor.
///
/// Retorna el `id` del registro creado — necesario para eliminarlo una vez
/// que Let's Encrypt valida el challenge.
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
        // TTL corto: el record se elimina tras la validacion.
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

/// Elimina el registro TXT de challenge DNS-01 del proveedor.
///
/// Se llama siempre tras la validacion, tanto si fue exitosa como si fallo.
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
