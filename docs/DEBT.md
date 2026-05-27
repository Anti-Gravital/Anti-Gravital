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
