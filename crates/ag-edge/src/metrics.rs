//! Edge data-plane metrics exported to `ag-observe` (blueprint section 16.1).
//!
//! Records route-resolution latency and the route cache hit/miss split (from
//! which a hit-ratio is derived on the dashboard). Calls are no-ops when no
//! recorder is registered, so the edge stays zero-cost by default.

use metrics::{counter, histogram};

/// Records the latency of a single hostname route resolution, labelled by the
/// resolution outcome (`custom`, `existing`, `fallback`, `unknown`).
pub fn record_route_resolution(outcome: &'static str, seconds: f64) {
    histogram!(
        "ag_edge_route_resolution_seconds",
        "outcome" => outcome,
    )
    .record(seconds);
}

/// Records a route-cache lookup as a hit or a miss.
///
/// A hit is a hostname that matched a custom binding (exact or wildcard); a miss
/// is a hostname with no custom binding (it falls through to the legacy resolver
/// or the global fallback). The hit-ratio is `hits / (hits + misses)`.
pub fn record_route_cache_lookup(hit: bool) {
    if hit {
        counter!("ag_edge_route_cache_hits_total").increment(1);
    } else {
        counter!("ag_edge_route_cache_misses_total").increment(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_do_not_panic() {
        // The calls must not panic even when no recorder is registered.
        record_route_resolution("custom", 0.000_01);
        record_route_resolution("fallback", 0.0);
        record_route_cache_lookup(true);
        record_route_cache_lookup(false);
    }
}
