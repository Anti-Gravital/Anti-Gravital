//! Destination MX resolution and `site_name` rollup for the native MTA.
//!
//! The delivery loop resolves the MX hosts of a recipient domain, orders them
//! by preference, and computes a `site_name` so that traffic shaping (a later
//! phase) can throttle all domains that share the same MX set as one
//! destination. The ordering and rollup are pure and unit-tested; the DNS
//! lookup itself uses `hickory-resolver` and is exercised by an ignored
//! network test.

use std::net::SocketAddr;

use hickory_resolver::{
    config::{NameServerConfig, ResolverConfig, ResolverOpts},
    net::runtime::TokioRuntimeProvider,
    proto::rr::RData,
    TokioResolver,
};

use crate::error::AgMailError;

/// A resolved mail exchanger with its preference (lower is higher priority).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MxHost {
    /// MX preference value; the delivery loop tries the lowest first.
    pub preference: u16,
    /// Mail host name, normalized (lowercased, no trailing dot).
    pub exchange: String,
}

impl MxHost {
    /// Builds a host, normalizing the exchange name.
    pub fn new(preference: u16, exchange: impl Into<String>) -> Self {
        Self {
            preference,
            exchange: normalize_host(&exchange.into()),
        }
    }
}

/// Lowercases a host name and removes a single trailing dot.
fn normalize_host(host: &str) -> String {
    host.trim().trim_end_matches('.').to_ascii_lowercase()
}

/// Orders MX hosts by ascending preference, breaking ties by name so the
/// result is deterministic.
pub fn sort_by_preference(mut hosts: Vec<MxHost>) -> Vec<MxHost> {
    hosts.sort_by(|a, b| {
        a.preference
            .cmp(&b.preference)
            .then_with(|| a.exchange.cmp(&b.exchange))
    });
    hosts
}

/// Computes the `site_name` rollup for a domain given its MX hosts.
///
/// All domains that share the same set of MX exchanges collapse to the same
/// `site_name`, so shaping applies to the shared destination rather than to
/// each domain. The identifier is the sorted, deduplicated, lowercased set of
/// exchanges joined by `,`. When a domain has no MX, RFC 5321 section 5.1
/// makes the domain itself the implicit mail host, and the rollup falls back
/// to the domain name. A smarter common-suffix merge is future work
/// (RFC-0009).
pub fn site_name(domain: &str, hosts: &[MxHost]) -> String {
    if hosts.is_empty() {
        return normalize_host(domain);
    }
    let mut names: Vec<String> = hosts.iter().map(|h| h.exchange.clone()).collect();
    names.sort();
    names.dedup();
    names.join(",")
}

/// Returns the MX hosts for a domain, applying the implicit-MX fallback.
///
/// On an empty MX set the domain itself is returned as a single host
/// (preference 0), per RFC 5321 section 5.1.
pub fn with_implicit_mx(domain: &str, hosts: Vec<MxHost>) -> Vec<MxHost> {
    if hosts.is_empty() {
        vec![MxHost::new(0, domain)]
    } else {
        hosts
    }
}

/// Resolver configuration for the native MTA.
#[derive(Debug, Clone)]
pub struct ResolverConfigMta {
    /// Recursive resolver to query (UDP/53). Defaults to a public resolver so
    /// behavior does not depend on the host's `/etc/resolv.conf`.
    pub nameserver: SocketAddr,
}

impl Default for ResolverConfigMta {
    fn default() -> Self {
        Self {
            nameserver: "1.1.1.1:53".parse().expect("valid default nameserver"),
        }
    }
}

/// Resolves the MX hosts of a domain, ordered by preference, with the
/// implicit-MX fallback applied. Performs a live DNS query.
pub async fn resolve_mx(
    domain: &str,
    config: &ResolverConfigMta,
) -> Result<Vec<MxHost>, AgMailError> {
    let ns = NameServerConfig::udp(config.nameserver.ip());
    let resolver_config = ResolverConfig::from_parts(None, vec![], vec![ns]);
    let resolver =
        TokioResolver::builder_with_config(resolver_config, TokioRuntimeProvider::default())
            .with_options(ResolverOpts::default())
            .build()
            .map_err(|e| AgMailError::Dns(e.to_string()))?;

    let hosts = match resolver.mx_lookup(domain).await {
        Ok(lookup) => lookup
            .answers()
            .iter()
            .filter_map(|record| match &record.data {
                RData::MX(mx) => Some(MxHost::new(mx.preference, mx.exchange.to_string())),
                _ => None,
            })
            .collect::<Vec<_>>(),
        Err(err) if err.is_no_records_found() => {
            // No MX records: fall back to the implicit MX (RFC 5321 s.5.1).
            Vec::new()
        }
        Err(err) => return Err(AgMailError::Dns(err.to_string())),
    };

    Ok(sort_by_preference(with_implicit_mx(domain, hosts)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_dot_and_lowercases() {
        assert_eq!(normalize_host("MX1.Example.COM."), "mx1.example.com");
    }

    #[test]
    fn sort_orders_by_preference_then_name() {
        let hosts = vec![
            MxHost::new(20, "b.mx.com"),
            MxHost::new(10, "z.mx.com"),
            MxHost::new(10, "a.mx.com"),
        ];
        let sorted = sort_by_preference(hosts);
        assert_eq!(sorted[0].exchange, "a.mx.com");
        assert_eq!(sorted[1].exchange, "z.mx.com");
        assert_eq!(sorted[2].exchange, "b.mx.com");
    }

    #[test]
    fn site_name_is_order_independent() {
        let a = vec![
            MxHost::new(10, "alt1.aspmx.l.google.com"),
            MxHost::new(1, "aspmx.l.google.com"),
        ];
        let b = vec![
            MxHost::new(5, "aspmx.l.google.com"),
            MxHost::new(99, "alt1.aspmx.l.google.com"),
        ];
        // Same MX set, different preferences/order => same site_name.
        assert_eq!(site_name("gmail.com", &a), site_name("googlemail.com", &b));
    }

    #[test]
    fn site_name_dedupes() {
        let hosts = vec![MxHost::new(10, "mx.x.com"), MxHost::new(20, "MX.x.com.")];
        assert_eq!(site_name("x.com", &hosts), "mx.x.com");
    }

    #[test]
    fn site_name_empty_falls_back_to_domain() {
        assert_eq!(site_name("No-MX.Example.com", &[]), "no-mx.example.com");
    }

    #[test]
    fn implicit_mx_used_when_empty() {
        let hosts = with_implicit_mx("example.com", vec![]);
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].exchange, "example.com");
        assert_eq!(hosts[0].preference, 0);
    }

    #[test]
    fn implicit_mx_noop_when_present() {
        let hosts = with_implicit_mx("example.com", vec![MxHost::new(10, "mx.example.com")]);
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].exchange, "mx.example.com");
    }

    #[tokio::test]
    #[ignore = "requires outbound DNS (UDP/53); run with --include-ignored"]
    async fn resolve_mx_live_gmail() {
        let hosts = resolve_mx("gmail.com", &ResolverConfigMta::default())
            .await
            .expect("MX lookup succeeds");
        assert!(!hosts.is_empty());
        assert!(hosts.iter().any(|h| h.exchange.ends_with("google.com")));
    }
}
