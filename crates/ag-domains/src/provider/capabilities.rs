//! Provider capability registry (blueprint sections 10.2 and 11.2).
//!
//! Describes, per DNS provider, what Anti-Gravital can automate today and which
//! provider DNS features matter for generating correct instructions. Two
//! distinct axes are tracked and must not be conflated:
//!
//! - Adapter axis (`adapter`): what *Anti-Gravital* can do for this provider —
//!   read/apply records through a built-in adapter. Today only the manual flow
//!   (paste records) and the Cloudflare adapter exist.
//! - Provider-feature axis (`apex_alias`, `cname_flattening`, `dns01_automation`):
//!   facts about the *provider* that the DNS instruction engine uses.
//!
//! This is a conservative, living list: a provider absent here is handled by the
//! manual flow. Entries claim only well-established facts; uncertain features are
//! left `false`. Additional adapters (RFC-0011 phase E) flip the adapter axis as
//! they land — the list is not a promise of automation that does not exist.

use serde::{Deserialize, Serialize};

/// What Anti-Gravital can do for a provider through a built-in adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterSupport {
    /// No adapter: records are pasted manually (native default).
    Manual,
    /// Adapter can read and apply records.
    ReadApply,
}

/// Capabilities of a single DNS provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    /// Stable provider key (lowercase, e.g. `cloudflare`).
    pub provider: &'static str,
    /// What Anti-Gravital automates for this provider.
    pub adapter: AdapterSupport,
    /// Provider supports an apex ALIAS/ANAME or CNAME flattening (so the apex
    /// can target a hostname instead of fixed IPs).
    pub apex_alias: bool,
    /// Provider flattens CNAMEs at the apex.
    pub cname_flattening: bool,
    /// Anti-Gravital can complete a DNS-01 challenge automatically via the
    /// adapter (requires `ReadApply`).
    pub dns01_automation: bool,
    /// Anti-Gravital can export a BIND zone fragment for this provider. Always
    /// true: zone export is provider-agnostic.
    pub zone_export: bool,
    /// Domain purchase/transfer. Always false in v1 (no registrar; that is a
    /// future `ag-registrars`).
    pub domain_purchase: bool,
}

impl ProviderCapabilities {
    const fn manual(provider: &'static str, apex_alias: bool, cname_flattening: bool) -> Self {
        Self {
            provider,
            adapter: AdapterSupport::Manual,
            apex_alias,
            cname_flattening,
            dns01_automation: false,
            zone_export: true,
            domain_purchase: false,
        }
    }
}

/// The known-provider capability registry.
///
/// Order is stable for deterministic output (e.g. the REST endpoint and docs).
pub fn known_provider_capabilities() -> Vec<ProviderCapabilities> {
    vec![
        // Manual flow: works for every provider; no apex alias / flattening assumed.
        ProviderCapabilities::manual("manual", false, false),
        // Cloudflare: built-in adapter + CNAME flattening at apex + DNS-01 automation.
        ProviderCapabilities {
            provider: "cloudflare",
            adapter: AdapterSupport::ReadApply,
            apex_alias: true,
            cname_flattening: true,
            dns01_automation: true,
            zone_export: true,
            domain_purchase: false,
        },
        // Route 53: read/apply adapter (feature `route53`) + provider Alias at apex.
        ProviderCapabilities {
            provider: "route53",
            adapter: AdapterSupport::ReadApply,
            apex_alias: true,
            cname_flattening: false,
            dns01_automation: true,
            zone_export: true,
            domain_purchase: false,
        },
        // Google Cloud DNS: read/apply adapter (feature `google-cloud-dns`); no apex alias.
        ProviderCapabilities {
            provider: "google-cloud-dns",
            adapter: AdapterSupport::ReadApply,
            apex_alias: false,
            cname_flattening: false,
            dns01_automation: true,
            zone_export: true,
            domain_purchase: false,
        },
        // Azure DNS: read/apply adapter (feature `azure-dns`) + apex alias record sets.
        ProviderCapabilities {
            provider: "azure-dns",
            adapter: AdapterSupport::ReadApply,
            apex_alias: true,
            cname_flattening: false,
            dns01_automation: true,
            zone_export: true,
            domain_purchase: false,
        },
        // Namecheap: host-record API exists, but no Anti-Gravital adapter yet.
        ProviderCapabilities::manual("namecheap", false, false),
        // Hostinger: hPanel DNS; manual flow + BIND export.
        ProviderCapabilities::manual("hostinger", false, false),
        // Squarespace: shows required records; manual flow.
        ProviderCapabilities::manual("squarespace", false, false),
    ]
}

/// Looks up the capabilities for a provider key (case-insensitive).
pub fn capabilities_for(provider: &str) -> Option<ProviderCapabilities> {
    let key = provider.trim().to_ascii_lowercase();
    known_provider_capabilities()
        .into_iter()
        .find(|c| c.provider == key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_manual_and_cloudflare() {
        let all = known_provider_capabilities();
        assert!(all.iter().any(|c| c.provider == "manual"));
        let cf = capabilities_for("CloudFlare").unwrap();
        assert_eq!(cf.adapter, AdapterSupport::ReadApply);
        assert!(cf.cname_flattening);
        assert!(cf.dns01_automation);
    }

    #[test]
    fn unknown_provider_is_none() {
        assert!(capabilities_for("no-such-provider").is_none());
    }

    #[test]
    fn no_provider_claims_domain_purchase() {
        assert!(known_provider_capabilities()
            .iter()
            .all(|c| !c.domain_purchase));
    }

    #[test]
    fn dns01_automation_requires_an_adapter() {
        // A provider can only auto-complete DNS-01 if it has a read/apply adapter.
        for c in known_provider_capabilities() {
            if c.dns01_automation {
                assert_eq!(c.adapter, AdapterSupport::ReadApply, "{}", c.provider);
            }
        }
    }

    #[test]
    fn zone_export_is_universal() {
        assert!(known_provider_capabilities().iter().all(|c| c.zone_export));
    }

    #[test]
    fn serializes_to_json() {
        let cf = capabilities_for("cloudflare").unwrap();
        let json = serde_json::to_string(&cf).unwrap();
        assert!(json.contains("\"provider\":\"cloudflare\""));
        assert!(json.contains("\"adapter\":\"read_apply\""));
    }
}
