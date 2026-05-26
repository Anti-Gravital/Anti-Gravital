//! Generation and application of DNS records for mail delivery.
//!
//! `ag-mail` declares its requirements through `MailDnsRequirements`
//! and this module materializes them as `DnsRecordSpec` and applies them via
//! `DnsProvider`. No dependency cycle: `ag-mail` and `ag-domains` know
//! each other only through these intermediate types.
//!
//! The main function of the cooperation flow is `apply_mail_records`:
//! it generates the necessary records and upserts them against the DNS zone,
//! creating the ones that do not exist and updating those whose content differs.

use crate::{
    error::AgDomainsError,
    metrics,
    provider::DnsProvider,
    record::{DnsRecordSpec, RecordType},
};

/// DNS requirements for a domain's mail delivery.
///
/// The caller (typically `ag-mail`) fills this struct and passes it to
/// `generate_mail_records`. `ag-domains` materializes it into DNS records.
#[derive(Debug, Clone)]
pub struct MailDnsRequirements {
    /// Root domain (e.g., `"ejemplo.com"`).
    pub domain: String,

    /// Active DKIM selectors. Each selector produces a TXT record
    /// `{selector}._domainkey.{domain}`.
    pub dkim_selectors: Vec<DkimSelector>,

    /// Additional SPF includes (e.g., `"include:sendgrid.net"`).
    pub spf_includes: Vec<String>,

    /// IP addresses authorized in SPF (`ip4:`/`ip6:` mechanism).
    pub spf_ips: Vec<String>,

    /// DMARC policy: `"none"`, `"quarantine"` or `"reject"`.
    pub dmarc_policy: DmarcPolicy,

    /// Email to which the aggregated DMARC reports are sent.
    pub dmarc_rua: Option<String>,
}

/// DKIM selector with its public key.
#[derive(Debug, Clone)]
pub struct DkimSelector {
    /// Selector name (e.g., `"s1"`, `"mail"`).
    pub name: String,
    /// DKIM TXT record value (full content, without the `v=DKIM1` prefix).
    pub public_key_record: String,
}

/// DMARC policy (RFC 7489).
#[derive(Debug, Clone, Default)]
pub enum DmarcPolicy {
    /// Only collect reports, do not affect delivery.
    #[default]
    None,
    /// Mark failing messages as spam.
    Quarantine,
    /// Reject failing messages.
    Reject,
}

impl DmarcPolicy {
    fn as_str(&self) -> &'static str {
        match self {
            DmarcPolicy::None => "none",
            DmarcPolicy::Quarantine => "quarantine",
            DmarcPolicy::Reject => "reject",
        }
    }
}

/// Generates the DNS records needed for mail delivery.
///
/// Returns a list of `DnsRecordSpec` ready to apply via
/// `DnsProvider::create_record`. The caller must check whether they already
/// exist and upsert if appropriate.
pub fn generate_mail_records(req: &MailDnsRequirements) -> Vec<DnsRecordSpec> {
    let mut records = Vec::new();

    // SPF
    records.push(spf_record(req));

    // DKIM per selector
    for selector in &req.dkim_selectors {
        records.push(dkim_record(&req.domain, selector));
    }

    // DMARC
    records.push(dmarc_record(req));

    records
}

fn spf_record(req: &MailDnsRequirements) -> DnsRecordSpec {
    let includes = req
        .spf_includes
        .iter()
        .map(|i| format!("include:{i}"))
        .collect::<Vec<_>>();

    let ips = req.spf_ips.iter().map(|ip| {
        if ip.contains(':') {
            format!("ip6:{ip}")
        } else {
            format!("ip4:{ip}")
        }
    });

    let mechanisms = std::iter::once("mx".to_owned())
        .chain(includes)
        .chain(ips)
        .collect::<Vec<_>>()
        .join(" ");

    DnsRecordSpec {
        name: req.domain.clone(),
        record_type: RecordType::Txt,
        content: format!("v=spf1 {mechanisms} ~all"),
        ttl: 300,
        proxied: false,
    }
}

fn dkim_record(domain: &str, selector: &DkimSelector) -> DnsRecordSpec {
    DnsRecordSpec {
        name: format!("{}._domainkey.{}", selector.name, domain),
        record_type: RecordType::Txt,
        content: selector.public_key_record.clone(),
        ttl: 300,
        proxied: false,
    }
}

fn dmarc_record(req: &MailDnsRequirements) -> DnsRecordSpec {
    let rua = req
        .dmarc_rua
        .as_deref()
        .map(|email| format!("; rua=mailto:{email}"))
        .unwrap_or_default();

    DnsRecordSpec {
        name: format!("_dmarc.{}", req.domain),
        record_type: RecordType::Txt,
        content: format!("v=DMARC1; p={}{}", req.dmarc_policy.as_str(), rua),
        ttl: 300,
        proxied: false,
    }
}

