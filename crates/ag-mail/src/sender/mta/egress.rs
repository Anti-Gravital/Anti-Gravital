//! Egress sources and pools for the native MTA (Phase 4.6-B).
//!
//! An *egress source* is a sending identity: an optional source IP to bind the
//! outgoing socket to, plus the EHLO/HELO name announced on that path. An
//! *egress pool* groups sources and hands them out by **smooth weighted
//! round-robin** (the nginx SWRR algorithm), so a freshly added IP can be given
//! a smaller weight while it warms up and still receive a fair, evenly-spread
//! share of traffic.
//!
//! Selection is pure and deterministic given the weights, so it is unit-tested
//! exhaustively. Binding to a concrete source IP happens at connection time in
//! the delivery path (`mail-send` `SmtpClientBuilder::local_ip`).

use std::net::IpAddr;
use std::sync::Mutex;

use crate::error::AgMailError;

/// A single egress identity used to send mail.
#[derive(Debug, Clone)]
pub struct EgressSource {
    /// Stable name for logs, shaping keys and metrics.
    pub name: String,
    /// EHLO/HELO hostname announced on this path. Should have valid forward and
    /// reverse (PTR) DNS for the bound IP.
    pub ehlo_domain: String,
    /// Source IP to bind the outgoing socket to. `None` lets the OS choose.
    pub source_ip: Option<IpAddr>,
    /// Relative weight for round-robin selection (>= 1). A warming IP gets a
    /// smaller weight than a trusted one.
    pub weight: u32,
}

impl EgressSource {
    /// Creates a source with weight 1 and no bound IP (OS default route).
    pub fn new(name: impl Into<String>, ehlo_domain: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ehlo_domain: ehlo_domain.into(),
            source_ip: None,
            weight: 1,
        }
    }

    /// Sets the source IP to bind to.
    pub fn with_ip(mut self, ip: IpAddr) -> Self {
        self.source_ip = Some(ip);
        self
    }

    /// Sets the round-robin weight (clamped to at least 1).
    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight.max(1);
        self
    }
}

/// A pool of egress sources selected by smooth weighted round-robin.
#[derive(Debug)]
pub struct EgressPool {
    /// Pool name (e.g. a tenant or reputation tier).
    pub name: String,
    sources: Vec<EgressSource>,
    /// SWRR running counters, one per source. Behind a `Mutex` so selection is
    /// `&self` and the pool can live in a shared, cloneable config via `Arc`.
    current: Mutex<Vec<i64>>,
}

impl EgressPool {
    /// Builds a pool. Returns an error if there are no sources (an empty pool
    /// could never hand out a path).
    pub fn new(name: impl Into<String>, sources: Vec<EgressSource>) -> Result<Self, AgMailError> {
        if sources.is_empty() {
            return Err(AgMailError::Config(
                "egress pool requires at least one source".to_owned(),
            ));
        }
        let len = sources.len();
        Ok(Self {
            name: name.into(),
            sources,
            current: Mutex::new(vec![0; len]),
        })
    }

    /// Number of sources in the pool.
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    /// Whether the pool has no sources. Always `false` for a pool built with
    /// [`EgressPool::new`]; present to satisfy the clippy `len_without_is_empty`
    /// lint and for completeness.
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    /// Selects the next egress source by smooth weighted round-robin.
    ///
    /// Each call advances the running counters: every source's counter grows by
    /// its weight, the highest is chosen, and the chosen one is reduced by the
    /// total weight. Over `sum(weights)` calls each source is chosen exactly
    /// `weight` times, spread out rather than in bursts.
    pub fn select(&self) -> EgressSource {
        let mut current = self.current.lock().expect("egress pool mutex poisoned");
        let total: i64 = self.sources.iter().map(|s| i64::from(s.weight)).sum();

        let mut best = 0usize;
        for (i, source) in self.sources.iter().enumerate() {
            current[i] += i64::from(source.weight);
            if current[i] > current[best] {
                best = i;
            }
        }
        current[best] -= total;
        self.sources[best].clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src(name: &str, weight: u32) -> EgressSource {
        EgressSource::new(name, format!("{name}.mail.example")).with_weight(weight)
    }

    #[test]
    fn empty_pool_rejected() {
        assert!(matches!(
            EgressPool::new("p", vec![]),
            Err(AgMailError::Config(_))
        ));
    }

    #[test]
    fn single_source_always_selected() {
        let pool = EgressPool::new("p", vec![src("a", 1)]).unwrap();
        for _ in 0..5 {
            assert_eq!(pool.select().name, "a");
        }
    }

    #[test]
    fn equal_weights_round_robin() {
        let pool = EgressPool::new("p", vec![src("a", 1), src("b", 1)]).unwrap();
        let picks: Vec<String> = (0..4).map(|_| pool.select().name).collect();
        // Even split, alternating.
        assert_eq!(picks, vec!["a", "b", "a", "b"]);
    }

    #[test]
    fn weighted_distribution_is_proportional_and_smooth() {
        // a:3, b:1 over 4 selections => a thrice, b once, not bursty.
        let pool = EgressPool::new("p", vec![src("a", 3), src("b", 1)]).unwrap();
        let picks: Vec<String> = (0..4).map(|_| pool.select().name).collect();
        let a = picks.iter().filter(|n| *n == "a").count();
        let b = picks.iter().filter(|n| *n == "b").count();
        assert_eq!(a, 3);
        assert_eq!(b, 1);
        // SWRR spreads b out (3rd of 4) rather than bursting it at an end.
        assert_eq!(picks, vec!["a", "a", "b", "a"]);
        // The cycle repeats deterministically.
        let next: Vec<String> = (0..4).map(|_| pool.select().name).collect();
        assert_eq!(picks, next);
    }

    #[test]
    fn warming_ip_gets_small_share() {
        // Trusted IP weight 10, warming IP weight 1 => ~9% to the warming IP.
        let pool = EgressPool::new("p", vec![src("trusted", 10), src("warming", 1)]).unwrap();
        let total = 110;
        let warming = (0..total)
            .map(|_| pool.select().name)
            .filter(|n| n == "warming")
            .count();
        assert_eq!(warming, total / 11); // exactly weight share over full cycles
    }

    #[test]
    fn weight_clamped_to_one() {
        assert_eq!(EgressSource::new("a", "a.example").with_weight(0).weight, 1);
    }
}
