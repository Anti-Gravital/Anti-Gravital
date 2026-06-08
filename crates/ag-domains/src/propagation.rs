//! DNS propagation verification against multiple public resolvers.
//!
//! Used by `ag deploy` to block until the domain responds
//! correctly on at least N public resolvers. This prevents the
//! deployment from proceeding before DNS is visible to users.
//!
//! # Example
//!
//! ```rust,no_run
//! use ag_domains::propagation::{PropagationChecker, DEFAULT_RESOLVERS};
//!
//! #[tokio::main]
//! async fn main() {
//!     let checker = PropagationChecker::new(DEFAULT_RESOLVERS, 2);
//!     checker
//!         .wait_for_txt("_acme-challenge.ejemplo.com", "valor-esperado", 20)
//!         .await
//!         .expect("propagacion completada");
//! }
//! ```

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use hickory_resolver::{
    config::{NameServerConfig, ResolverConfig, ResolverOpts},
    net::runtime::TokioRuntimeProvider,
    proto::rr::{RData, RecordType as HickoryRecordType},
    TokioResolver,
};
use tracing::{debug, info};

use crate::error::AgDomainsError;
use crate::record::RecordType;

/// Public resolvers used by default: Google and Cloudflare.
pub const DEFAULT_RESOLVERS: &[SocketAddr] = &[
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53),
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 4, 4)), 53),
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 53),
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 0, 0, 1)), 53),
];

/// Result of a propagation check.
#[derive(Debug, Clone)]
pub struct PropagationResult {
    /// How many resolvers already see the expected value.
    pub confirmed: usize,
    /// Total resolvers queried.
    pub total: usize,
}

impl PropagationResult {
    /// `true` if all resolvers confirmed the value.
    pub fn is_fully_propagated(&self) -> bool {
        self.confirmed == self.total
    }

    /// Propagation percentage (0.0 - 1.0).
    pub fn ratio(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        self.confirmed as f64 / self.total as f64
    }
}

/// Verifies DNS propagation against a set of configured resolvers.
pub struct PropagationChecker {
    resolvers: Vec<TokioResolver>,
    min_confirmed: usize,
}

impl PropagationChecker {
    /// Creates a checker with the given nameservers and minimum confirmations.
    ///
    /// `nameservers` is a list of `SocketAddr` (IP:port) of DNS resolvers.
    /// `min_confirmed` is how many of them must respond correctly.
    pub fn new(nameservers: &[SocketAddr], min_confirmed: usize) -> Self {
        let resolvers = nameservers
            .iter()
            .map(|addr| build_resolver(*addr))
            .collect();
        Self {
            resolvers,
            min_confirmed,
        }
    }

    /// Queries a TXT record on all resolvers and counts how many
    /// contain `expected_value`.
    pub async fn check_txt(&self, name: &str, expected_value: &str) -> PropagationResult {
        let mut confirmed = 0usize;
        let fqdn = ensure_fqdn(name);

        for (i, resolver) in self.resolvers.iter().enumerate() {
            let found = resolver
                .txt_lookup(fqdn.as_str())
                .await
                .map(|lookup| {
                    lookup.answers().iter().any(|record| {
                        if let RData::TXT(txt) = &record.data {
                            txt.txt_data.iter().any(|bytes| {
                                std::str::from_utf8(bytes)
                                    .map(|s| s == expected_value)
                                    .unwrap_or(false)
                            })
                        } else {
                            false
                        }
                    })
                })
                .unwrap_or(false);

            debug!(
                resolver_index = i,
                name = fqdn,
                expected = expected_value,
                found,
                "consulta TXT"
            );

            if found {
                confirmed += 1;
            }
        }

        PropagationResult {
            confirmed,
            total: self.resolvers.len(),
        }
    }

    /// Queries an A record on all resolvers and counts how many
    /// return `expected_ip`.
    pub async fn check_a(&self, name: &str, expected_ip: std::net::IpAddr) -> PropagationResult {
        let mut confirmed = 0usize;
        let fqdn = ensure_fqdn(name);

        for resolver in &self.resolvers {
            let found = resolver
                .lookup_ip(fqdn.as_str())
                .await
                .map(|lookup| lookup.iter().any(|ip| ip == expected_ip))
                .unwrap_or(false);

            if found {
                confirmed += 1;
            }
        }

        PropagationResult {
            confirmed,
            total: self.resolvers.len(),
        }
    }

    /// Waits until `min_confirmed` resolvers see the TXT value,
    /// retrying every `retry_interval` seconds.
    ///
    /// `max_attempts` limits the number of probes to avoid infinite loops.
    pub async fn wait_for_txt(
        &self,
        name: &str,
        expected_value: &str,
        max_attempts: u32,
    ) -> Result<PropagationResult, AgDomainsError> {
        let retry_interval = Duration::from_secs(5);

        for attempt in 1..=max_attempts {
            let result = self.check_txt(name, expected_value).await;
            info!(
                attempt,
                max_attempts,
                confirmed = result.confirmed,
                total = result.total,
                name,
                "sondeo de propagacion DNS"
            );

            if result.confirmed >= self.min_confirmed {
                return Ok(result);
            }

            if attempt < max_attempts {
                tokio::time::sleep(retry_interval).await;
            }
        }

        Err(AgDomainsError::PropagationPending(format!(
            "{name}: solo {}/{} resolvers confirmaron tras {max_attempts} intentos",
            // We do a final check for the message.
            self.check_txt(name, expected_value).await.confirmed,
            self.resolvers.len(),
        )))
    }
}

