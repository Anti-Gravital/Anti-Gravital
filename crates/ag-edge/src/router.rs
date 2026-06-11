//! Hostname-to-target routing with safe fallback.
//!
//! Resolution precedence (RFC-0011 section 14.2; blueprint section 14.2):
//!
//! 1. exact custom hostname binding
//! 2. wildcard custom hostname binding
//! 3. legacy default project route (caller-supplied)
//! 4. existing global fallback
//!
//! Unknown custom hostnames fail closed so they never resolve to another
//! tenant's binding.

use std::collections::HashMap;

use ag_domains::hostname::Hostname;
use serde::{Deserialize, Serialize};

/// A resolved binding from a hostname to an Anti-Gravital target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteBinding {
    /// Canonical ASCII hostname identity.
    pub hostname_ascii: String,
    /// Project identifier.
    pub project: String,
    /// Environment identifier.
    pub environment: String,
    /// Target reference (service id or project id).
    pub target_ref: String,
}

/// The outcome of resolving an incoming hostname.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteResolution {
    /// Matched a custom domain binding (exact or wildcard).
    Custom(RouteBinding),
    /// Matched an existing legacy default route (handle returned by the caller).
    Existing(String),
    /// No binding and no legacy route: hand off to the global fallback.
    ExistingFallback,
    /// The hostname is malformed or unknown; fail closed.
    Unknown,
}

/// An index of custom hostname bindings for fast resolution.
///
/// Exact bindings are keyed by canonical ASCII hostname. Wildcard bindings are
/// stored by their base (the registered domain or parent), matching exactly one
/// label below the base, consistent with TLS wildcard semantics.
#[derive(Debug, Default)]
pub struct BindingCache {
    exact: HashMap<String, RouteBinding>,
    /// (base, binding) where base is the portion after `*.`.
    wildcard: Vec<(String, RouteBinding)>,
}

impl BindingCache {
    /// Creates an empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a binding. Wildcard hostnames (`*.base`) populate the wildcard
    /// table; everything else populates the exact table.
    pub fn insert(&mut self, binding: RouteBinding) {
        if let Some(base) = binding.hostname_ascii.strip_prefix("*.") {
            let base = base.to_owned();
            self.wildcard.push((base, binding));
        } else {
            self.exact.insert(binding.hostname_ascii.clone(), binding);
        }
    }

    /// Number of bindings (exact + wildcard).
    pub fn len(&self) -> usize {
        self.exact.len() + self.wildcard.len()
    }

    /// Whether the cache holds no bindings.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn match_exact(&self, ascii: &str) -> Option<&RouteBinding> {
        self.exact.get(ascii)
    }

    fn match_wildcard(&self, ascii: &str) -> Option<&RouteBinding> {
        for (base, binding) in &self.wildcard {
            if let Some(prefix) = ascii.strip_suffix(base) {
                // Require exactly one label before the base: `<label>.<base>`.
                let prefix = prefix.strip_suffix('.').unwrap_or(prefix);
                if !prefix.is_empty() && !prefix.contains('.') {
                    return Some(binding);
                }
            }
        }
        None
    }
}

/// Resolves an incoming hostname against the binding cache, then a legacy
/// default resolver, then the global fallback.
///
/// `legacy` returns an opaque route handle for hostnames that the pre-existing
/// (non-custom-domain) routing already serves; it preserves backward
/// compatibility. Returning `None` from `legacy` for an unknown hostname yields
/// [`RouteResolution::ExistingFallback`] (legacy global behavior), never
/// another tenant's binding.
pub fn resolve_hostname(
    cache: &BindingCache,
    hostname: &str,
    legacy: impl Fn(&str) -> Option<String>,
) -> RouteResolution {
    let started = std::time::Instant::now();
    let resolution = resolve_hostname_inner(cache, hostname, legacy);

    // Edge observability (blueprint section 16.1): route-resolution latency and
    // the cache hit/miss split. A custom binding is a cache hit; anything else
    // (legacy, fallback) is a miss. `Unknown` is a malformed host, not a lookup.
    let outcome = match &resolution {
        RouteResolution::Custom(_) => "custom",
        RouteResolution::Existing(_) => "existing",
        RouteResolution::ExistingFallback => "fallback",
        RouteResolution::Unknown => "unknown",
    };
    crate::metrics::record_route_resolution(outcome, started.elapsed().as_secs_f64());
    if !matches!(resolution, RouteResolution::Unknown) {
        crate::metrics::record_route_cache_lookup(matches!(resolution, RouteResolution::Custom(_)));
    }

    resolution
}

