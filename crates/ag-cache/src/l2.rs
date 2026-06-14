//! L2 distributed cache backing.
//!
//! RFC-0005 chose an in-process RESP2 server as the *default* L2 to avoid an
//! operational Redis dependency. This module adds the complementary option that
//! RFC-0005 left for multi-instance deployments: an external Redis L2 as an
//! opt-in adapter behind the `redis-l2` feature, with the in-process path kept
//! as the native default (ADR-0009 rule 2).
//!
//! # Consistency model
//!
//! The L2 is the shared source of truth across instances. [`AgCache`] reads
//! through it on an L1 miss (populating L1) and writes through it on
//! set/delete/invalidate. A cross-instance invalidation therefore propagates by
//! removing the entry from the shared L2; other instances observe it on their
//! next read-through. The L1 staleness window is bounded by the L1 TTL
//! (`l1_ttl_secs`): an instance that already cached a value in L1 keeps serving
//! it until that TTL elapses. Active L1 invalidation broadcast is intentionally
//! out of scope here (RFC-0005 keeps pub/sub in `ag-realtime`).
//!
//! [`AgCache`]: crate::AgCache

use std::time::Duration;

use async_trait::async_trait;

use crate::CacheError;

/// A distributed L2 cache backing shared across instances.
#[async_trait]
pub trait L2Cache: Send + Sync {
    /// Fetches the raw bytes for `key`, if present.
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError>;

    /// Stores `value` under `key` with the given `ttl`, registering `tags` for
    /// group invalidation.
    async fn set(
        &self,
        key: &str,
        value: &[u8],
        ttl: Duration,
        tags: &[&str],
    ) -> Result<(), CacheError>;

    /// Removes the entry with `key`.
    async fn delete(&self, key: &str) -> Result<(), CacheError>;

    /// Removes every entry registered under `tag`.
    async fn invalidate_tag(&self, tag: &str) -> Result<(), CacheError>;
}

/// Process-local reference L2.
///
/// It is the reference implementation and powers the distributed-invalidation
/// tests: two [`AgCache`](crate::AgCache) instances sharing one `InMemoryL2`
/// behave like two instances sharing one Redis. It does not survive the process
/// and is not distributed, so production multi-instance setups use
/// [`RedisL2`] (feature `redis-l2`).
#[derive(Debug, Default)]
pub struct InMemoryL2 {
    store: std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
    tags: std::sync::Mutex<std::collections::HashMap<String, std::collections::HashSet<String>>>,
}

impl InMemoryL2 {
    /// Creates an empty L2.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl L2Cache for InMemoryL2 {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        Ok(self
            .store
            .lock()
            .expect("l2 lock poisoned")
            .get(key)
            .cloned())
    }

    async fn set(
        &self,
        key: &str,
        value: &[u8],
        _ttl: Duration,
        tags: &[&str],
    ) -> Result<(), CacheError> {
        self.store
            .lock()
            .expect("l2 lock poisoned")
            .insert(key.to_owned(), value.to_vec());
        if !tags.is_empty() {
            let mut idx = self.tags.lock().expect("l2 lock poisoned");
            for tag in tags {
                idx.entry((*tag).to_owned())
                    .or_default()
                    .insert(key.to_owned());
            }
        }
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        self.store.lock().expect("l2 lock poisoned").remove(key);
        Ok(())
    }

    async fn invalidate_tag(&self, tag: &str) -> Result<(), CacheError> {
        let keys = self
            .tags
            .lock()
            .expect("l2 lock poisoned")
            .remove(tag)
            .unwrap_or_default();
        let mut store = self.store.lock().expect("l2 lock poisoned");
        for key in keys {
            store.remove(&key);
        }
        Ok(())
    }
}

#[cfg(feature = "redis-l2")]
pub use redis_backend::RedisL2;

#[cfg(feature = "redis-l2")]
mod redis_backend {
    use super::{CacheError, L2Cache};
    use async_trait::async_trait;
    use redis::aio::ConnectionManager;
    use redis::AsyncCommands;
    use std::time::Duration;

    fn redis_err(e: redis::RedisError) -> CacheError {
        CacheError::Redis(e.to_string())
    }

    /// Tag set key prefix in Redis (`ag:tag:{tag}` -> set of member keys).
    fn tag_key(tag: &str) -> String {
        format!("ag:tag:{tag}")
    }

    /// External Redis L2 adapter (feature `redis-l2`).
    ///
    /// Uses a multiplexed [`ConnectionManager`] that reconnects automatically.
    /// Tags are tracked as Redis sets so a tag invalidation can delete every
    /// member key in one round trip.
    pub struct RedisL2 {
        conn: ConnectionManager,
    }

    impl RedisL2 {
        /// Connects to the Redis instance at `url` (e.g. `redis://host:6379`).
        pub async fn connect(url: &str) -> Result<Self, CacheError> {
            let client = redis::Client::open(url).map_err(redis_err)?;
            let conn = ConnectionManager::new(client).await.map_err(redis_err)?;
            Ok(Self { conn })
        }
    }

    #[async_trait]
    impl L2Cache for RedisL2 {
        async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
            let mut conn = self.conn.clone();
            let value: Option<Vec<u8>> = conn.get(key).await.map_err(redis_err)?;
            Ok(value)
        }

        async fn set(
            &self,
            key: &str,
            value: &[u8],
            ttl: Duration,
            tags: &[&str],
        ) -> Result<(), CacheError> {
            let mut conn = self.conn.clone();
            let secs = ttl.as_secs().max(1);
            let _: () = conn.set_ex(key, value, secs).await.map_err(redis_err)?;
            for tag in tags {
                let _: i64 = conn.sadd(tag_key(tag), key).await.map_err(redis_err)?;
            }
            Ok(())
        }

        async fn delete(&self, key: &str) -> Result<(), CacheError> {
            let mut conn = self.conn.clone();
            let _: i64 = conn.del(key).await.map_err(redis_err)?;
            Ok(())
        }

        async fn invalidate_tag(&self, tag: &str) -> Result<(), CacheError> {
            let mut conn = self.conn.clone();
            let set_key = tag_key(tag);
            let members: Vec<String> = conn.smembers(&set_key).await.map_err(redis_err)?;
            if !members.is_empty() {
                let _: i64 = conn.del(members).await.map_err(redis_err)?;
            }
            let _: i64 = conn.del(set_key).await.map_err(redis_err)?;
            Ok(())
        }
    }
}