// ---- internal helpers -------------------------------------------------------

fn build_resolver(addr: SocketAddr) -> TokioResolver {
    let ns = NameServerConfig::udp(addr.ip());
    let config = ResolverConfig::from_parts(None, vec![], vec![ns]);

    let mut opts = ResolverOpts::default();
    // Do not use a cache so we measure real propagation.
    opts.cache_size = 0;

    TokioResolver::builder_with_config(config, TokioRuntimeProvider::default())
        .with_options(opts)
        .build()
        .expect("resolver configuration is always valid")
}

fn ensure_fqdn(name: &str) -> String {
    if name.ends_with('.') {
        name.to_owned()
    } else {
        format!("{name}.")
    }
}

/// Extracts the observed value of a DNS record for the expected `record_type`.
///
/// Pure (no I/O), so the per-type parsing is unit-tested deterministically;
/// [`lookup_observed`] feeds it the records returned by a resolver.
fn rdata_value(record_type: RecordType, rdata: &RData) -> Option<String> {
    match (record_type, rdata) {
        (RecordType::A, RData::A(a)) => Some(a.0.to_string()),
        (RecordType::Aaaa, RData::AAAA(a)) => Some(a.0.to_string()),
        (RecordType::Cname, RData::CNAME(_)) => {
            Some(rdata.to_string().trim_end_matches('.').to_owned())
        }
        (RecordType::Txt, RData::TXT(txt)) => Some(
            txt.txt_data
                .iter()
                .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
                .collect::<Vec<_>>()
                .join(""),
        ),
        _ => None,
    }
}

/// Looks up the observed values of one record `name`/`record_type` at a single
/// resolver. Used by `ag domains diagnose` to compare expected vs observed DNS
/// (blueprint section 16.3). Returns an empty vector on lookup failure (the
/// caller renders that as "missing").
pub async fn lookup_observed(
    resolver_addr: SocketAddr,
    name: &str,
    record_type: RecordType,
) -> Vec<String> {
    let hickory_type = match record_type {
        RecordType::A => HickoryRecordType::A,
        RecordType::Aaaa => HickoryRecordType::AAAA,
        RecordType::Cname => HickoryRecordType::CNAME,
        RecordType::Txt => HickoryRecordType::TXT,
        // MX is not used by diagnose; avoid a network call.
        RecordType::Mx => return Vec::new(),
    };

    let resolver = build_resolver(resolver_addr);
    let fqdn = ensure_fqdn(name);

    match resolver.lookup(fqdn.as_str(), hickory_type).await {
        Ok(lookup) => lookup
            .answers()
            .iter()
            .filter_map(|record| rdata_value(record_type.clone(), &record.data))
            .collect(),
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rdata_value_parses_each_type() {
        use hickory_resolver::proto::rr::rdata::{CNAME, TXT};
        use hickory_resolver::proto::rr::Name;
        use std::net::{Ipv4Addr, Ipv6Addr};

        let a = RData::A(Ipv4Addr::new(203, 0, 113, 10).into());
        assert_eq!(
            rdata_value(RecordType::A, &a).as_deref(),
            Some("203.0.113.10")
        );
        // Type mismatch yields None.
        assert_eq!(rdata_value(RecordType::Aaaa, &a), None);

        let aaaa = RData::AAAA("2001:db8::10".parse::<Ipv6Addr>().unwrap().into());
        assert_eq!(
            rdata_value(RecordType::Aaaa, &aaaa).as_deref(),
            Some("2001:db8::10")
        );

        let cname = RData::CNAME(CNAME(Name::from_ascii("edge.example-cloud.net.").unwrap()));
        assert_eq!(
            rdata_value(RecordType::Cname, &cname).as_deref(),
            Some("edge.example-cloud.net")
        );

        let txt = RData::TXT(TXT::new(vec!["ag-verification=tok".to_owned()]));
        assert_eq!(
            rdata_value(RecordType::Txt, &txt).as_deref(),
            Some("ag-verification=tok")
        );
    }

    #[tokio::test]
    async fn lookup_observed_mx_returns_empty_without_network() {
        // MX is not modelled by diagnose; the function returns early, no lookup.
        let addr = "127.0.0.1:53".parse().unwrap();
        assert!(lookup_observed(addr, "example.com", RecordType::Mx)
            .await
            .is_empty());
    }

    #[test]
    fn ensure_fqdn_adds_dot() {
        assert_eq!(ensure_fqdn("ejemplo.com"), "ejemplo.com.");
        assert_eq!(ensure_fqdn("ejemplo.com."), "ejemplo.com.");
    }

    #[test]
    fn propagation_result_ratio() {
        let r = PropagationResult {
            confirmed: 3,
            total: 4,
        };
        assert!((r.ratio() - 0.75).abs() < 1e-9);
    }

    #[test]
    fn propagation_result_fully_propagated() {
        let full = PropagationResult {
            confirmed: 4,
            total: 4,
        };
        let partial = PropagationResult {
            confirmed: 3,
            total: 4,
        };
        assert!(full.is_fully_propagated());
        assert!(!partial.is_fully_propagated());
    }

    #[test]
    fn propagation_result_zero_total() {
        let r = PropagationResult {
            confirmed: 0,
            total: 0,
        };
        assert_eq!(r.ratio(), 0.0);
    }
}
