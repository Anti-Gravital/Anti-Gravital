# Technical Debt Register (frozen historical record)

This file is a **frozen historical record**. Per CLAUDE.md rule 29, live
technical debt lives in GitHub Issues (label `tech-debt`), which are the single
honest view of project state. No new entries are added here; existing entries
are migrated to Issues when touched and replaced by their issue reference. The
file is not deleted, to preserve traceability.

For current, actionable debt see the GitHub Issues board filtered by the
`tech-debt` label.

> Convention (historical): each entry had reason, impact, expected removal,
> owning plan and target. Resolved or migrated entries below reference the
> commit/PR or the GitHub Issue that supersedes them.

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
- Reason: only the built-in `StringTemplate` (`{{var}}` substitution) engine
  existed; templates with loops/conditionals/filters were not supported.
- Status: closed (2026-06-04). The `MailTemplate` trait already abstracts the
  engine; `template::jinja::MinijinjaTemplate` (feature `minijinja`) implements
  it over the `minijinja` engine, supporting loops, conditionals and filters,
  as a drop-in alternative. `StringTemplate` stays the default with no extra
  dependency. Compile-time variable validation against the DSL remains via
  `template::validate` (unchanged).

### DEBT-017 — Native MTA: RSA DKIM keys
- Reason: `sender::mta::dkim` (feature `mta`) initially signed with Ed25519 only.
- Impact: domains that publish only an RSA DKIM key could not be signed natively.
- Status: closed (2026-06-04). `DkimConfig` now supports Ed25519 (PKCS#8 DER)
  and RSA-SHA256 (PKCS#8 DER and PEM) via `mail_auth ... RsaKey::from_key_der`;
  positive tests for both algorithms. (Was DEBT-012; renumbered to avoid a
  collision with the existing ag-realtime DEBT-012.)

### DEBT-018 — Native MTA: metrics and CI coverage
- Reason: the `mta` send path emitted only `tracing`, and CI did not exercise
  the `mta` feature.
- Impact: no `ag_mail_*` counters for native deliveries; `mta` code unbuilt in CI.
- Status: closed (2026-06-04). `MtaSender::send` records
  `ag_mail_sent_total`/`ag_mail_send_latency_seconds` and `ag_mail_retry_total`
  on MX failover; new `mail-mta` job in `.github/workflows/ci.yml` builds,
  tests and clippies `--features mta` (RFC-0009 section 4.8). (Was DEBT-013;
  renumbered to avoid a collision with the existing ag-cache DEBT-013.)

### DEBT-019 — Native MTA: two-tier queue, shaping and egress pools (Phase 4.6-B)
- Reason: Phase 4.6-A was synchronous direct delivery, with no scheduled/ready
  queue, traffic shaping, or egress pools.
- Status: closed (2026-06-04). Implemented: `sender::mta::egress` (egress
  sources/pools with smooth weighted round-robin for IP warming),
  `sender::mta::shaping` (per-`site_name` token-bucket rate limit + connection
  cap), and `sender::mta::queue` (in-memory two-tier scheduled/ready queue with
  retry/backoff, max-age and `max_ready`, a `DeliveryBackend` trait that
  `MtaSender` implements, and a `run` worker). The optional **durable spool**
  (JetStream/PostgreSQL) for cross-restart persistence is split out as DEBT-023.

### DEBT-020 — Native MTA: asynchronous DSN/FBL intake
- Reason: bounce/complaint suppression was driven only by synchronous SMTP
  replies; asynchronous DSN (RFC 3464) and ARF feedback-loop messages were not
  parsed.
- Status: closed (2026-06-04). `sender::mta::dsn` parses DSN delivery-status
  and ARF feedback-report messages (`mail-parser`) and feeds the suppression
  list: `process_dsn` hard-bounces permanently-failed recipients, `process_arf`
  suppresses complaints. Pure field parsers plus MIME-part location, unit-tested
  with RFC-shaped sample messages. Wiring an inbound endpoint that calls these
  is the integrator's responsibility (and part of the REST API, DEBT-021).

### DEBT-023 — Native MTA: durable queue spool (JetStream / PostgreSQL)
- Reason: the two-tier queue (DEBT-019) is in-memory; in-flight deliveries do
  not survive a restart.
- Impact: a crash/restart loses scheduled (not-yet-delivered) MTA jobs.
- Expected removal: a `Spool` backend behind an opt-in feature
  (`queue-jetstream` and/or a PostgreSQL mirror), keeping the in-memory spool as
  the native default (`ADR-0009`).
- Environment: testable here and in CI. A local PostgreSQL (16 is present in the
  dev image) and a self-hosted NATS/JetStream binary both run as ephemeral
  services / CI service containers; neither requires a third party. The debt is
  the unimplemented backend, not a missing environment.
- Status: migrated to GitHub issue #151 (live tracking). A durable PostgreSQL
  spool now ships behind the `queue-postgres` feature
  (`sender::mta::spool::PostgresSpool`), mirroring the scheduled queue so jobs
  survive a restart; the in-memory tier stays the default (ADR-0009 rule 2). The
  backend-agnostic recover mechanism is covered by in-process tests; the live
  PostgreSQL round-trip is an `#[ignore]` test gated on `DATABASE_URL`. Owning
  plan: RFC-0009 section 4.2. Target: Phase 4.6-B.

### DEBT-021 — Native MTA: REST API, webhooks, marketing (Phases 4.6-C/D)
- Reason: the multi-tenant REST surface and the marketing objects
  (broadcasts/contacts/segments/topics, one-click unsubscribe) are not
  implemented.
- Impact: no managed email-sending HTTP API yet.
- Progress (narrowed 2026-06-04): Signed webhooks are implemented:
  `api::webhook` (feature `api`) signs/verifies HMAC-SHA256 over
  `{id}.{timestamp}.{payload}` with `whsec_` secrets, multi-signature headers
  and replay-window checks (constant-time verify). The HTTP routes, the
  PostgreSQL data model, idempotency, and the marketing objects remain.
- Expected removal: Phases 4.6-C/D per RFC-0009 section 4.5/4.6 (behind the
  `api` feature).
- Environment: testable here and in CI. The PostgreSQL data model and the REST
  routes run against a local/ephemeral PostgreSQL (16 is present in the dev
  image) or a CI service container; no third party is involved. The debt is the
  unimplemented surface, not a missing environment.
- Status: migrated to GitHub issue #152 (live tracking). Owning plan: RFC-0009. Target: Phase 4.6-C/D.

### DEBT-022 — Native MTA: live-delivery integration test
- Reason: the direct MX delivery path (`MtaSender::submit`, `resolve::resolve_mx`)
  is only covered by `#[ignore]` tests; the ESMTP/STARTTLS/DKIM protocol path is
  not yet exercised by an automated end-to-end test.
- Impact: the network path is exercised manually, not in automated CI.
- Environment: the *protocol path* is testable here and in CI against a local
  SMTP sink (e.g. `aiosmtpd` or a Rust sink container) plus a fixture resolver;
  outbound 443 is open so the sink/resolver are installable. What is genuinely
  unavailable is real delivery to an external public MX: outbound port 25 to the
  internet is blocked in this sandbox and on hosted CI (verified). That last mile
  stays a manual/staging gate; the in-CI test must target the local sink, not a
  real mailbox.
- Expected removal: add a CI service container acting as a sink MTA and a
  fixture resolver, then de-`ignore` the protocol-path delivery test. Real
  external-MX delivery remains a manual gate on a host with port 25 egress.
- Status: migrated to GitHub issue #153 (live tracking). Owning plan: RFC-0009 section 4.8. Target: Phase 4.6-B.

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
- Status: closed (stale, 2026-06-13). Route 53, Google Cloud DNS, Azure DNS
  and Namecheap adapters landed (issues #80-#83, tracking #76); see
  `crates/ag-domains/src/provider/`. Cloudflare is no longer the only adapter.

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
- Status: migrated to GitHub issue #154 (live tracking). Target: Phase 5+.

## Tooling

### DEBT-010 — Coverage gate in CI
- Reason: no cargo-tarpaulin gate; roadmap requires >=80% per crate.
- Expected removal: plan P6.
- Status: closed (P6, 2026-05-26). `coverage` job added to `.github/workflows/quality.yml`
  using cargo-tarpaulin with `--fail-under 80`. Runs on every push/PR to main.

### DEBT-011 — Unified installer
- Reason: the cross-platform installers originally lacked enforced parity and
  referred to release checksums that did not exist.
- Expected removal: plan P6.
- Status: closed (P6 corrective, 2026-06-10). `install.sh` (Linux/macOS) and
  `install.ps1` (Windows) enforce the workspace MSRV, build in release mode,
  and install `ag` via `cargo install --locked`. The authoritative installer
  hashes live in `checksums/installers.sha256`; CI checks MSRV parity, hashes,
  and shell syntax. See `docs/security/INSTALLATION_INTEGRITY.md`.

## Pre-Phase 5 audit (2026-05-29)

Non-blocking items found by the pre-Phase 5 audit (see `docs/audits/`). Blocking
findings were fixed in-branch and are not listed here.

### DEBT-012 - ag-realtime event-persistence does synchronous, per-event file I/O
- Reason: `EventBuffer::append` (feature `event-persistence`) opened and wrote the
  file on every call; if used on an async hot path it blocked the runtime.
- Impact: throughput degradation under high event rates when the feature is on.
- Expected removal: async I/O or `spawn_blocking` plus a persistent file handle.
- Status: closed. `EventBuffer` keeps a persistent append handle;
  `append_async` uses `spawn_blocking` behind a configurable semaphore
  (default 64 pending appends), and `flush_async` is the shutdown barrier.
  Tests cover bounded backpressure, permit reuse, concurrent integrity, replay
  policy, and flush behavior. Verified with
  `cargo test -p ag-realtime --features event-persistence` and Clippy.
  Source: `pre-fase5-concurrency.md` S7-1; issue #73.

### DEBT-013 - native cache server has no connection cap
- Reason: the RESP2 server spawned one task per connection without a limit.
- Impact: a connection flood could spawn many tasks (DoS surface). The Stage 4 fix
  already bounds per-command allocation.
- Expected removal: a connection-limit semaphore in the accept loop.
- Status: closed (`audit/fases-0-4-5-consolidation`). `NativeCacheServer::bind`
  now uses `DEFAULT_MAX_CONNECTIONS`; tests cover explicit limits and zero-limit
  rejection. Verified with `cargo test -p ag-cache --features native-server`
  (29 unit passed, 1 ignored benchmark, 13 RESP2 integration passed, 2 doc-tests).
  Source: `pre-fase5-concurrency.md` S7-2.

### DEBT-014 — broaden fuzz and property coverage
- Reason: Stage 4 added targets for RESP2/storage-key/signed-URL; Stage 5 added
  properties for ag-storage. ag-auth/ag-mail/ag-domains/ag-dsl still lack
  dedicated fuzz/property coverage.
- Impact: lower assurance on those parsers/validators (currently covered by unit
  tests only).
- Expected removal: add targets/properties per `pre-fase5-fuzzing.md` §4.6 and
  `pre-fase5-properties.md` § Recommended.
- Status: migrated to GitHub issue #155 (live tracking). Severity: Medium.

### DEBT-015 — 24h fuzz manual gate not yet executed
- Reason: only 60s smoke runs were executed in the audit; the 24h gate is manual.
- Impact: deep fuzz assurance pending.
- Expected removal: run `cargo +nightly fuzz run <target> -- -max_total_time=86400`
  per target on a Linux x86-64 host; record logs.
- Status: migrated to GitHub issue #156 (live tracking). Severity: Medium. Source: `pre-fase5-fuzzing.md` §4.5.

### DEBT-016 — example READMEs still in Spanish
- Reason: the five `examples/` READMEs predated ADR-0008 (English-canonical).
- Impact: cosmetic/consistency only; content was accurate.
- Status: closed (2026-06-04). All five `examples/*/README.md`
  (auth-mail-demo, realtime-chat, todo-api, ecommerce-api, ai-backend)
  translated to English, preserving code, tables and command examples.

## ag-domains control plane

### DEBT-024 — eTLD+1 via two-label heuristic (no Public Suffix List)
- Status: RESOLVED (issue #78, RFC-0016). A single shared
  `ag-domains::registrable::registrable_domain` is the only eTLD+1 source for
  `hostname` and `issuance`; PSL-correct behind the `psl` Cargo feature, with the
  two-label heuristic as the offline default (ADR-0009). See GitHub issue #78.

### DEBT-025 — ag-domains control plane deferred phases (RFC-0011 E-F)
- Reason: phases A, B, C and D are implemented. A = control-plane library +
  manual CLI flow. B = live edge listeners (`ag-edge` `server`/`tls`): HTTP-01
  responder, Host/:authority routing, canonical redirects and HTTPS with SNI
  certificate selection (real TCP/TLS tests). C = REST API (`ag-domains` `api`)
  with real-HTTP tests. D = SQL-backed store (`ag-domains` `sql-store`,
  Postgres) with `#[ignore]` integration tests. Remaining: provider automation /
  Domain Connect / additional adapters (phase E; Cloudflare sync and BIND export
  already exist), and the `ag-registrars` module (phase F).
- Impact: E/F require external services (provider credentials) to exercise end
  to end; the native default flow (attach -> instructions -> ownership -> serve
  HTTPS via edge) and the optional SQL store are complete and tested.
- Expected removal: implement phases E-F per RFC-0011, each additive and
  feature-gated with a native default; service-backed tests use the repo's
  `#[ignore]` convention.
- Status: migrated to tracking issue #76 (live tracking). Severity: Medium. Source: RFC-0011 §5.
