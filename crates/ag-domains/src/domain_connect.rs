//! Domain Connect discovery and template application (blueprint section 12).
//!
//! [Domain Connect](https://www.domainconnect.org/) lets an operator authorize a
//! DNS template at their provider instead of pasting records by hand. The flow:
//!
//! 1. **Discover** the provider's `_domainconnect` TXT record for the domain to
//!    learn the Domain Connect API host ([`parse_domainconnect_record`]).
//! 2. Fetch the provider **settings** to learn the synchronous-UX base URL
//!    ([`parse_settings`]) — the HTTP fetch is the integration boundary; parsing
//!    is here and unit-tested.
//! 3. Build the **template variables** from the Anti-Gravital records, MX-safe so
//!    existing email is never disturbed ([`template_variables`]).
//! 4. **Redirect** the operator to the provider's synchronous authorization flow
//!    ([`build_sync_apply_url`]).
//! 5. **Verify independently**: Domain Connect accelerates setup, but the
//!    ownership token and routing records it applies are the same ones
//!    [`crate::instructions::generate_instructions`] produces, so the existing
//!    ownership/diagnostics verification path confirms the result unchanged.
//!
//! Everything here is provider-agnostic and pure; no provider is reimplemented.

use serde::Deserialize;

use crate::error::AgDomainsError;
use crate::instructions::DnsInstructions;
use crate::record::RecordType;

/// Discovered Domain Connect entry point for a domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainConnectDiscovery {
    /// The Domain Connect API host published in `_domainconnect.<domain>`.
    pub api_host: String,
}

/// DNS name of the discovery record (Domain Connect spec section "Discovery").
pub fn domainconnect_record_name(domain: &str) -> String {
    format!("_domainconnect.{}", domain.trim_end_matches('.'))
}

/// Parses the value of a `_domainconnect` TXT record into the API host.
///
/// The record value is the hostname of the provider's Domain Connect API
/// (e.g. `domainconnect.provider.example`). Rejects empty values, embedded
/// schemes/paths and whitespace, so a malformed record cannot redirect the flow
/// to an arbitrary URL.
pub fn parse_domainconnect_record(
    txt_value: &str,
) -> Result<DomainConnectDiscovery, AgDomainsError> {
    let host = txt_value.trim().trim_end_matches('.');
    if host.is_empty()
        || host.contains("://")
        || host.contains('/')
        || host.contains(char::is_whitespace)
        || !host.contains('.')
    {
        return Err(AgDomainsError::Config(format!(
            "invalid _domainconnect record value: {txt_value:?}"
        )));
    }
    Ok(DomainConnectDiscovery {
        api_host: host.to_ascii_lowercase(),
    })
}

/// Provider settings returned by the Domain Connect API `/v2/{domain}/settings`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainConnectSettings {
    /// Human-readable provider name.
    pub provider_name: String,
    /// Base URL of the synchronous (interactive) authorization UX.
    pub url_sync_ux: String,
    /// Base URL of the asynchronous (OAuth) authorization UX, if offered.
    pub url_async_ux: Option<String>,
    /// Base URL of the provider's API, if advertised.
    pub url_api: Option<String>,
}

#[derive(Deserialize)]
struct RawSettings {
    #[serde(rename = "providerName")]
    provider_name: String,
    #[serde(rename = "urlSyncUX")]
    url_sync_ux: String,
    #[serde(rename = "urlAsyncUX")]
    url_async_ux: Option<String>,
    #[serde(rename = "urlAPI")]
    url_api: Option<String>,
}

/// Parses the Domain Connect provider settings JSON.
pub fn parse_settings(json: &str) -> Result<DomainConnectSettings, AgDomainsError> {
    let raw: RawSettings = serde_json::from_str(json).map_err(|e| {
        AgDomainsError::Config(format!("invalid Domain Connect settings JSON: {e}"))
    })?;
    if raw.url_sync_ux.trim().is_empty() {
        return Err(AgDomainsError::Config(
            "Domain Connect settings missing urlSyncUX".to_owned(),
        ));
    }
    Ok(DomainConnectSettings {
        provider_name: raw.provider_name,
        url_sync_ux: raw.url_sync_ux.trim_end_matches('/').to_owned(),
        url_async_ux: raw.url_async_ux,
        url_api: raw.url_api,
    })
}

/// A Domain Connect template to apply, identified by provider/service id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyTemplate {
    /// The template's provider id (registered with the DNS provider).
    pub provider_id: String,
    /// The template's service id.
    pub service_id: String,
    /// Template variables (`name` -> `value`) substituted into the template.
    pub variables: Vec<(String, String)>,
}

/// Derives Domain Connect template variables from Anti-Gravital DNS instructions.
///
/// Maps the routing and ownership records to template variables. The result is
/// **MX-safe by construction**: only A/AAAA/CNAME/TXT records are ever emitted,
/// so applying the template never adds or removes MX records and existing email
/// keeps working. Variable names follow a stable convention
/// (`ip`/`ip6`/`cname`/`www`/`txtname`/`txtvalue`); the registered template
/// references the same names.
pub fn template_variables(instructions: &DnsInstructions) -> Vec<(String, String)> {
    let mut vars: Vec<(String, String)> = Vec::new();
    for rec in &instructions.routing {
        match rec.record_type {
            RecordType::A => vars.push(("ip".to_owned(), rec.value.clone())),
            RecordType::Aaaa => vars.push(("ip6".to_owned(), rec.value.clone())),
            RecordType::Cname => {
                // Distinguish the `www` companion from the primary CNAME target.
                let key = if rec.name.starts_with("www.") {
                    "www"
                } else {
                    "cname"
                };
                vars.push((key.to_owned(), rec.value.clone()));
            }
            // MX-safe: routing never contains MX/TXT; guard anyway.
            RecordType::Mx | RecordType::Txt => {}
        }
    }
    vars.push(("txtname".to_owned(), instructions.ownership.name.clone()));
    vars.push(("txtvalue".to_owned(), instructions.ownership.value.clone()));
    vars
}

