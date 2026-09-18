//! Multilevel cache for the Anti-Gravital ecosystem.
//!
//! Offers transparent levels:
//! - **L1**: moka in memory (TinyLFU, no contended locks, always available).
//! - **L2 native**: optional in-process RESP2 server (feature `native-server`),
//!   compatible with any Redis client. Backed by the same L1 store.
//! - **L2 distributed**: optional external Redis backing (feature `redis-l2`)
//!   for multi-instance deployments. Read-through populates L1; writes and
//!   invalidations go through the shared store (see [`l2`]).
//!
//! # Status
//!
//! L1 fully operational. Native RESP2 server (RFC-0005) under the
//! `native-server` feature. External Redis L2 under the `redis-l2` feature: the
//! in-process path stays the default, the external backing is opt-in (ADR-0009
//! rule 2).
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
//!
//! # Native RESP2 server
//!
//! ```no_run
//! use ag_cache::{AgCache, CacheConfig};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let mut cfg = CacheConfig::default();
//! cfg.native_server_enabled = true;
//! cfg.native_server_port = 6379;
//! let cache = AgCache::new(cfg).await?;
//! // redis-cli -p 6379 ping  =>  PONG
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod l1;
pub mod l2;
pub mod tags;

#[cfg(feature = "native-server")]
pub mod server;

pub use config::CacheConfig;
pub use l2::{InMemoryL2, L2Cache};

#[cfg(feature = "redis-l2")]
pub use l2::RedisL2;

use l1::L1Cache;
use std::sync::Arc;
use std::time::Duration;

/// Error from the cache subsystem.
#[derive(Debug)]
#[non_exhaustive]
pub enum CacheError {
    /// Connection or communication error with Redis.
    Redis(String),
    /// I/O error starting the native server.
    #[cfg(feature = "native-server")]
    NativeServer(std::io::Error),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::Redis(msg) => write!(f, "Redis error: {msg}"),
            #[cfg(feature = "native-server")]
            CacheError::NativeServer(e) => write!(f, "native server error: {e}"),
        }
    }
}

impl std::error::Error for CacheError {}

/// Multilevel cache with L1 (moka), an optional native RESP2 server and an
/// optional external Redis L2 (feature `redis-l2`).
pub struct AgCache {
    l1: Arc<L1Cache>,
    l2: Option<Arc<dyn L2Cache>>,
    ttl: Duration,
}

impl AgCache {
    /// Creates a new [`AgCache`] with the given configuration.
    ///
    /// If `native_server_enabled` is `true` and feature `native-server` is active,
    /// spawns the RESP2 server in a background task. If `redis_url` is set and
    /// feature `redis-l2` is active, connects the external Redis L2; otherwise
    /// the cache is L1-only (the native default, ADR-0009 rule 2).
    pub async fn new(config: CacheConfig) -> Result<Self, CacheError> {
        let ttl = Duration::from_secs(config.l1_ttl_secs);
        let l1 = Arc::new(L1Cache::new(config.l1_max_capacity, ttl));
        let l2 = connect_l2(&config).await?;

        #[cfg(feature = "native-server")]
        if config.native_server_enabled {
            let srv = server::NativeCacheServer::bind(config.native_server_port, Arc::clone(&l1))
                .await
                .map_err(CacheError::NativeServer)?;
            tokio::spawn(srv.serve());
        }

        Ok(Self { l1, l2, ttl })
    }

    /// Injects a custom [`L2Cache`] backing, overriding any configured one.
    ///
    /// Used to plug a non-default distributed backing (and by tests that share
    /// one [`InMemoryL2`] across instances).
    #[must_use]
    pub fn with_l2(mut self, l2: Arc<dyn L2Cache>) -> Self {
        self.l2 = Some(l2);
        self
    }

