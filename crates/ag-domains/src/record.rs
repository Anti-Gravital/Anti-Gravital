//! DNS record types.

use serde::{Deserialize, Serialize};

/// DNS record types supported by `ag-domains`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RecordType {
    /// IPv4 address.
    A,
    /// IPv6 address.
    Aaaa,
    /// Canonical alias.
    Cname,
    /// Arbitrary text record (SPF, DKIM, ACME, etc.).
    Txt,
    /// Mail exchanger.
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

/// Specification for creating or updating a DNS record.
///
/// Does not include `id` or `zone_id`: those are assigned by the provider. This
/// separation reflects the Command/DTO pattern: the caller describes the desired
/// state without needing to know the provider's internal identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DnsRecordSpec {
    /// Record name (e.g., `"api.ejemplo.com"` or `"@"`).
    pub name: String,
    /// DNS record type.
    pub record_type: RecordType,
    /// Record content (IP, FQDN, TXT value, MX priority, etc.).
    pub content: String,
    /// Time-to-live in seconds. `1` = "automatic" on Cloudflare.
    pub ttl: u32,
    /// Cloudflare proxy enabled. Ignored by other providers.
    #[serde(default)]
    pub proxied: bool,
}

/// DNS record with an identity assigned by the provider.
///
/// Represents the current state in the DNS provider. Obtained as the
/// response of `DnsProvider::create_record`, `update_record` or
/// `list_records`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DnsRecord {
    /// Opaque record identifier assigned by the provider.
    pub id: String,
    /// Identifier of the DNS zone the record belongs to.
    pub zone_id: String,
    /// Record name.
    pub name: String,
    /// DNS record type.
    pub record_type: RecordType,
    /// Record content.
    pub content: String,
    /// Time-to-live in seconds.
    pub ttl: u32,
    /// Cloudflare proxy enabled.
    #[serde(default)]
    pub proxied: bool,
}

impl DnsRecord {
    /// Extracts the identity-less specification for reuse in updates.
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