/// Builds the synchronous Domain Connect "apply template" URL the operator is
/// redirected to (Domain Connect spec, synchronous flow).
///
/// Format: `{urlSyncUX}/v2/domainTemplates/providers/{providerId}/services/{serviceId}/apply?domain={domain}&{vars}`.
/// All query values are percent-encoded.
pub fn build_sync_apply_url(
    settings: &DomainConnectSettings,
    template: &ApplyTemplate,
    domain: &str,
) -> String {
    let mut url = format!(
        "{}/v2/domainTemplates/providers/{}/services/{}/apply?domain={}",
        settings.url_sync_ux,
        encode(&template.provider_id),
        encode(&template.service_id),
        encode(domain),
    );
    for (name, value) in &template.variables {
        url.push('&');
        url.push_str(&encode(name));
        url.push('=');
        url.push_str(&encode(value));
    }
    url
}

/// Percent-encodes a query component, leaving only the RFC 3986 unreserved set.
fn encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hostname::Hostname;
    use crate::instructions::{generate_instructions, EdgeTargets};
    use std::net::Ipv4Addr;

    #[test]
    fn discovery_record_name() {
        assert_eq!(
            domainconnect_record_name("example.com"),
            "_domainconnect.example.com"
        );
    }

    #[test]
    fn parses_valid_discovery_record() {
        let d = parse_domainconnect_record("domainconnect.Provider.example.").unwrap();
        assert_eq!(d.api_host, "domainconnect.provider.example");
    }

    #[test]
    fn rejects_malformed_discovery_records() {
        assert!(parse_domainconnect_record("").is_err());
        assert!(parse_domainconnect_record("https://evil.example/x").is_err());
        assert!(parse_domainconnect_record("has space.example").is_err());
        assert!(parse_domainconnect_record("nolabel").is_err());
    }

    #[test]
    fn parses_settings() {
        let json = r#"{
            "providerName": "Example DNS",
            "urlSyncUX": "https://dc.example.com/sync/",
            "urlAsyncUX": "https://dc.example.com/async",
            "urlAPI": "https://api.dc.example.com"
        }"#;
        let s = parse_settings(json).unwrap();
        assert_eq!(s.provider_name, "Example DNS");
        // Trailing slash is normalized off so URL building does not double it.
        assert_eq!(s.url_sync_ux, "https://dc.example.com/sync");
        assert_eq!(s.url_api.as_deref(), Some("https://api.dc.example.com"));
    }

    #[test]
    fn settings_require_sync_ux() {
        let json = r#"{"providerName":"X","urlSyncUX":""}"#;
        assert!(parse_settings(json).is_err());
    }

    fn instructions(host: &str) -> DnsInstructions {
        let h = Hostname::parse(host).unwrap();
        let targets =
            EdgeTargets::new("edge.example-cloud.net").with_ipv4(Ipv4Addr::new(203, 0, 113, 10));
        generate_instructions(&h, &targets, "ag-verification=tok123")
    }

    #[test]
    fn template_variables_are_mx_safe_and_carry_ownership() {
        // Apex emits A + www CNAME + ownership TXT; never MX.
        let vars = template_variables(&instructions("example.com"));
        assert!(vars.iter().any(|(k, v)| k == "ip" && v == "203.0.113.10"));
        assert!(vars.iter().any(|(k, _)| k == "www"));
        assert!(vars
            .iter()
            .any(|(k, v)| k == "txtvalue" && v == "ag-verification=tok123"));
        // MX safety: no variable ever encodes an MX record.
        assert!(!vars.iter().any(|(k, _)| k == "mx"));
    }

    #[test]
    fn subdomain_template_uses_cname() {
        let vars = template_variables(&instructions("api.example.com"));
        assert!(vars
            .iter()
            .any(|(k, v)| k == "cname" && v == "edge.example-cloud.net"));
    }

    #[test]
    fn builds_sync_apply_url_with_encoding() {
        let settings = DomainConnectSettings {
            provider_name: "X".to_owned(),
            url_sync_ux: "https://dc.example.com/sync".to_owned(),
            url_async_ux: None,
            url_api: None,
        };
        let template = ApplyTemplate {
            provider_id: "anti-gravital.dev".to_owned(),
            service_id: "attach".to_owned(),
            variables: vec![("txtvalue".to_owned(), "ag-verification=a=b".to_owned())],
        };
        let url = build_sync_apply_url(&settings, &template, "example.com");
        assert!(url.starts_with(
            "https://dc.example.com/sync/v2/domainTemplates/providers/anti-gravital.dev/services/attach/apply?domain=example.com"
        ));
        // The `=` inside the token is percent-encoded so it cannot break parsing.
        assert!(url.ends_with("&txtvalue=ag-verification%3Da%3Db"));
    }

    #[test]
    fn ownership_variable_matches_independent_verification_record() {
        // The Domain Connect template carries exactly the ownership record the
        // existing verification path expects, so verification is reused unchanged.
        let ins = instructions("api.example.com");
        let vars = template_variables(&ins);
        let txtname = vars
            .iter()
            .find(|(k, _)| k == "txtname")
            .map(|(_, v)| v.clone());
        let txtvalue = vars
            .iter()
            .find(|(k, _)| k == "txtvalue")
            .map(|(_, v)| v.clone());
        assert_eq!(txtname, Some(ins.ownership.name.clone()));
        assert_eq!(txtvalue, Some(ins.ownership.value.clone()));
    }
}
