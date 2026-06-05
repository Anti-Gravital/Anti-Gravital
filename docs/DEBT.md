# Technical Debt Register

Single source of truth for tracked technical debt across Anti-Gravital. Every
"skeleton"/TODO/TECH-DEBT marker in the codebase must point here. Format per
CLAUDE.md section 29.

> Convention: each entry has reason, impact, expected removal, owning plan and
> target. Dates are absolute. Close an entry only when the code and its plan agree.

## ag-mail

### DEBT-001 — Persistent queue backend
- Reason: the queue is in-memory only; messages are lost on restart.
- Impact: no delivery guarantees across restarts; no retry durability.
- Expected removal: plan P2 (`queue-persistent` feature over `ag-data`).
- Status: closed (P2, 2026-05-26). `PersistentQueue` implemented in
  `crates/ag-mail/src/queue/store.rs` with `FOR UPDATE SKIP LOCKED` worker.

### DEBT-002 — Custom SMTP headers ignored
- Reason: lettre limitations drop arbitrary custom headers in the SMTP adapter.
- Impact: custom headers set on `Email` are silently ignored over SMTP.
- Expected removal: plan P2 (review lettre API or contribute upstream).
- Status: closed (P2, 2026-05-26). Implemented via `lettre::message::HeaderValue::new`
  + `MessageBuilder::raw_header` in `SmtpSender`. Non-ASCII header names are logged
  and skipped gracefully.

### DEBT-003 — External template engines
- Reason: only the built-in `StringTemplate` engine exists.
- Impact: no askama/minijinja support; no compile-time variable validation vs DSL.
- Expected removal: future plan; trait-based engine selection.
- Status: open. Target: Phase 5+.

## ag-cache

### DEBT-004 — Native L2 over RESP2 (no Redis)
- Reason: L2 is a stub that only logs a warning; Redis/fred is not wired.
- Impact: no distributed cache; vendor-lock risk if Redis is added directly.
- Expected removal: plan P5, gated on RFC-0005 approval.
- Status: closed (P5, 2026-05-26). `NativeCacheServer` implemented under feature
  `native-server` (RFC-0005). Supports GET/SET/DEL/EXISTS/MGET/MSET/EXPIRE/TTL/KEYS/
  PING/FLUSHDB/DBSIZE/COMMAND. Per-key TTL via `DashMap`. 13 integration tests via
  raw TcpStream. External Redis L2 (multi-instance) remains deferred (TECH-DEBT in lib.rs).

## ag-domains

### DEBT-005 — notAfter parsing for scheduled renewal
- Reason: `acme/renewal.rs` renews every cycle because `notAfter` is not parsed.
- Impact: unnecessary ACME calls; no date-based renewal or near-expiry alerts.
- Expected removal: plan P3.
- Status: closed (P3, 2026-05-26). `parse_not_after` implemented via `x509-parser`.
  `spawn_renewal_task` now sleeps until the renewal window using `seconds_until_renewal`.
  `ag_domains_cert_days_until_expiry` gauge exported after each successful issuance.

### DEBT-006 — Additional DNS adapters
- Reason: only Cloudflare adapter exists.
- Impact: limited provider choice (optional, not blocking).
- Expected removal: future, as opt-in adapters behind features.
- Status: open. Target: Phase 5+.

## ag-realtime

### DEBT-007 — Scalability proof (50k connections)
- Reason: no load test demonstrates the Phase 4 criterion of 50,000 connections.
- Impact: scalability claim is unverified.
- Expected removal: plan P4.
- Status: closed (P4, 2026-05-26). `tests/load_50k.rs` runs 50k subscribers, achieves
  100% delivery in < 35 ms (debug). Methodology in `docs/benchmarks/ag-realtime-load.md`.
  Marked `#[ignore]`; run in manual gate.

### DEBT-008 — Event persistence buffer
- Reason: in-memory event bus loses critical events on restart.
- Impact: no durability for critical events.
- Expected removal: plan P4 (optional `event-persistence` feature).
- Status: closed (P4, 2026-05-26). `EventBuffer` (append-only NDJSON) + `replay_into_bus`
  implemented in `crates/ag-realtime/src/persistence.rs` under `event-persistence` feature.

## ag-data

### DEBT-009 — DSL-generated typed ORM / RLS / multi-tenancy
- Reason: typed queries, row-level security and multi-tenancy come from the DSL.
- Impact: advanced data features unavailable until generated.
- Expected removal: later phases (Phase 3 ORM, later for RLS/multi-tenancy).
- Status: open. Target: Phase 5+.

## Tooling

