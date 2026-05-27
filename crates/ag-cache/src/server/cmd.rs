//! RESP2 command dispatch.
//!
//! Each public function handles one RESP2 command. All functions share
//! the same `Arc<L1Cache>` and `Arc<DashMap<String, Option<Instant>>>`.
//! The expiry map is the authoritative registry of live keys.

use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;

use tokio::io::AsyncWriteExt;

use crate::l1::L1Cache;
use crate::server::resp::{
    write_array, write_bulk, write_error, write_integer, write_ok, write_simple, Writer,
};

/// Authoritative registry of live keys and their optional expiry deadlines.
///
/// `None` means the key is persistent (no TTL). `Some(deadline)` means the key
/// expires at `deadline` and must be treated as absent once `deadline` has passed.
pub type ExpiryMap = DashMap<String, Option<Instant>>;

/// Returns `true` if `key` is live (present and not expired).
fn is_live(expiry: &ExpiryMap, key: &str) -> bool {
    match expiry.get(key) {
        None => false,
        Some(entry) => match *entry {
            None => true,
            Some(deadline) => deadline > Instant::now(),
        },
    }
}

/// Removes an expired key from both L1 and the expiry map.
async fn evict_if_expired(cache: &L1Cache, expiry: &ExpiryMap, key: &str) {
    if let Some(entry) = expiry.get(key) {
        if let Some(deadline) = *entry {
            if deadline <= Instant::now() {
                drop(entry);
                cache.delete(key).await;
                expiry.remove(key);
            }
        }
    }
}

// ── PING ──────────────────────────────────────────────────────────────────

/// Handles the RESP2 `PING` command. Returns `PONG` or echoes the argument.
pub async fn cmd_ping(args: &[Vec<u8>], w: &mut Writer) {
    if args.len() >= 2 {
        write_bulk(w, Some(&args[1])).await;
    } else {
        write_simple(w, "PONG").await;
    }
}

// ── GET ───────────────────────────────────────────────────────────────────

/// Handles the RESP2 `GET` command. Returns the value or nil if absent/expired.
pub async fn cmd_get(
    args: &[Vec<u8>],
    cache: &Arc<L1Cache>,
    expiry: &Arc<ExpiryMap>,
    w: &mut Writer,
) {
    if args.len() < 2 {
        write_error(w, "wrong number of arguments for GET").await;
        return;
    }
    let key = match std::str::from_utf8(&args[1]) {
        Ok(k) => k,
        Err(_) => {
            write_error(w, "key must be valid UTF-8").await;
            return;
        }
    };
    evict_if_expired(cache, expiry, key).await;
    if !is_live(expiry, key) {
        write_bulk(w, None).await;
        return;
    }
    let val = cache.get_bytes(key).await;
    write_bulk(w, val.as_deref()).await;
}

// ── SET ───────────────────────────────────────────────────────────────────

/// Handles the RESP2 `SET` command with optional `EX`, `PX`, `NX`, and `XX` flags.
pub async fn cmd_set(
    args: &[Vec<u8>],
    cache: &Arc<L1Cache>,
    expiry: &Arc<ExpiryMap>,
    w: &mut Writer,
) {
    // SET key value [EX seconds] [PX milliseconds] [NX] [XX]
    if args.len() < 3 {
        write_error(w, "wrong number of arguments for SET").await;
        return;
    }
    let key = match std::str::from_utf8(&args[1]) {
        Ok(k) => k.to_owned(),
        Err(_) => {
            write_error(w, "key must be valid UTF-8").await;
            return;
        }
    };
    let value = args[2].clone();

    let mut deadline: Option<Instant> = None;
    let mut nx = false;
    let mut xx = false;

    let mut i = 3usize;
    while i < args.len() {
        let opt = String::from_utf8_lossy(&args[i]).to_uppercase();
        match opt.as_str() {
            "EX" => {
                i += 1;
                let secs: u64 = match args
                    .get(i)
                    .and_then(|a| std::str::from_utf8(a).ok().and_then(|s| s.parse().ok()))
                {
                    Some(v) => v,
                    None => {
                        write_error(w, "value is not an integer or out of range").await;
                        return;
                    }
                };
                deadline = Some(Instant::now() + Duration::from_secs(secs));
            }
            "PX" => {
                i += 1;
                let ms: u64 = match args
                    .get(i)
                    .and_then(|a| std::str::from_utf8(a).ok().and_then(|s| s.parse().ok()))
                {
                    Some(v) => v,
                    None => {
                        write_error(w, "value is not an integer or out of range").await;
                        return;
                    }
                };
                deadline = Some(Instant::now() + Duration::from_millis(ms));
            }
            "NX" => nx = true,
            "XX" => xx = true,
            _ => {}
        }
        i += 1;
    }

    evict_if_expired(cache, expiry, &key).await;
    let exists = is_live(expiry, &key);

    if nx && exists {
        write_bulk(w, None).await; // NX: key already exists, do nothing
        return;
    }
    if xx && !exists {
        write_bulk(w, None).await; // XX: key does not exist, do nothing
        return;
    }

    cache.set_bytes(&key, value).await;
    expiry.insert(key, deadline);
    write_ok(w).await;
}

