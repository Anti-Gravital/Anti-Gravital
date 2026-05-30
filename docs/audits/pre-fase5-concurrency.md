# Pre-Phase 5 Concurrency, Async & Backpressure (Stage 7)

> Stage 7 deliverable of the master audit plan. Avoids classic async-framework
> failures: unbounded tasks, misused locks, blocking the runtime, missing limits
> and timeouts, background tasks without graceful shutdown.

- **Date:** 2026-05-29
- **Branch:** `audit-pre-fase5`
- **Method:** `rg` over `crates/` for `tokio::spawn`, `Arc<Mutex>`/`RwLock`,
  `std::fs`, `std::thread`, `block_in_place`/`spawn_blocking`, unbounded channels;
  plus targeted reads of the hot paths.

## Inventory of `tokio::spawn`

| Location | Nature | Verdict |
| --- | --- | --- |
| `ag-cache/server/mod.rs:63` | one task per accepted TCP connection | standard; see S7-2 |
| `ag-core/shield/mod.rs:224` | per-connection hyper serving task | standard |
| `ag-realtime/lib.rs:129`, `ws.rs:29`, `external.rs:96` | per-connection / per-subscription | standard |
| `ag-storage/lib.rs:82` | starts the embedded server (one-time) | ok |
| `ag-cache/lib.rs:107` | starts the native server (one-time) | ok |
| `ag-domains/acme/renewal.rs:120` | renewal loop (one-time, sleeps to window) | ok |
| `ag-mail/queue/mod.rs:84` | the queue worker (one-time) | ok |
| `ag-mail/queue/mod.rs:144` | per-retry delayed re-enqueue | bounded; see S7-3 |

No per-request unbounded spawning. `std::thread::spawn` appears only in an
`ag-auth/refresh.rs` `#[test]` (thread-safety test), not production.

## Channels & backpressure

**No unbounded channels anywhere.** The mail queue (`InMemoryQueue`) uses
`mpsc::channel(capacity)`; `enqueue` is `tx.send().await`, which blocks the caller
when full — real backpressure. The retry channel is bounded (`mpsc::channel(64)`).

## Blocking I/O on the async runtime

- `ag-storage`: `get`/`put`/`delete`/`exists` use `tokio::fs` (async). `list`
  runs the synchronous `std::fs::read_dir` inside `tokio::task::spawn_blocking`
  (line 92) — correct. The only bare `std::fs` is `create_dir_all` in the
  constructor (one-time). **No runtime blocking.**
- `ag-core`: bare `std::fs` is config/key/TLS loading at startup, plus a test
  helper — not on a request path.
- `ag-realtime/persistence.rs`: see S7-1.

## Locks

`Arc<Mutex>`/`RwLock` usages are appropriate: `ag-cache` `Mutex<TagIndex>` (tag
index, not the value hot path — values are in moka), `ag-auth`
`RwLock<HashSet>` (refresh blacklist, read-mostly), `ag-lsp` `Mutex<HashMap>`
(document store), and `Mutex<()>` env-guards in tests. No lock held across a hot
request path or `.await` in a way that would serialize throughput.

## External connection timeouts

Plan rule: every external connection needs a timeout + controlled retry.

### Finding S7-4 — reqwest clients without timeout (Medium, fixed)
- `ag-domains` `CloudflareProvider` built its reqwest client with no `.timeout()`.
- `ag-mail` `ResendSender` used `reqwest::Client::new()` (no timeout).
- A slow/unresponsive endpoint could hang the call indefinitely (resource
  exhaustion). **Fixed:** both now set a 30s `.timeout(...)`.
- `ag-auth` OAuth uses an **injected** client (`OAuthClient::from_config(cfg,
  http)`), so the timeout is the caller's responsibility — not a defect here
  (recommend examples construct the client with a timeout).
- NATS (`ag-realtime`) connection uses `async_nats` `ConnectOptions`; retries and
  reconnection are handled by the client.

## Findings summary

| ID | Severity | Status |
| --- | --- | --- |
| S7-4 | Medium | Fixed — 30s timeout on Cloudflare + Resend reqwest clients |
| S7-1 | Medium | Open (non-blocking): `ag-realtime` `EventBuffer::append` (feature `event-persistence`) does a synchronous file open+write **per event** and reopens the file each call. If used on an async hot path it blocks the runtime. Recommend async I/O / `spawn_blocking` and a persistent handle. Opt-in feature, durability tradeoff. |
| S7-2 | Low | Open: native cache server spawns one task per connection with no connection cap; recommend a connection-limit semaphore (complements the Stage 4 RESP2 DoS fix). |
| S7-3 | Low | Open: mail retry spawns one task per retry (bounded by channel capacities); a timer/delay-queue would be cleaner. |

## Graceful shutdown

`ag-core` Shield serves via `hyper-util` graceful server; `ag-mail` queue worker
ends when senders drop; the cache/storage servers run until the listener drops.
No detached background task lacks a teardown path. Documented invariants are
consistent with the code.

## Gate rows advanced

Concurrency review complete; the one actionable timeout gap (S7-4) is fixed.
Remaining items (S7-1/2/3) are documented, non-blocking debt. Contributes to the
**API/DX** and overall robustness assessment for the final gate.