fn resolve_hostname_inner(
    cache: &BindingCache,
    hostname: &str,
    legacy: impl Fn(&str) -> Option<String>,
) -> RouteResolution {
    let normalized = match Hostname::parse(hostname) {
        Ok(h) => h.ascii().to_owned(),
        Err(_) => return RouteResolution::Unknown,
    };

    if let Some(binding) = cache.match_exact(&normalized) {
        return RouteResolution::Custom(binding.clone());
    }
    if let Some(binding) = cache.match_wildcard(&normalized) {
        return RouteResolution::Custom(binding.clone());
    }
    if let Some(handle) = legacy(&normalized) {
        return RouteResolution::Existing(handle);
    }
    RouteResolution::ExistingFallback
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding(host: &str, target: &str) -> RouteBinding {
        RouteBinding {
            hostname_ascii: host.to_owned(),
            project: "site".to_owned(),
            environment: "production".to_owned(),
            target_ref: target.to_owned(),
        }
    }

    fn no_legacy(_: &str) -> Option<String> {
        None
    }

    #[test]
    fn exact_binding_resolves() {
        let mut cache = BindingCache::new();
        cache.insert(binding("api.example.com", "svc_api"));
        let r = resolve_hostname(&cache, "api.example.com", no_legacy);
        assert!(matches!(r, RouteResolution::Custom(b) if b.target_ref == "svc_api"));
    }

    #[test]
    fn host_is_normalized_before_match() {
        let mut cache = BindingCache::new();
        cache.insert(binding("api.example.com", "svc_api"));
        let r = resolve_hostname(&cache, "API.Example.com", no_legacy);
        assert!(matches!(r, RouteResolution::Custom(_)));
    }

    #[test]
    fn wildcard_matches_single_label() {
        let mut cache = BindingCache::new();
        cache.insert(binding("*.example.com", "svc_wild"));
        let r = resolve_hostname(&cache, "anything.example.com", no_legacy);
        assert!(matches!(r, RouteResolution::Custom(b) if b.target_ref == "svc_wild"));
    }

    #[test]
    fn wildcard_does_not_match_two_labels() {
        let mut cache = BindingCache::new();
        cache.insert(binding("*.example.com", "svc_wild"));
        let r = resolve_hostname(&cache, "a.b.example.com", no_legacy);
        assert!(matches!(r, RouteResolution::ExistingFallback));
    }

    #[test]
    fn exact_takes_precedence_over_wildcard() {
        let mut cache = BindingCache::new();
        cache.insert(binding("*.example.com", "svc_wild"));
        cache.insert(binding("api.example.com", "svc_exact"));
        let r = resolve_hostname(&cache, "api.example.com", no_legacy);
        assert!(matches!(r, RouteResolution::Custom(b) if b.target_ref == "svc_exact"));
    }

    #[test]
    fn legacy_route_used_when_no_binding() {
        let cache = BindingCache::new();
        let r = resolve_hostname(&cache, "app.gravital.dev", |h| Some(format!("legacy:{h}")));
        assert!(matches!(r, RouteResolution::Existing(h) if h == "legacy:app.gravital.dev"));
    }

    #[test]
    fn unknown_custom_hostname_fails_closed() {
        let cache = BindingCache::new();
        // No binding, no legacy match: must NOT resolve to a tenant.
        let r = resolve_hostname(&cache, "evil.example.com", no_legacy);
        assert_eq!(r, RouteResolution::ExistingFallback);
    }

    #[test]
    fn malformed_hostname_is_unknown() {
        let cache = BindingCache::new();
        let r = resolve_hostname(&cache, "https://x/y", no_legacy);
        assert_eq!(r, RouteResolution::Unknown);
    }
}