// ── DEL ───────────────────────────────────────────────────────────────────

/// Handles the RESP2 `DEL` command. Returns the count of actually deleted keys.
pub async fn cmd_del(
    args: &[Vec<u8>],
    cache: &Arc<L1Cache>,
    expiry: &Arc<ExpiryMap>,
    w: &mut Writer,
) {
    if args.len() < 2 {
        write_error(w, "wrong number of arguments for DEL").await;
        return;
    }
    let mut deleted = 0i64;
    for arg in &args[1..] {
        if let Ok(key) = std::str::from_utf8(arg) {
            if is_live(expiry, key) {
                cache.delete(key).await;
                expiry.remove(key);
                deleted += 1;
            }
        }
    }
    write_integer(w, deleted).await;
}

// ── EXISTS ────────────────────────────────────────────────────────────────

/// Handles the RESP2 `EXISTS` command. Returns the count of live keys among those given.
pub async fn cmd_exists(
    args: &[Vec<u8>],
    cache: &Arc<L1Cache>,
    expiry: &Arc<ExpiryMap>,
    w: &mut Writer,
) {
    if args.len() < 2 {
        write_error(w, "wrong number of arguments for EXISTS").await;
        return;
    }
    let mut count = 0i64;
    for arg in &args[1..] {
        if let Ok(key) = std::str::from_utf8(arg) {
            evict_if_expired(cache, expiry, key).await;
            if is_live(expiry, key) {
                count += 1;
            }
        }
    }
    write_integer(w, count).await;
}

// ── MGET ──────────────────────────────────────────────────────────────────

/// Handles the RESP2 `MGET` command. Returns a bulk-string array, nil for absent/expired keys.
pub async fn cmd_mget(
    args: &[Vec<u8>],
    cache: &Arc<L1Cache>,
    expiry: &Arc<ExpiryMap>,
    w: &mut Writer,
) {
    if args.len() < 2 {
        write_error(w, "wrong number of arguments for MGET").await;
        return;
    }
    let count = args.len() - 1;
    let _ = w.write_all(format!("*{count}\r\n").as_bytes()).await;
    for arg in &args[1..] {
        if let Ok(key) = std::str::from_utf8(arg) {
            evict_if_expired(cache, expiry, key).await;
            if is_live(expiry, key) {
                let val = cache.get_bytes(key).await;
                write_bulk(w, val.as_deref()).await;
            } else {
                write_bulk(w, None).await;
            }
        } else {
            write_bulk(w, None).await;
        }
    }
}

// ── MSET ──────────────────────────────────────────────────────────────────

/// Handles the RESP2 `MSET` command. Sets multiple key-value pairs atomically (persistent, no TTL).
pub async fn cmd_mset(
    args: &[Vec<u8>],
    cache: &Arc<L1Cache>,
    expiry: &Arc<ExpiryMap>,
    w: &mut Writer,
) {
    // MSET key value [key value ...]
    if args.len() < 3 || (args.len() - 1) % 2 != 0 {
        write_error(w, "wrong number of arguments for MSET").await;
        return;
    }
    let mut i = 1usize;
    while i + 1 < args.len() {
        if let Ok(key) = std::str::from_utf8(&args[i]) {
            let value = args[i + 1].clone();
            cache.set_bytes(key, value).await;
            expiry.insert(key.to_owned(), None);
        }
        i += 2;
    }
    write_ok(w).await;
}

// ── EXPIRE ────────────────────────────────────────────────────────────────

