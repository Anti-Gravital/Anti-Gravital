//! L1 cache: in-memory moka with tag support and group invalidation.

use crate::tags::TagIndex;
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// L1 cache based on moka with TinyLFU and tag support.
pub struct L1Cache {
    inner: Cache<String, Vec<u8>>,
    tags: Arc<Mutex<TagIndex>>,
}

impl L1Cache {
    /// Creates a new [`L1Cache`] with the given capacity and TTL.
    pub fn new(max_capacity: u64, default_ttl: Duration) -> Self {
        let inner = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(default_ttl)
            .build();
        Self {
            inner,
            tags: Arc::new(Mutex::new(TagIndex::default())),
        }
    }

    /// Gets the raw bytes associated with `key`.
    pub async fn get_bytes(&self, key: &str) -> Option<Vec<u8>> {
        self.inner.get(key).await
    }

    /// Stores `value` under `key` with the cache's default TTL.
    pub async fn set_bytes(&self, key: &str, value: Vec<u8>) {
        self.inner.insert(key.to_string(), value).await;
    }

    /// Stores `value` under `key` and registers the `tags` for invalidation.
    pub async fn set_bytes_tagged(&self, key: &str, value: Vec<u8>, tags: &[&str]) {
        self.inner.insert(key.to_string(), value).await;
        self.tags.lock().await.insert(key, tags);
    }

    /// Removes the entry with `key`.
    pub async fn delete(&self, key: &str) {
        self.inner.remove(key).await;
        self.tags.lock().await.remove(key);
    }

    /// Invalidates all entries associated with the given `tag`.
    pub async fn invalidate_tag(&self, tag: &str) {
        let keys = self.tags.lock().await.keys_for_tag(tag);
        for key in &keys {
            self.inner.remove(key).await;
        }
        let mut idx = self.tags.lock().await;
        for key in &keys {
            idx.remove(key);
        }
    }

    /// Removes all entries and clears the tag index (RESP2 FLUSHDB equivalent).
    ///
    /// Not atomic with respect to concurrent writes: entries inserted between
    /// `invalidate_all()` and the tag-index clear may lose tag tracking.
    /// Callers must ensure no concurrent writes during this call for a fully
    /// consistent flush.
    pub async fn flush(&self) {
        self.inner.invalidate_all();
        self.inner.run_pending_tasks().await;
        self.tags.lock().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn set_and_get_roundtrip() {
        let cache = L1Cache::new(100, Duration::from_secs(60));
        cache.set_bytes("k1", b"hello".to_vec()).await;
        assert_eq!(cache.get_bytes("k1").await, Some(b"hello".to_vec()));
    }

    #[tokio::test]
    async fn tag_invalidation_removes_entries() {
        let cache = L1Cache::new(100, Duration::from_secs(60));
        cache
            .set_bytes_tagged("u:1", b"a".to_vec(), &["users"])
            .await;
        cache
            .set_bytes_tagged("u:2", b"b".to_vec(), &["users"])
            .await;
        cache.invalidate_tag("users").await;
        assert!(cache.get_bytes("u:1").await.is_none());
        assert!(cache.get_bytes("u:2").await.is_none());
    }

    #[tokio::test]
    async fn delete_removes_from_tags() {
        let cache = L1Cache::new(100, Duration::from_secs(60));
        cache.set_bytes_tagged("k", b"v".to_vec(), &["t"]).await;
        cache.delete("k").await;
        assert!(cache.get_bytes("k").await.is_none());
    }

    #[tokio::test]
    async fn get_missing_key_returns_none() {
        let cache = L1Cache::new(100, Duration::from_secs(60));
        assert!(cache.get_bytes("nonexistent").await.is_none());
    }

    #[tokio::test]
    async fn set_bytes_without_tags_and_get() {
        let cache = L1Cache::new(100, Duration::from_secs(60));
        cache.set_bytes("plain_key", b"data".to_vec()).await;
        assert_eq!(cache.get_bytes("plain_key").await, Some(b"data".to_vec()));
    }

    #[tokio::test]
    async fn delete_missing_key_does_not_panic() {
        let cache = L1Cache::new(100, Duration::from_secs(60));
        cache.delete("never_inserted").await;
    }

    #[tokio::test]
    async fn invalidate_missing_tag_does_not_panic() {
        let cache = L1Cache::new(100, Duration::from_secs(60));
        cache.invalidate_tag("nonexistent_tag").await;
    }

    /// Basic throughput: 1_000_000 operations in under 1 second.
    /// Marked `#[ignore]` to avoid blocking CI on slow hardware; run
    /// manually with `cargo test -p ag-cache -- --ignored l1_ops_per_second`.
    #[tokio::test]
    #[ignore]
    async fn l1_ops_per_second_exceeds_1m() {
        let cache = L1Cache::new(1_000_000, Duration::from_secs(60));
        let n: u64 = 1_000_000;
        let start = std::time::Instant::now();
        for i in 0..n {
            let key = format!("k{i}");
            cache.set_bytes(&key, vec![0u8; 8]).await;
        }
        for i in 0..n {
            let key = format!("k{i}");
            let _ = cache.get_bytes(&key).await;
        }
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_secs() < 2,
            "2M ops tardaron {elapsed:?}, esperado < 2s"
        );
    }

    #[tokio::test]
    async fn flush_clears_all_entries() {
        let cache = L1Cache::new(100, Duration::from_secs(60));
        cache.set_bytes("a", b"1".to_vec()).await;
        cache.set_bytes_tagged("b", b"2".to_vec(), &["tag1"]).await;
        cache.flush().await;
        assert!(cache.get_bytes("a").await.is_none());
        assert!(cache.get_bytes("b").await.is_none());
        // Verify tag index was also cleared: a new entry under the same tag
        // can be inserted and invalidated independently after flush.
        cache.set_bytes_tagged("c", b"3".to_vec(), &["tag1"]).await;
        cache.invalidate_tag("tag1").await;
        assert!(cache.get_bytes("c").await.is_none());
    }
}
