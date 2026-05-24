//! Tipos de registro DNS.

use serde::{Deserialize, Serialize};

/// Tipos de registro DNS soportados por `ag-domains`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RecordType {
    /// Direccion IPv4.
    A,
    /// Direccion IPv6.
    Aaaa,
    /// Alias canonico.
    Cname,
    /// Registro de texto arbitrario (SPF, DKIM, ACME, etc.).
    Txt,
    /// Intercambiador de correo.
    Mx,
}

impl std::fmt::Display for RecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordType::A => write!(f, "A"),
            RecordType::Aaaa => write!(f, "AAAA"),
            RecordType::Cname => write!(f, "CNAME"),
            RecordType::Txt => write!(f, "TXT"),
            RecordType::Mx => write!(f, "MX"),
        }
    }
}

/// Especificacion para crear o actualizar un registro DNS.
///
/// No incluye `id` ni `zone_id`: esos los asigna el proveedor. Esta
/// separacion refleja el patron Command/DTO: el caller describe el estado
/// deseado sin necesitar conocer la identidad interna del proveedor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DnsRecordSpec {
    /// Nombre del registro (e.g., `"api.ejemplo.com"` o `"@"`).
    pub name: String,
    /// Tipo de registro DNS.
    pub record_type: RecordType,
    /// Contenido del registro (IP, FQDN, valor TXT, prioridad MX, etc.).
    pub content: String,
    /// Time-to-live en segundos. `1` = "automatico" en Cloudflare.
    pub ttl: u32,
    /// Proxy de Cloudflare activo. Ignorado por otros proveedores.
    #[serde(default)]
    pub proxied: bool,
}

/// Registro DNS con identidad asignada por el proveedor.
///
/// Representa el estado actual en el proveedor DNS. Se obtiene como
/// respuesta de `DnsProvider::create_record`, `update_record` o
/// `list_records`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DnsRecord {
    /// Identificador opaco del registro asignado por el proveedor.
    pub id: String,
    /// Identificador de la zona DNS a la que pertenece el registro.
    pub zone_id: String,
    /// Nombre del registro.
    pub name: String,
    /// Tipo de registro DNS.
    pub record_type: RecordType,
    /// Contenido del registro.
    pub content: String,
    /// Time-to-live en segundos.
    pub ttl: u32,
    /// Proxy de Cloudflare activo.
    #[serde(default)]
    pub proxied: bool,
}

impl DnsRecord {
    /// Extrae la especificacion sin identidad para reusar en actualizaciones.
    pub fn spec(&self) -> DnsRecordSpec {
        DnsRecordSpec {
            name: self.name.clone(),
            record_type: self.record_type.clone(),
            content: self.content.clone(),
            ttl: self.ttl,
            proxied: self.proxied,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_type_serde_roundtrip() {
        for (rt, expected_json) in [
            (RecordType::A, "\"A\""),
            (RecordType::Aaaa, "\"AAAA\""),
            (RecordType::Cname, "\"CNAME\""),
            (RecordType::Txt, "\"TXT\""),
            (RecordType::Mx, "\"MX\""),
        ] {
            let serialized = serde_json::to_string(&rt).unwrap();
            assert_eq!(serialized, expected_json, "serialization mismatch for {rt}");

            let deserialized: RecordType = serde_json::from_str(expected_json).unwrap();
            assert_eq!(
                deserialized, rt,
                "deserialization mismatch for {expected_json}"
            );
        }
    }

    #[test]
    fn record_type_display() {
        assert_eq!(RecordType::A.to_string(), "A");
        assert_eq!(RecordType::Aaaa.to_string(), "AAAA");
        assert_eq!(RecordType::Cname.to_string(), "CNAME");
        assert_eq!(RecordType::Txt.to_string(), "TXT");
        assert_eq!(RecordType::Mx.to_string(), "MX");
    }

    #[test]
    fn dns_record_spec_serde_roundtrip() {
        let spec = DnsRecordSpec {
            name: "api.ejemplo.com".to_owned(),
            record_type: RecordType::A,
            content: "192.0.2.1".to_owned(),
            ttl: 300,
            proxied: false,
        };
        let json = serde_json::to_string(&spec).unwrap();
        let decoded: DnsRecordSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(spec, decoded);
    }

    #[test]
    fn dns_record_spec_proxied_default_false() {
        let json = r#"{"name":"x","record_type":"A","content":"1.2.3.4","ttl":60}"#;
        let spec: DnsRecordSpec = serde_json::from_str(json).unwrap();
        assert!(
            !spec.proxied,
            "proxied debe ser false por defecto si se omite"
        );
    }

    #[test]
    fn dns_record_to_spec() {
        let record = DnsRecord {
            id: "abc123".to_owned(),
            zone_id: "zone456".to_owned(),
            name: "correo.ejemplo.com".to_owned(),
            record_type: RecordType::Txt,
            content: "v=spf1 include:sendgrid.net ~all".to_owned(),
            ttl: 1,
            proxied: false,
        };
        let spec = record.spec();
        assert_eq!(spec.name, record.name);
        assert_eq!(spec.record_type, record.record_type);
        assert_eq!(spec.content, record.content);
        assert_eq!(spec.ttl, record.ttl);
        assert_eq!(spec.proxied, record.proxied);
    }
}
