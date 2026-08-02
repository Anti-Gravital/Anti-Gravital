//! Live external-Redis L2 test for `ag-cache` (issue #144).
//!
//! Ignored by default: it needs a reachable Redis through `REDIS_URL`. Run with:
//!
//! ```sh
//! REDIS_URL=redis://localhost:6379 \
//!   cargo test -p ag-cache --features redis-l2 --test redis_l2 -- --ignored
//! ```
//!
//! Self-isolating: it uses time-unique keys and cleans them up.
#![cfg(feature = "redis-l2")]

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ag_cache::{AgCache, CacheConfig, L2Cache, RedisL2};

fn unique() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

#[tokio::test]
#[ignore = "requires REDIS_URL pointing at a live Redis"]
async fn redis_l2_roundtrip_and_distributed_invalidation() {
    let url = std::env::var("REDIS_URL").expect("REDIS_URL must point at a live Redis");
    let l2 = Arc::new(RedisL2::connect(&url).await.unwrap());

    let key = format!("ag:test:k:{}", unique());
    let tag = format!("ag:test:tag:{}", unique());

    // Direct adapter round-trip.
    l2.set(&key, b"v", Duration::from_secs(60), &[tag.as_str()])
        .await
        .unwrap();
    assert_eq!(l2.get(&key).await.unwrap(), Some(b"v".to_vec()));

    // A fresh AgCache (empty L1) reads it through the shared Redis.
    let cache = AgCache::new(CacheConfig::default())
        .await
        .unwrap()
        .with_l2(l2.clone());
    assert_eq!(cache.get(&key).await, Some(b"v".to_vec()));

    // Tag invalidation removes it from the shared store; a brand-new instance
    // no longer finds it (distributed invalidation through Redis).
    l2.invalidate_tag(&tag).await.unwrap();
    let fresh = AgCache::new(CacheConfig::default())
        .await
        .unwrap()
        .with_l2(l2.clone());
    assert_eq!(fresh.get(&key).await, None);

    // Cleanup (key already gone via tag invalidation; idempotent).
    l2.delete(&key).await.unwrap();
}
