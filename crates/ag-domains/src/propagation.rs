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
    config::{NameServerConfig, NameServerConfigGroup, Protocol, ResolverConfig, ResolverOpts},
    TokioAsyncResolver,
};
use tracing::{debug, info};

use crate::error::AgDomainsError;

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
    resolvers: Vec<TokioAsyncResolver>,
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
                    lookup.iter().any(|txt| {
                        txt.iter().any(|bytes| {
                            std::str::from_utf8(bytes)
                                .map(|s| s == expected_value)
                                .unwrap_or(false)
                        })
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

fn build_resolver(addr: SocketAddr) -> TokioAsyncResolver {
    let ns = NameServerConfig::new(addr, Protocol::Udp);
    let mut group = NameServerConfigGroup::new();
    group.push(ns);

    let mut config = ResolverConfig::new();
    for server in group.iter() {
        config.add_name_server(server.clone());
    }

    let mut opts = ResolverOpts::default();
    // Do not use a cache so we measure real propagation.
    opts.cache_size = 0;

    TokioAsyncResolver::tokio(config, opts)
}

fn ensure_fqdn(name: &str) -> String {
    if name.ends_with('.') {
        name.to_owned()
    } else {
        format!("{name}.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
