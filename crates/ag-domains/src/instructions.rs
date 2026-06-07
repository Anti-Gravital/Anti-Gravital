//! DNS instruction engine.
//!
//! Generates the exact, copy-paste DNS records an operator must publish at
//! their provider to route a hostname to the Anti-Gravital edge and to prove
//! ownership (RFC-0009 section 5; blueprint section 10). The engine is
//! provider-agnostic; provider-specific automation is a later, feature-gated
//! phase. A BIND zone-file fragment export is provided for portability.

use std::net::{Ipv4Addr, Ipv6Addr};

use crate::hostname::{Hostname, HostnameKind};
use crate::ownership::ownership_record_name;
use crate::record::RecordType;

/// The edge targets a hostname should point at.
///
/// These are deployment configuration, not hardcoded infrastructure: the
/// caller (CLI flag / env / `ag-cloud`) supplies the real edge host and IPs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeTargets {
    /// CNAME target host for subdomains/wildcards (e.g. `edge.example-cloud.net`).
    pub edge_host: String,
    /// Apex IPv4 addresses (A records).
    pub ipv4: Vec<Ipv4Addr>,
    /// Apex IPv6 addresses (AAAA records).
    pub ipv6: Vec<Ipv6Addr>,
    /// TTL applied to generated records, in seconds.
    pub ttl: u32,
}

impl EdgeTargets {
    /// Creates edge targets with a 300s TTL and no apex IPs.
    pub fn new(edge_host: impl Into<String>) -> Self {
        Self {
            edge_host: edge_host.into(),
            ipv4: Vec::new(),
            ipv6: Vec::new(),
            ttl: 300,
        }
    }

    /// Adds an apex IPv4 address.
    pub fn with_ipv4(mut self, addr: Ipv4Addr) -> Self {
        self.ipv4.push(addr);
        self
    }

    /// Adds an apex IPv6 address.
    pub fn with_ipv6(mut self, addr: Ipv6Addr) -> Self {
        self.ipv6.push(addr);
        self
    }
}

/// A single record the operator must create.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionRecord {
    /// Record type.
    pub record_type: RecordType,
    /// Fully-qualified record name.
    pub name: String,
    /// Record value.
    pub value: String,
    /// TTL in seconds.
    pub ttl: u32,
    /// Whether this record is the ownership proof (vs routing).
    pub is_ownership: bool,
}

/// The full set of DNS instructions for an attachment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsInstructions {
    /// Routing records (A/AAAA/CNAME).
    pub routing: Vec<InstructionRecord>,
    /// The ownership TXT record.
    pub ownership: InstructionRecord,
    /// Notes the operator should be aware of (e.g. apex CNAME caveat).
    pub notes: Vec<String>,
}

impl DnsInstructions {
    /// Iterates over all records (routing first, then ownership).
    pub fn all(&self) -> impl Iterator<Item = &InstructionRecord> {
        self.routing.iter().chain(std::iter::once(&self.ownership))
    }
}

/// Generates DNS instructions for a hostname against the given edge targets and
/// expected ownership token.
pub fn generate_instructions(
    hostname: &Hostname,
    targets: &EdgeTargets,
    ownership_value: &str,
) -> DnsInstructions {
    let ttl = targets.ttl;
    let mut routing = Vec::new();
    let mut notes = Vec::new();

    match hostname.kind() {
        HostnameKind::Apex => {
            for ip in &targets.ipv4 {
                routing.push(InstructionRecord {
                    record_type: RecordType::A,
                    name: hostname.ascii().to_owned(),
                    value: ip.to_string(),
                    ttl,
                    is_ownership: false,
                });
            }
            for ip in &targets.ipv6 {
                routing.push(InstructionRecord {
                    record_type: RecordType::Aaaa,
                    name: hostname.ascii().to_owned(),
                    value: ip.to_string(),
                    ttl,
                    is_ownership: false,
                });
            }
            // Recommended companion record for the common `www` host.
            routing.push(InstructionRecord {
                record_type: RecordType::Cname,
                name: format!("www.{}", hostname.ascii()),
                value: targets.edge_host.clone(),
                ttl,
                is_ownership: false,
            });
            if targets.ipv4.is_empty() && targets.ipv6.is_empty() {
                notes.push(
                    "No apex IPs configured. Apex domains need A/AAAA records (or provider \
                     CNAME flattening / ALIAS) to point at the edge."
                        .to_owned(),
                );
            }
        }
        HostnameKind::Subdomain | HostnameKind::Wildcard => {
            routing.push(InstructionRecord {
                record_type: RecordType::Cname,
                name: hostname.ascii().to_owned(),
                value: targets.edge_host.clone(),
                ttl,
                is_ownership: false,
            });
            if hostname.kind() == HostnameKind::Wildcard {
                notes.push(
                    "Wildcard hostnames require a DNS-01 ACME challenge for TLS; an \
                     _acme-challenge TXT record will be requested during issuance."
                        .to_owned(),
                );
            }
        }
    }

    let ownership = InstructionRecord {
        record_type: RecordType::Txt,
        name: ownership_record_name(hostname),
        value: ownership_value.to_owned(),
        ttl,
        is_ownership: true,
    };

    DnsInstructions {
        routing,
        ownership,
        notes,
    }
}