### DEBT-010 — Coverage gate in CI
- Reason: no cargo-tarpaulin gate; roadmap requires >=80% per crate.
- Expected removal: plan P6.
- Status: closed (P6, 2026-05-26). `coverage` job added to `.github/workflows/quality.yml`
  using cargo-tarpaulin with `--fail-under 80`. Runs on every push/PR to main.

### DEBT-011 — Unified installer
- Reason: no install.sh / install.ps1.
- Expected removal: plan P6.
- Status: closed (P6, 2026-05-26). `install.sh` (Linux/macOS) and `install.ps1` (Windows)
  added to repo root. Both verify the Rust toolchain, build the workspace in release mode,
  and install `ag` via `cargo install --locked`. Auditable per ADR-0009 rule 4.

## Pre-Phase 5 audit (2026-05-29)

Non-blocking items found by the pre-Phase 5 audit (see `docs/audits/`). Blocking
findings were fixed in-branch and are not listed here.

### DEBT-012 — ag-realtime event-persistence does synchronous, per-event file I/O
- Reason: `EventBuffer::append` (feature `event-persistence`) opens and writes the
  file on every call; if used on an async hot path it blocks the runtime.
- Impact: throughput degradation under high event rates when the feature is on.
- Expected removal: async I/O or `spawn_blocking` + a persistent file handle.
- Status: open. Severity: Medium. Source: `pre-fase5-concurrency.md` S7-1.

### DEBT-013 — native cache server has no connection cap
- Reason: the RESP2 server spawns one task per connection without a limit.
- Impact: a connection flood can spawn many tasks (DoS surface). The Stage 4 fix
  already bounds per-command allocation.
- Expected removal: a connection-limit semaphore in the accept loop.
- Status: open. Severity: Low. Source: `pre-fase5-concurrency.md` S7-2.

### DEBT-014 — broaden fuzz and property coverage
- Reason: Stage 4 added targets for RESP2/storage-key/signed-URL; Stage 5 added
  properties for ag-storage. ag-auth/ag-mail/ag-domains/ag-dsl still lack
  dedicated fuzz/property coverage.
- Impact: lower assurance on those parsers/validators (currently covered by unit
  tests only).
- Expected removal: add targets/properties per `pre-fase5-fuzzing.md` §4.6 and
  `pre-fase5-properties.md` § Recommended.
- Status: open. Severity: Medium.

### DEBT-015 — 24h fuzz manual gate not yet executed
- Reason: only 60s smoke runs were executed in the audit; the 24h gate is manual.
- Impact: deep fuzz assurance pending.
- Expected removal: run `cargo +nightly fuzz run <target> -- -max_total_time=86400`
  per target on a Linux x86-64 host; record logs.
- Status: open. Severity: Medium. Source: `pre-fase5-fuzzing.md` §4.5.

### DEBT-016 — example READMEs still in Spanish
- Reason: the five `examples/` READMEs predate ADR-0008 (English-canonical) and
  remain in Spanish. ADR-0008 permits gradual on-touch migration and does not
  block.
- Impact: cosmetic/consistency only; content is accurate.
- Expected removal: translate to English when each example is next touched.
- Status: open. Severity: Low. Source: `pre-fase5-examples.md` F9-6.

### DEBT-017 — eTLD+1 via two-label heuristic (no Public Suffix List)
- Reason: `ag-domains::hostname` derives the registrable domain by taking the
  last two labels. Multi-label public suffixes (`co.uk`, `com.br`, etc.) are
  misclassified, which skews apex vs subdomain detection and generated DNS
  instructions for those TLDs.
- Impact: incorrect instructions/classification for domains under multi-label
  public suffixes. Single-label TLDs (`.com`, `.net`, `.io`) are correct.
- Expected removal: a PSL-backed implementation behind a dependency RFC
  (candidate crate: `psl` or `publicsuffix`); gate it so the native default
  keeps working offline.
- Status: open. Severity: Medium. Source: RFC-0009 §7.

### DEBT-018 — ag-domains control plane deferred phases (RFC-0009 B-F)
- Reason: phase A landed the control-plane library, the `ag-edge` data-plane
  library and the manual CLI flow. Live edge listeners (80/443) + HTTP-01
  responder (phase B), REST API `/v1/domains/...` (phase C), SQL-backed store
  (phase D), provider automation / Domain Connect / DNS-01 wildcards (phase E)
  and the `ag-registrars` module (phase F) are not implemented.
- Impact: native serving of HTTPS for an attached hostname is not wired end to
  end yet; attachment, instructions, ownership proof and routing logic exist and
  are tested in isolation.
- Expected removal: implement phases B-F per RFC-0009, each additive and
  feature-gated with a native default.
- Status: open. Severity: Medium. Source: RFC-0009 §5.
