//! Trait `DnsProvider` y tests de contrato compartidos por todos los adapters.
//!
//! El trait es intencionalmente pequeño: solo las operaciones CRUD
//! fundamentales. Los adapters no deben exponer primitivas especificas del
//! proveedor a traves de este trait — eso violaria el principio de
//! inversion de dependencias del Clean Architecture.

use async_trait::async_trait;

use crate::{
    error::AgDomainsError,
    record::{DnsRecord, DnsRecordSpec},
};

#[cfg(feature = "cloudflare")]
pub mod cloudflare;

/// Abstraccion sobre cualquier proveedor DNS.
///
/// Los adapters implementan este trait. El ecosistema lo consume via
/// generics (`impl DnsProvider`) o trait objects (`Box<dyn DnsProvider>`)
/// segun el contexto.
#[async_trait]
pub trait DnsProvider: Send + Sync {
    /// Nombre del proveedor para logs y mensajes de error.
    fn name(&self) -> &'static str;

    /// Resuelve el identificador interno de zona para el dominio dado.
    ///
    /// La zona es la unidad de administracion DNS del proveedor. En
    /// Cloudflare corresponde al Zone ID; en otros proveedores puede
    /// llamarse diferente pero el concepto es el mismo.
    async fn zone_id(&self, domain: &str) -> Result<String, AgDomainsError>;

    /// Lista todos los registros DNS de una zona.
    async fn list_records(&self, zone_id: &str) -> Result<Vec<DnsRecord>, AgDomainsError>;

    /// Crea un nuevo registro DNS y devuelve el registro con su identidad.
    async fn create_record(
        &self,
        zone_id: &str,
        spec: &DnsRecordSpec,
    ) -> Result<DnsRecord, AgDomainsError>;

    /// Reemplaza un registro DNS existente y devuelve el estado actualizado.
    async fn update_record(
        &self,
        zone_id: &str,
        record_id: &str,
        spec: &DnsRecordSpec,
    ) -> Result<DnsRecord, AgDomainsError>;

    /// Elimina un registro DNS de la zona.
    async fn delete_record(&self, zone_id: &str, record_id: &str) -> Result<(), AgDomainsError>;
}

/// Suite de tests de contrato que todo adapter debe pasar.
///
/// Se invoca desde los tests de cada adapter, pasando un provider
/// configurado contra un servidor de prueba (wiremock, fake, etc.).
/// Garantiza que ningun adapter derive silenciosamente del contrato.
#[cfg(test)]
pub mod contract {
    use super::*;
    use crate::record::RecordType;

    /// Verifica que create_record + list_records + delete_record funcionen.
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

    /// Verifica que update_record produzca el nuevo contenido.
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
