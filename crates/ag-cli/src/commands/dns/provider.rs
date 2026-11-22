//! Optional provider adapter, contacted only on explicit `--provider` opt-in.

use ag_domains::provider::DnsProvider;
use ag_domains::record::DnsRecordSpec;

use super::model::*;

/// Local-only summary of the optional adapter step. Never contains a
/// credential, a provider zone id or any adapter response body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProviderSummary {
    pub(crate) created: usize,
    pub(crate) updated: usize,
    pub(crate) unchanged: usize,
}

pub(crate) async fn apply_provider(
    plan: &DnsPlan,
    provider: &str,
    zone_id: Option<&str>,
) -> Result<Option<ProviderSummary>, String> {
    if provider == "none" {
        return Ok(None);
    }
    if provider != "cloudflare" {
        return Err(format!(
            "provider '{provider}' is not available in this build; supported opt-in providers: \
             none, cloudflare"
        ));
    }

    let token = std::env::var("AG_CLOUDFLARE_TOKEN").map_err(|_| {
        "provider 'cloudflare' requires AG_CLOUDFLARE_TOKEN in the environment; no credential is \
         read from the manifest"
            .to_owned()
    })?;
    let zone_id = zone_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            "provider 'cloudflare' requires --zone-id or AG_DNS_ZONE_ID; no provider zone id is \
             written to any generated file"
                .to_owned()
        })?;
    if token.is_empty() {
        return Err("AG_CLOUDFLARE_TOKEN is empty".to_owned());
    }

    let adapter = ag_domains::provider::cloudflare::CloudflareProvider::new(token);
    let existing = adapter
        .list_records(zone_id)
        .await
        .map_err(|e| format!("provider list_records failed: {e}"))?;

    let mut summary = ProviderSummary {
        created: 0,
        updated: 0,
        unchanged: 0,
    };
    for record in &plan.records {
        let spec = DnsRecordSpec {
            name: record.name.clone(),
            record_type: record.record_type.clone(),
            content: record.value.clone(),
            ttl: record.ttl,
            proxied: false,
        };
        let unchanged = existing.iter().any(|current| {
            current.name == spec.name
                && current.record_type == spec.record_type
                && current.content == spec.content
                && current.ttl == spec.ttl
        });
        if unchanged {
            summary.unchanged += 1;
            continue;
        }
        let changed = existing
            .iter()
            .find(|current| current.name == spec.name && current.record_type == spec.record_type);
        match changed {
            Some(current) => {
                adapter
                    .update_record(zone_id, &current.id, &spec)
                    .await
                    .map_err(|e| format!("provider update_record failed: {e}"))?;
                summary.updated += 1;
            }
            None => {
                adapter
                    .create_record(zone_id, &spec)
                    .await
                    .map_err(|e| format!("provider create_record failed: {e}"))?;
                summary.created += 1;
            }
        }
    }
    Ok(Some(summary))
}