    /// Gets raw bytes from the cache.
    ///
    /// On an L1 miss, reads through the L2 (if configured) and, on an L2 hit,
    /// populates L1 before returning. An L2 backend error is logged and treated
    /// as a miss, so a transient L2 outage degrades to L1-only rather than
    /// failing the read.
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        if let Some(value) = self.l1.get_bytes(key).await {
            tracing::debug!(key, "cache hit L1");
            return Some(value);
        }
        if let Some(l2) = &self.l2 {
            match l2.get(key).await {
                Ok(Some(value)) => {
                    tracing::debug!(key, "cache hit L2");
                    self.l1.set_bytes(key, value.clone()).await;
                    return Some(value);
                }
                Ok(None) => {}
                Err(e) => tracing::warn!(key, error = %e, "L2 get failed; serving as L1 miss"),
            }
        }
        tracing::debug!(key, "cache miss");
        None
    }

    /// Stores bytes in the cache with optional tags for invalidation, writing
    /// through to the L2 when configured.
    pub async fn set(&self, key: &str, value: Vec<u8>, tags: &[&str]) {
        if tags.is_empty() {
            self.l1.set_bytes(key, value.clone()).await;
        } else {
            self.l1.set_bytes_tagged(key, value.clone(), tags).await;
        }
        if let Some(l2) = &self.l2 {
            if let Err(e) = l2.set(key, &value, self.ttl, tags).await {
                tracing::warn!(key, error = %e, "L2 set failed; value cached in L1 only");
            }
        }
    }

    /// Invalidates all entries associated with the given tag, in L1 and the L2.
    pub async fn invalidate_tag(&self, tag: &str) {
        self.l1.invalidate_tag(tag).await;
        if let Some(l2) = &self.l2 {
            if let Err(e) = l2.invalidate_tag(tag).await {
                tracing::warn!(tag, error = %e, "L2 tag invalidation failed");
            }
        }
    }

    /// Removes the entry with the given key, from L1 and the L2.
    pub async fn delete(&self, key: &str) {
        self.l1.delete(key).await;
        if let Some(l2) = &self.l2 {
            if let Err(e) = l2.delete(key).await {
                tracing::warn!(key, error = %e, "L2 delete failed");
            }
        }
    }
}

/// Connects the external Redis L2 when the `redis-l2` feature is on and
/// `redis_url` is set; otherwise returns `None` (the native default).
#[cfg(feature = "redis-l2")]
async fn connect_l2(config: &CacheConfig) -> Result<Option<Arc<dyn L2Cache>>, CacheError> {
    match &config.redis_url {
        Some(url) => {
            let backing = l2::RedisL2::connect(url).await?;
            Ok(Some(Arc::new(backing) as Arc<dyn L2Cache>))
        }
        None => Ok(None),
    }
}

#[cfg(not(feature = "redis-l2"))]
async fn connect_l2(config: &CacheConfig) -> Result<Option<Arc<dyn L2Cache>>, CacheError> {
    if config.redis_url.is_some() {
        tracing::warn!(
            "REDIS_URL is set but the `redis-l2` feature is not enabled; running L1-only"
        );
    }
    Ok(None)
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

    // ---- L2 backing (backend-agnostic, in-process via a shared InMemoryL2) ---

    #[tokio::test]
    async fn l2_read_through_populates_l1() {
        let shared = Arc::new(InMemoryL2::new());
        shared
            .set("k", b"v", Duration::from_secs(60), &[])
            .await
            .unwrap();
        let cache = AgCache::new(CacheConfig::default())
            .await
            .unwrap()
            .with_l2(shared.clone());
        // L1 miss -> L2 hit -> value returned (and L1 populated).
        assert_eq!(cache.get("k").await, Some(b"v".to_vec()));
        // Subsequent read now served by L1 even if L2 is emptied.
        shared.delete("k").await.unwrap();
        assert_eq!(cache.get("k").await, Some(b"v".to_vec()));
    }

    #[tokio::test]
    async fn set_writes_through_to_l2() {
        let shared = Arc::new(InMemoryL2::new());
        let cache = AgCache::new(CacheConfig::default())
            .await
            .unwrap()
            .with_l2(shared.clone());
        cache.set("k", b"v".to_vec(), &[]).await;
        assert_eq!(shared.get("k").await.unwrap(), Some(b"v".to_vec()));
    }

    #[tokio::test]
    async fn distributed_invalidation_via_shared_l2() {
        // Two instances sharing one L2 stand in for two instances sharing one
        // Redis. Invalidation on A propagates to a fresh instance through L2.
        let shared = Arc::new(InMemoryL2::new());
        let a = AgCache::new(CacheConfig::default())
            .await
            .unwrap()
            .with_l2(shared.clone());
        let b = AgCache::new(CacheConfig::default())
            .await
            .unwrap()
            .with_l2(shared.clone());

        a.set("u:1", b"x".to_vec(), &["users"]).await;
        // B sees A's write through the shared L2 (B's L1 was empty).
        assert_eq!(b.get("u:1").await, Some(b"x".to_vec()));

        // A invalidates the tag, removing it from the shared L2.
        a.invalidate_tag("users").await;

        // A brand-new instance (empty L1) no longer finds it: the invalidation
        // propagated across instances through the shared L2.
        let c = AgCache::new(CacheConfig::default())
            .await
            .unwrap()
            .with_l2(shared.clone());
        assert_eq!(c.get("u:1").await, None);
    }

    #[tokio::test]
    async fn delete_propagates_to_l2() {
        let shared = Arc::new(InMemoryL2::new());
        let cache = AgCache::new(CacheConfig::default())
            .await
            .unwrap()
            .with_l2(shared.clone());
        cache.set("k", b"v".to_vec(), &[]).await;
        cache.delete("k").await;
        assert_eq!(shared.get("k").await.unwrap(), None);
    }
}