/// Renders the instructions as a BIND zone-file fragment.
///
/// Names are emitted relative to `$ORIGIN <registered-domain>.`; the apex is
/// rendered as `@`.
pub fn export_bind_zone(hostname: &Hostname, instructions: &DnsInstructions) -> String {
    let origin = hostname.registered_domain();
    let mut out = format!("$ORIGIN {origin}.\n");
    for rec in instructions.all() {
        let rel = relative_name(&rec.name, origin);
        let value = match rec.record_type {
            RecordType::Cname => ensure_trailing_dot(&rec.value),
            RecordType::Txt => format!("\"{}\"", rec.value),
            _ => rec.value.clone(),
        };
        out.push_str(&format!(
            "{:<24} {} IN {:<5} {}\n",
            rel, rec.ttl, rec.record_type, value
        ));
    }
    out
}

fn relative_name(name: &str, origin: &str) -> String {
    if name == origin {
        "@".to_owned()
    } else if let Some(prefix) = name.strip_suffix(&format!(".{origin}")) {
        prefix.to_owned()
    } else {
        ensure_trailing_dot(name)
    }
}

fn ensure_trailing_dot(s: &str) -> String {
    if s.ends_with('.') {
        s.to_owned()
    } else {
        format!("{s}.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn targets() -> EdgeTargets {
        EdgeTargets::new("edge.example-cloud.net")
            .with_ipv4(Ipv4Addr::new(203, 0, 113, 10))
            .with_ipv6("2001:db8::10".parse().unwrap())
    }

    #[test]
    fn apex_generates_a_aaaa_and_www() {
        let h = Hostname::parse("example.com").unwrap();
        let ins = generate_instructions(&h, &targets(), "ag-verification=tok");
        let types: Vec<_> = ins.routing.iter().map(|r| r.record_type.clone()).collect();
        assert!(types.contains(&RecordType::A));
        assert!(types.contains(&RecordType::Aaaa));
        assert!(types.contains(&RecordType::Cname));
        assert_eq!(ins.ownership.name, "_ag-domain.example.com");
        assert!(ins.ownership.is_ownership);
    }

    #[test]
    fn subdomain_generates_single_cname() {
        let h = Hostname::parse("api.example.com").unwrap();
        let ins = generate_instructions(&h, &targets(), "ag-verification=tok");
        assert_eq!(ins.routing.len(), 1);
        assert_eq!(ins.routing[0].record_type, RecordType::Cname);
        assert_eq!(ins.routing[0].name, "api.example.com");
        assert_eq!(ins.routing[0].value, "edge.example-cloud.net");
    }

    #[test]
    fn apex_without_ips_emits_note() {
        let h = Hostname::parse("example.com").unwrap();
        let ins = generate_instructions(&h, &EdgeTargets::new("edge.x.net"), "tok");
        assert!(ins.notes.iter().any(|n| n.contains("apex")));
    }

    #[test]
    fn wildcard_emits_dns01_note() {
        let h = Hostname::parse("*.example.com").unwrap();
        let ins = generate_instructions(&h, &targets(), "tok");
        assert!(ins.notes.iter().any(|n| n.contains("DNS-01")));
    }

    #[test]
    fn bind_export_renders_apex_as_at() {
        let h = Hostname::parse("example.com").unwrap();
        let ins = generate_instructions(&h, &targets(), "ag-verification=tok");
        let zone = export_bind_zone(&h, &ins);
        assert!(zone.contains("$ORIGIN example.com."));
        assert!(zone.contains("@ "));
        assert!(zone.contains("edge.example-cloud.net."));
        assert!(zone.contains("\"ag-verification=tok\""));
        assert!(zone.contains("_ag-domain"));
    }
}