/// Handles the RESP2 `EXPIRE` command. Sets a TTL in seconds on an existing key.
/// Returns 1 if the TTL was applied, 0 if the key does not exist.
pub async fn cmd_expire(
    args: &[Vec<u8>],
    cache: &Arc<L1Cache>,
    expiry: &Arc<ExpiryMap>,
    w: &mut Writer,
) {
    if args.len() < 3 {
        write_error(w, "wrong number of arguments for EXPIRE").await;
        return;
    }
    let key = match std::str::from_utf8(&args[1]) {
        Ok(k) => k,
        Err(_) => {
            write_error(w, "key must be valid UTF-8").await;
            return;
        }
    };
    let secs: u64 = match std::str::from_utf8(&args[2])
        .ok()
        .and_then(|s| s.parse().ok())
    {
        Some(v) => v,
        None => {
            write_error(w, "value is not an integer or out of range").await;
            return;
        }
    };
    evict_if_expired(cache, expiry, key).await;
    if !is_live(expiry, key) {
        write_integer(w, 0).await; // key does not exist
        return;
    }
    expiry.insert(
        key.to_owned(),
        Some(Instant::now() + Duration::from_secs(secs)),
    );
    write_integer(w, 1).await;
}

// ── TTL ───────────────────────────────────────────────────────────────────

/// Handles the RESP2 `TTL` command. Returns remaining seconds, -1 for persistent, -2 for absent.
pub async fn cmd_ttl(
    args: &[Vec<u8>],
    cache: &Arc<L1Cache>,
    expiry: &Arc<ExpiryMap>,
    w: &mut Writer,
) {
    if args.len() < 2 {
        write_error(w, "wrong number of arguments for TTL").await;
        return;
    }
    let key = match std::str::from_utf8(&args[1]) {
        Ok(k) => k,
        Err(_) => {
            write_error(w, "key must be valid UTF-8").await;
            return;
        }
    };
    evict_if_expired(cache, expiry, key).await;
    match expiry.get(key) {
        None => write_integer(w, -2).await, // key does not exist
        Some(entry) => {
            let opt = *entry;
            drop(entry);
            match opt {
                None => write_integer(w, -1).await, // persistent key
                Some(deadline) => {
                    let now = Instant::now();
                    if deadline <= now {
                        cache.delete(key).await;
                        expiry.remove(key);
                        write_integer(w, -2).await;
                    } else {
                        write_integer(w, (deadline - now).as_secs() as i64).await;
                    }
                }
            }
        }
    }
}

// ── KEYS ──────────────────────────────────────────────────────────────────

/// Handles the RESP2 `KEYS` command. Only the `*` wildcard and exact-match patterns are supported.
pub async fn cmd_keys(args: &[Vec<u8>], expiry: &Arc<ExpiryMap>, w: &mut Writer) {
    // Only the `*` wildcard is supported (RFC-0005 section 4.2).
    if args.len() < 2 {
        write_error(w, "wrong number of arguments for KEYS").await;
        return;
    }
    let pattern = String::from_utf8_lossy(&args[1]);
    let now = Instant::now();
    let live_keys: Vec<Vec<u8>> = expiry
        .iter()
        .filter(|entry| match *entry.value() {
            None => true,
            Some(deadline) => deadline > now,
        })
        .filter(|entry| pattern == "*" || entry.key() == pattern.as_ref())
        .map(|entry| entry.key().as_bytes().to_vec())
        .collect();
    write_array(w, &live_keys).await;
}

// ── FLUSHDB ───────────────────────────────────────────────────────────────

/// Handles the RESP2 `FLUSHDB` command. Clears all keys from L1 and the expiry map.
pub async fn cmd_flushdb(cache: &Arc<L1Cache>, expiry: &Arc<ExpiryMap>, w: &mut Writer) {
    cache.flush().await;
    expiry.clear();
    write_ok(w).await;
}

// ── DBSIZE ────────────────────────────────────────────────────────────────

/// Handles the RESP2 `DBSIZE` command. Returns the count of live (non-expired) keys.
pub async fn cmd_dbsize(expiry: &Arc<ExpiryMap>, w: &mut Writer) {
    let now = Instant::now();
    let count = expiry
        .iter()
        .filter(|e| match *e.value() {
            None => true,
            Some(deadline) => deadline > now,
        })
        .count();
    write_integer(w, count as i64).await;
}

// ── COMMAND ───────────────────────────────────────────────────────────────

/// Handles the RESP2 `COMMAND` introspection request. Returns an empty array for client compatibility.
pub async fn cmd_command(w: &mut Writer) {
    // Return empty array: enough for clients that auto-detect capabilities.
    write_array(w, &[]).await;
}
