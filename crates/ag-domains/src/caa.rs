//! CAA preflight before ACME certificate issuance.
//!
//! Certification Authority Authorization (RFC 8659) records let a domain owner
//! restrict which CAs may issue certificates. Before ordering a certificate,
//! `ag-domains` queries CAA records and reports whether the configured CA is
//! authorized, producing an actionable error instead of a late ACME failure
//! (RFC-0011 section 5; blueprint section 13.5).
//!
//! The decision logic is pure and unit-tested. The live DNS query is gated
//! behind the `propagation` feature. Preflight is advisory: when records cannot
//! be parsed it does not block issuance (ACME remains the authoritative gate).

/// A simplified CAA `issue` / `issuewild` record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaaIssue {
    /// Authorized issuer domain, or `None` for an explicit `";"` (no issuance).
    pub issuer: Option<String>,
}

/// Result of a CAA preflight evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaaDecision {
    /// Whether the configured CA is allowed to issue.
    pub allowed: bool,
    /// The issuer domains observed (empty means unrestricted).
    pub authorized_issuers: Vec<String>,
    /// Human-readable explanation.
    pub reason: String,
}

/// Evaluates whether `ca_domain` (e.g. `letsencrypt.org`) may issue, given the
/// observed CAA issue/issuewild records for the domain.
///
/// Per RFC 8659: no relevant CAA records means any CA may issue. If issue
/// records are present, the CA must match one of the listed issuer domains.
pub fn evaluate(ca_domain: &str, issues: &[CaaIssue]) -> CaaDecision {
    if issues.is_empty() {
        return CaaDecision {
            allowed: true,
            authorized_issuers: Vec::new(),
            reason: "no CAA issue records; any CA may issue".to_owned(),
        };
    }

    let authorized_issuers: Vec<String> = issues.iter().filter_map(|i| i.issuer.clone()).collect();

    let target = ca_domain.trim().to_ascii_lowercase();
    let allowed = authorized_issuers
        .iter()
        .any(|i| i.trim().to_ascii_lowercase() == target);

    let reason = if allowed {
        format!("CAA authorizes {ca_domain}")
    } else if authorized_issuers.is_empty() {
        format!("CAA explicitly forbids issuance (';'); {ca_domain} cannot issue")
    } else {
        format!(
            "CAA does not authorize {ca_domain}; allowed: {}",
            authorized_issuers.join(", ")
        )
    };

    CaaDecision {
        allowed,
        authorized_issuers,
        reason,
    }
}

#[cfg(feature = "propagation")]
mod live {
    use super::*;
    use std::net::SocketAddr;

    use hickory_resolver::{
        config::{NameServerConfig, ResolverConfig, ResolverOpts},
        net::runtime::TokioRuntimeProvider,
        proto::rr::{RData, RecordType},
        TokioResolver,
    };

    use crate::error::AgDomainsError;

    /// Queries CAA records for `domain` via `resolver_addr` and evaluates whether
    /// `ca_domain` may issue.
    ///
    /// A lookup failure is treated as "no records" (advisory preflight): it
    /// returns an unrestricted decision rather than blocking issuance.
    pub async fn preflight(
        resolver_addr: SocketAddr,
        domain: &str,
        ca_domain: &str,
    ) -> Result<CaaDecision, AgDomainsError> {
        let ns = NameServerConfig::udp(resolver_addr.ip());
        let config = ResolverConfig::from_parts(None, vec![], vec![ns]);
        let mut opts = ResolverOpts::default();
        opts.cache_size = 0;
        let resolver = TokioResolver::builder_with_config(config, TokioRuntimeProvider::default())
            .with_options(opts)
            .build()
            .map_err(|e| AgDomainsError::Config(e.to_string()))?;

        let fqdn = if domain.ends_with('.') {
            domain.to_owned()
        } else {
            format!("{domain}.")
        };

        let issues = match resolver.lookup(fqdn.as_str(), RecordType::CAA).await {
            Ok(lookup) => lookup
                .answers()
                .iter()
                .filter_map(|record| {
                    if let RData::CAA(caa) = &record.data {
                        // value_as_issue succeeds for both issue and issuewild.
                        caa.value_as_issue().ok().map(|(name, _)| CaaIssue {
                            issuer: name.map(|n| n.to_string().trim_end_matches('.').to_owned()),
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            Err(_) => Vec::new(),
        };

        Ok(evaluate(ca_domain, &issues))
    }
}

#[cfg(feature = "propagation")]
pub use live::preflight;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_records_allows_any() {
        let d = evaluate("letsencrypt.org", &[]);
        assert!(d.allowed);
    }

    #[test]
    fn matching_issuer_allows() {
        let issues = vec![CaaIssue {
            issuer: Some("letsencrypt.org".to_owned()),
        }];
        let d = evaluate("letsencrypt.org", &issues);
        assert!(d.allowed);
    }

    #[test]
    fn non_matching_issuer_blocks() {
        let issues = vec![CaaIssue {
            issuer: Some("digicert.com".to_owned()),
        }];
        let d = evaluate("letsencrypt.org", &issues);
        assert!(!d.allowed);
        assert!(d.reason.contains("digicert.com"));
    }

    #[test]
    fn explicit_forbid_blocks() {
        let issues = vec![CaaIssue { issuer: None }];
        let d = evaluate("letsencrypt.org", &issues);
        assert!(!d.allowed);
        assert!(d.reason.contains("forbids"));
    }

    #[test]
    fn case_insensitive_match() {
        let issues = vec![CaaIssue {
            issuer: Some("LetsEncrypt.ORG".to_owned()),
        }];
        assert!(evaluate("letsencrypt.org", &issues).allowed);
    }
}