/// Result of the mail DNS records upsert.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MailRecordsResult {
    /// Records created because they did not exist.
    pub created: usize,
    /// Records updated because the content differed.
    pub updated: usize,
    /// Records unchanged (already existed with the correct content).
    pub unchanged: usize,
}

/// Applies the DNS records needed for mail delivery.
///
/// For each record that `generate_mail_records` produces:
/// - If it does not exist in the zone: it is created with `DnsProvider::create_record`.
/// - If it exists but the content is different: it is updated.
/// - If it exists with the same content: it is skipped (idempotent).
///
/// The function is safe to call repeatedly; re-runs
/// return `unchanged` for all already-correct records.
pub async fn apply_mail_records<P: DnsProvider>(
    req: &MailDnsRequirements,
    provider: &P,
    zone_id: &str,
) -> Result<MailRecordsResult, AgDomainsError> {
    let desired = generate_mail_records(req);
    let existing = provider.list_records(zone_id).await?;

    let mut result = MailRecordsResult::default();

    for spec in &desired {
        // Look for an existing record with the same name and type.
        let existing_match = existing
            .iter()
            .find(|r| r.name == spec.name && r.record_type == spec.record_type);

        match existing_match {
            None => {
                provider.create_record(zone_id, spec).await?;
                metrics::record_dns_upsert(
                    provider.name(),
                    &format!("{:?}", spec.record_type),
                    true,
                );
                result.created += 1;
            }
            Some(existing_rec) if existing_rec.content != spec.content => {
                provider
                    .update_record(zone_id, &existing_rec.id, spec)
                    .await?;
                metrics::record_dns_upsert(
                    provider.name(),
                    &format!("{:?}", spec.record_type),
                    true,
                );
                result.updated += 1;
            }
            Some(_) => {
                result.unchanged += 1;
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    use super::*;
    use crate::record::DnsRecord;

    // --- InMemoryProvider for apply_mail_records tests -----------------------

    #[derive(Default)]
    struct InMemoryProvider {
        records: Arc<Mutex<Vec<DnsRecord>>>,
        next_id: Arc<Mutex<u64>>,
    }

    impl InMemoryProvider {
        fn all_records(&self) -> Vec<DnsRecord> {
            self.records.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl DnsProvider for InMemoryProvider {
        fn name(&self) -> &'static str {
            "memory"
        }

        async fn zone_id(&self, _domain: &str) -> Result<String, AgDomainsError> {
            Ok("zone-1".to_owned())
        }

        async fn list_records(&self, _zone_id: &str) -> Result<Vec<DnsRecord>, AgDomainsError> {
            Ok(self.records.lock().unwrap().clone())
        }

        async fn create_record(
            &self,
            zone_id: &str,
            spec: &DnsRecordSpec,
        ) -> Result<DnsRecord, AgDomainsError> {
            let mut id_guard = self.next_id.lock().unwrap();
            *id_guard += 1;
            let record = DnsRecord {
                id: id_guard.to_string(),
                zone_id: zone_id.to_owned(),
                name: spec.name.clone(),
                record_type: spec.record_type.clone(),
                content: spec.content.clone(),
                ttl: spec.ttl,
                proxied: spec.proxied,
            };
            self.records.lock().unwrap().push(record.clone());
            Ok(record)
        }

        async fn update_record(
            &self,
            _zone_id: &str,
            record_id: &str,
            spec: &DnsRecordSpec,
        ) -> Result<DnsRecord, AgDomainsError> {
            let mut records = self.records.lock().unwrap();
            let rec = records
                .iter_mut()
                .find(|r| r.id == record_id)
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
            let mut records = self.records.lock().unwrap();
            records.retain(|r| r.id != record_id);
            Ok(())
        }
    }

    fn base_req() -> MailDnsRequirements {
        MailDnsRequirements {
            domain: "ejemplo.com".to_owned(),
            dkim_selectors: vec![DkimSelector {
                name: "s1".to_owned(),
                public_key_record: "v=DKIM1; k=rsa; p=MIIB...".to_owned(),
            }],
            spf_includes: vec!["sendgrid.net".to_owned()],
            spf_ips: vec!["192.0.2.1".to_owned()],
            dmarc_policy: DmarcPolicy::Quarantine,
            dmarc_rua: Some("admin@ejemplo.com".to_owned()),
        }
    }

    #[test]
    fn generates_three_records() {
        let records = generate_mail_records(&base_req());
        // SPF + 1 DKIM + DMARC
        assert_eq!(records.len(), 3);
    }

    #[test]
    fn spf_content() {
        let records = generate_mail_records(&base_req());
        let spf = records.iter().find(|r| r.name == "ejemplo.com").unwrap();
        assert!(
            spf.content.starts_with("v=spf1"),
            "SPF debe empezar con v=spf1"
        );
        assert!(spf.content.contains("include:sendgrid.net"));
        assert!(spf.content.contains("ip4:192.0.2.1"));
    }

    #[test]
    fn dkim_name_format() {
        let records = generate_mail_records(&base_req());
        let dkim = records
            .iter()
            .find(|r| r.name == "s1._domainkey.ejemplo.com")
            .unwrap();
        assert_eq!(dkim.record_type, RecordType::Txt);
    }

    #[test]
    fn dmarc_policy_and_rua() {
        let records = generate_mail_records(&base_req());
        let dmarc = records
            .iter()
            .find(|r| r.name == "_dmarc.ejemplo.com")
            .unwrap();
        assert!(dmarc.content.contains("p=quarantine"));
        assert!(dmarc.content.contains("rua=mailto:admin@ejemplo.com"));
    }

    #[test]
    fn dmarc_reject_no_rua() {
        let req = MailDnsRequirements {
            dmarc_policy: DmarcPolicy::Reject,
            dmarc_rua: None,
            ..base_req()
        };
        let records = generate_mail_records(&req);
        let dmarc = records
            .iter()
            .find(|r| r.name == "_dmarc.ejemplo.com")
            .unwrap();
        assert!(dmarc.content.contains("p=reject"));
        assert!(!dmarc.content.contains("rua="));
    }

    #[test]
    fn ipv6_spf_prefix() {
        let req = MailDnsRequirements {
            spf_ips: vec!["2001:db8::1".to_owned()],
            ..base_req()
        };
        let records = generate_mail_records(&req);
        let spf = records.iter().find(|r| r.name == "ejemplo.com").unwrap();
        assert!(spf.content.contains("ip6:2001:db8::1"));
    }

    // --- apply_mail_records tests --------------------------------------------

    #[tokio::test]
    async fn apply_creates_all_records_on_empty_zone() {
        let provider = InMemoryProvider::default();
        let result = apply_mail_records(&base_req(), &provider, "zone-1")
            .await
            .unwrap();
        assert_eq!(result.created, 3); // SPF + DKIM + DMARC
        assert_eq!(result.updated, 0);
        assert_eq!(result.unchanged, 0);
        assert_eq!(provider.all_records().len(), 3);
    }

    #[tokio::test]
    async fn apply_is_idempotent_when_content_matches() {
        let provider = InMemoryProvider::default();
        // First pass: creates everything
        apply_mail_records(&base_req(), &provider, "zone-1")
            .await
            .unwrap();
        // Second pass: nothing should change
        let result = apply_mail_records(&base_req(), &provider, "zone-1")
            .await
            .unwrap();
        assert_eq!(result.created, 0);
        assert_eq!(result.updated, 0);
        assert_eq!(result.unchanged, 3);
    }

    #[tokio::test]
    async fn apply_updates_record_when_content_changes() {
        let provider = InMemoryProvider::default();
        // Create the records with the base configuration.
        apply_mail_records(&base_req(), &provider, "zone-1")
            .await
            .unwrap();

        // Change the DMARC policy — the _dmarc record must be updated.
        let req_v2 = MailDnsRequirements {
            dmarc_policy: DmarcPolicy::Reject,
            ..base_req()
        };
        let result = apply_mail_records(&req_v2, &provider, "zone-1")
            .await
            .unwrap();
        assert_eq!(result.updated, 1); // only DMARC
        assert_eq!(result.unchanged, 2); // SPF and DKIM unchanged

        let dmarc = provider
            .all_records()
            .into_iter()
            .find(|r| r.name == "_dmarc.ejemplo.com")
            .unwrap();
        assert!(dmarc.content.contains("p=reject"));
    }

    #[tokio::test]
    async fn apply_adds_new_dkim_selector() {
        let provider = InMemoryProvider::default();
        apply_mail_records(&base_req(), &provider, "zone-1")
            .await
            .unwrap();

        // Add a second DKIM selector.
        let req_v2 = MailDnsRequirements {
            dkim_selectors: vec![
                DkimSelector {
                    name: "s1".to_owned(),
                    public_key_record: "v=DKIM1; k=rsa; p=MIIB...".to_owned(),
                },
                DkimSelector {
                    name: "s2".to_owned(),
                    public_key_record: "v=DKIM1; k=rsa; p=MIIC...".to_owned(),
                },
            ],
            ..base_req()
        };
        let result = apply_mail_records(&req_v2, &provider, "zone-1")
            .await
            .unwrap();
        assert_eq!(result.created, 1); // s2 is new
        assert_eq!(result.unchanged, 3); // SPF, s1, DMARC unchanged
        assert_eq!(provider.all_records().len(), 4);
    }
}
