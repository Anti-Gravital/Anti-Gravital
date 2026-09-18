# ag-cache

Two-level cache for Anti-Gravital: in-process L1 with tag-based invalidation (moka),
an optional native L2 RESP2 server compatible with any Redis client, and an
optional external Redis L2 for multi-instance deployments.

> Status: Phase 4 — L1 implemented (in-process, tag-based invalidation, moka).
> Native RESP2 L2 server implemented under feature `native-server` (RFC-0005).
> External Redis L2 implemented under feature `redis-l2`: read-through populates
> L1, writes and invalidations go through the shared store, and the in-process
> path stays the default (ADR-0009 rule 2).

## Minimal usage (L1 only)

```rust
use ag_cache::{AgCache, CacheConfig};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let cache = AgCache::new(CacheConfig::default()).await?;
cache.set("user:123", b"data".to_vec(), &[]).await;
let val: Option<Vec<u8>> = cache.get("user:123").await;
# Ok(())
# }
```

## Native RESP2 server (L2)

Enable the `native-server` Cargo feature to start an in-process TCP server
that speaks the RESP2 protocol. Any standard Redis client connects to it
without needing an actual Redis process.

```toml
[dependencies]
ag-cache = { version = "0.0.0", features = ["native-server"] }
```

```rust
use ag_cache::{AgCache, CacheConfig};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let mut cfg = CacheConfig::default();
cfg.native_server_enabled = true;
cfg.native_server_port = 6379; // or any free port

let cache = AgCache::new(cfg).await?;
// redis-cli -p 6379 ping  =>  PONG
# Ok(())
# }
```

### Supported commands

GET, SET (EX/PX/NX/XX), DEL, EXISTS, MGET, MSET, EXPIRE, TTL, KEYS (*),
PING, FLUSHDB, DBSIZE, COMMAND.

### Limitations

- **Single-node only.** No replication or clustering. Use a real Redis for
  multi-instance architectures where the cache must be shared.
- **No persistence.** Data is lost on process restart. This is cache, not a database.
- **No AUTH.** The server listens only on `127.0.0.1` (loopback) by default.
- **No TLS.** Use a real Redis for encrypted connections.
- **KEYS pattern** only supports `*` (all keys). Prefix patterns are not implemented.

## External Redis L2 (multi-instance)

Enable the `redis-l2` Cargo feature and set `redis_url` (or `REDIS_URL`) to back
the cache with an external Redis shared across instances. The L2 is the shared
source of truth: reads populate L1 on a miss (read-through), and writes, deletes
and tag invalidations go through the shared store, so an invalidation on one
instance propagates to the others on their next read-through. The L1 staleness
window is bounded by `l1_ttl_secs`. The in-process path stays the default
(ADR-0009 rule 2): without the feature, the cache is L1-only and a set
`REDIS_URL` only logs a warning.

```toml
[dependencies]
ag-cache = { version = "0.0.0", features = ["redis-l2"] }
```

```rust
use ag_cache::{AgCache, CacheConfig};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let mut cfg = CacheConfig::default();
cfg.redis_url = Some("redis://localhost:6379".to_owned());

let cache = AgCache::new(cfg).await?; // connects the external Redis L2
cache.set("user:123", b"data".to_vec(), &["users"]).await;
# Ok(())
# }
```

## Capabilities

### L1 (moka)

- High-concurrency in-memory cache with moka (TinyLFU eviction).
- Tag-based invalidation: `set_bytes_tagged(key, value, &["tag1", "tag2"])`.
- Configurable per-entry TTL or global via `default_ttl`.
- Automatic eviction when capacity is exceeded.

### L2 native (feature `native-server`)

- In-process RESP2 TCP server — no external Redis required.
- Backed by the same L1 moka store (no data duplication).
- Per-key TTL tracked via `DashMap<String, Option<Instant>>` alongside moka's TTL.
- Compatible with redis-cli, redis-rs, ioredis, and any RESP2 client.

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `CACHE_NATIVE_SERVER` | `false` | Enable native RESP2 server (`1` or `true`) |
| `CACHE_NATIVE_PORT` | `6379` | TCP port for the native RESP2 server |
| `REDIS_URL` | — | External Redis URL (L2, deferred — logs a warning if set) |

## References

- RFC-0005: `docs/rfc/RFC-0005-ag-cache-native-l2.md`
- Architecture: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` section 8.3
- Technical charter: `CLAUDE.md`
