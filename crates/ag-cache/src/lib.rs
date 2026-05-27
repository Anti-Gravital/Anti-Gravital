//! Multilevel cache for the Anti-Gravital ecosystem.
//!
//! Offers two transparent levels:
//! - **L1**: moka in memory (TinyLFU, no contended locks, always available).
//! - **L2**: Redis via fred (optional, for distributed cache across instances).
//!
//! # Status
//!
//! L1 fully operational. L2 (Redis/fred) remains pending as TECH-DEBT
//! for the second iteration of ag-cache in Phase 4 — the fred v10 API
//! requires feature adjustments and connection configuration in CI.
//!
//! # Minimal usage (L1 only)
//!
//! ```no_run
//! use ag_cache::{AgCache, CacheConfig};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let cache = AgCache::new(CacheConfig::default()).await?;
//! cache.set("user:123", b"datos".to_vec(), &[]).await;
//! let val: Option<Vec<u8>> = cache.get("user:123").await;
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod l1;
pub mod tags;

#[cfg(feature = "native-server")]
pub mod server;

pub use config::CacheConfig;

use l1::L1Cache;
use std::time::Duration;

/// Error from the cache subsystem.
#[derive(Debug)]
pub enum CacheError {
    /// Connection or communication error with Redis.
    Redis(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::Redis(msg) => write!(f, "error Redis: {msg}"),
        }
    }
}

impl std::error::Error for CacheError {}

/// Multilevel cache with L1 (moka) and optional L2 (Redis).
///
/// Build with [`AgCache::new`] passing a [`CacheConfig`]. If
/// `config.redis_url` is `None`, only L1 is active.
pub struct AgCache {
    l1: L1Cache,
    // TECH-DEBT:
    // reason: L2 Redis requires fred with a real connection; the fred v10 API
    //         does not expose the documented features (tokio-runtime, codec).
    //         Integrated in the second iteration of ag-cache in Phase 4.
    // impact: without L2, distributed invalidation across instances does not work.
    // expected removal: second iteration of ag-cache, Phase 4.
}

impl AgCache {
    /// Creates a new [`AgCache`] with the given configuration.
    ///
    /// If `config.redis_url` is `Some`, emits a warning via tracing but L2
    /// is not activated until it is implemented (see TECH-DEBT in the code).
    pub async fn new(config: CacheConfig) -> Result<Self, CacheError> {
        let ttl = Duration::from_secs(config.l1_ttl_secs);
        let l1 = L1Cache::new(config.l1_max_capacity, ttl);

        if config.redis_url.is_some() {
            tracing::warn!(
                "REDIS_URL configurada pero L2 Redis no esta activo en esta version (TECH-DEBT)"
            );
        }

        Ok(Self { l1 })
    }

    /// Gets raw bytes from the cache.
    ///
    /// Looks up L1 first. On a hit, logs `cache hit L1` via tracing.
    /// If there is no result, logs `cache miss`.
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let result = self.l1.get_bytes(key).await;
        if result.is_some() {
            tracing::debug!(key, "cache hit L1");
        } else {
            tracing::debug!(key, "cache miss");
        }
        result
    }

    /// Stores bytes in the cache with optional tags for invalidation.
    ///
    /// Writes to L1. If `tags` is empty, no tags are registered.
    pub async fn set(&self, key: &str, value: Vec<u8>, tags: &[&str]) {
        if tags.is_empty() {
            self.l1.set_bytes(key, value).await;
        } else {
            self.l1.set_bytes_tagged(key, value, tags).await;
        }
    }

    /// Invalidates all entries associated with the given tag in L1.
    pub async fn invalidate_tag(&self, tag: &str) {
        self.l1.invalidate_tag(tag).await;
    }

    /// Removes the entry with the given key.
    pub async fn delete(&self, key: &str) {
        self.l1.delete(key).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_without_redis_works() {
        let cache = AgCache::new(CacheConfig::default()).await.unwrap();
        cache.set("k", b"v".to_vec(), &[]).await;
        assert_eq!(cache.get("k").await, Some(b"v".to_vec()));
    }

    #[tokio::test]
    async fn invalidate_tag_clears_entries() {
        let cache = AgCache::new(CacheConfig::default()).await.unwrap();
        cache.set("u:1", b"a".to_vec(), &["users"]).await;
        cache.set("u:2", b"b".to_vec(), &["users"]).await;
        cache.invalidate_tag("users").await;
        assert!(cache.get("u:1").await.is_none());
        assert!(cache.get("u:2").await.is_none());
    }

    #[tokio::test]
    async fn delete_removes_single_entry() {
        let cache = AgCache::new(CacheConfig::default()).await.unwrap();
        cache.set("k", b"v".to_vec(), &[]).await;
        cache.delete("k").await;
        assert!(cache.get("k").await.is_none());
    }

    #[tokio::test]
    async fn get_returns_none_for_missing_key() {
        let cache = AgCache::new(CacheConfig::default()).await.unwrap();
        assert!(cache.get("nonexistent").await.is_none());
    }
}
