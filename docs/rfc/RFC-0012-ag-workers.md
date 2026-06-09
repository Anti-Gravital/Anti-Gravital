# RFC-0012: ag-workers — Rust-native background execution engine

- Status: accepted
- Author: Angel Nereira (BDFL), Gravital Labs
- Draft date: 2026-06-08
- Target phase: 4.6-D (pre-Phase-5 extraction/hardening, sibling of 4.6-A `mta`,
  4.6-C `api`)
- Modules or crates affected: `ag-workers` (new, estándar diferido), `ag-dsl`
  (new `worker` declaration), `ag-cli` (new `ag workers` subcommands), `ag-data`
  (consumed for the durable backend, unchanged contract), `ag-observe` (consumed),
  `ag-mail` (later migration of its generic queue), `ag-edge` (producer-only
  consumer)
- Predecessor RFC: none (extracts the proven queue pattern from `ag-mail`)
- Governing ADR: ADR-0013 (this engine's execution model and the authorization to
  implement before the pre-Phase-5 gate closes)
- Comment period: waived by BDFL decision. This RFC records an explicit,
  owner-authorized extraction recorded per CLAUDE.md rules 22 and 28 (RFC required
  before new crates, new CLI commands, DSL changes and module-boundary changes).

> Release-gate honesty. The pre-Phase-5 release gate
> (`docs/audits/PRE_FASE5_RELEASE_GATE.md`) is open and Phase 1 external exit
> criteria remain unmet. This RFC specifies a complete target design and the owner
> authorizes implementation, but no production-readiness or GA claim is made for
> `ag-workers` until the gate permits it (see ADR-0013, §9 here, and the acceptance
> criteria in §5).

## 1. Motivation

Anti-Gravital needs a single, Rust-native substrate to move non-request work out of
HTTP handlers — background jobs, retries, dead-letter handling, scheduled tasks and
worker pools — while preserving the framework's core properties: no mandatory
external runtime, schema-first contracts, strict acyclic crate boundaries,
native observability, operational simplicity (single binary by default), bounded
concurrency and explicit configuration.

The pattern is not speculative. `ag-mail` already ships it in miniature:
`crates/ag-mail/src/queue/mod.rs` (`InMemoryQueue`, exponential-backoff worker) and
`crates/ag-mail/src/queue/store.rs` (`PersistentQueue` over `ag-data`, `RetryPolicy`,
`next_retry_at`, `FOR UPDATE SKIP LOCKED` leasing) implement retry, backoff, a
durable queue and worker execution inside one module. Phase 4.6 is actively expanding
`ag-mail` (4.6-A `mta`, 4.6-C `api`), and each future consumer that needs deferred
work (webhook delivery, ACME renewal in `ag-domains`/`ag-edge`, post-upload
processing in `ag-storage`, notification fanout, report generation) will otherwise
grow its own queue, retry, backoff, scheduler and metrics. That fragmentation is the
technical debt this RFC prevents.

The decision is to extract the proven pattern into a shared crate without breaking
`ag-mail`'s existing queue, so every future consumer builds on one boring, durable,
observable substrate.

## 2. Problem

A per-module queue cannot answer, consistently across the ecosystem:

- One job model with a typed payload, a closed handler registry and a uniform
  lifecycle.
- Durable, restart-surviving execution with retry, backoff and a dead-letter queue
  that never silently drops data.
- Atomic enqueue with the caller's database writes (no dual-write hazard).
- Correct behavior under the workspace `panic = "abort"` profile, including
  protection against a job that crash-loops the process.
- Distributed-singleton scheduling so an interval job fires once across N processes,
  not N times.
- Backpressure and admission control on every enqueue path.
- At-least-once execution with idempotency support and payload versioning across
  deploys.

Each of these implemented once, in `ag-mail` and again elsewhere, diverges in
reliability semantics. The ecosystem needs a single substrate.

## 3. Scope decision and boundaries

### 3.1 In scope (fixed; does not grow afterward)

```text
- one mental model: the job (typed payload, queue, retry policy, identity)
- typed JobHandler trait with a closed registry
- two first-class backends: in-memory (default) and PostgreSQL (durable, via ag-data)
- transactional enqueue (enqueue_in_tx) committing job + caller writes atomically
- retry with deterministic backoff/jitter, max attempts and max age
- a durable Dead-Letter Queue (DLQ) that never silently drops data
- poison-job circuit breaker, correct under panic = "abort"
- static and bounded-dynamic worker pools
- async I/O jobs and bounded CPU-bound jobs (no HTTP-path starvation)
- delayed and fixed-interval scheduling with distributed-singleton semantics in v1
- backpressure / admission control on every enqueue path
- at-least-once execution with idempotency support and payload versioning
- embedded, standalone, distributed and producer-only runtime modes
- full observability via ag-observe (metrics, tracing, structured logs)
- Anti-DSL `worker` declarations generating typed payloads and handler stubs
- feature-gated `ag workers ...` CLI
- graceful shutdown that never loses leased durable jobs
- a documented, test-backed migration path for ag-mail's generic queue
```

### 3.2 Explicitly out of scope (v1 commitment)

```text
- not a Kubernetes/Docker/NATS/PostgreSQL/Redis replacement
- not a distributed workflow / saga / job-graph engine (deferred)
- no mandatory external broker (NATS/Redis) — optional future backends only
- no separate worker process required by default
- no arbitrary untrusted-code execution; no runtime plugin model in v1
- no false panic-isolation claim that contradicts panic = "abort"
- no exactly-once promise (at-least-once with idempotency support is the contract)
- no control plane (scale/pause/resume/drain) in v1
```

### 3.3 Native-first invariant (ADR-0009 rules 2 and 5)

The default backend is in-memory (pure Rust, no external service). The durable
backend is PostgreSQL via `ag-data`, behind the `postgres` Cargo feature; it reuses
`ag-data`'s pool and embedded-migration machinery rather than opening a parallel
`sqlx` surface. NATS/JetStream and Redis backends are deferred and feature-gated.
No external service is ever required to use `ag-workers`.

## 4. Architecture

```text
job declaration (DSL or Rust)
  -> typed payload (rmp-serde, versioned)
  -> admission control (backpressure)
  -> queue backend (memory | postgres)
  -> lease by a worker (FOR UPDATE SKIP LOCKED on postgres)
  -> handler execution (async on Tokio | cpu-bound on spawn_blocking + semaphore)
  -> outcome classification (Complete | Retry | Discard | DeadLetter)
  -> ack | retry (backoff) | dead-letter | discard
  -> metrics / tracing / structured logs (ag-observe)
```

Dependency direction (acyclic; `ag-core` unchanged and independent):

```text
ag-workers -> { ag-data?, ag-observe?, ag-cache?, ag-realtime? }  (all feature-gated)
ag-dsl     -> generates worker contracts and handler stubs (no runtime link)
ag-cli     -> ag-workers via feature-gated command integration
ag-mail    -> may later consume ag-workers through an adapter (§5, M2-M4)
ag-edge    -> producer-only consumer of the enqueue API; never hosts workers
```

Forbidden (would create cycles or violate layering): `ag-workers` must not depend on
`ag-mail`, `ag-auth`, `ag-cloud`, `ag-domains`, `ag-edge`, `ag-ui` or `ag-cli`.
`ag-workers` classification is estándar diferido (ADR-0007 precedent, `ag-mail`):
production-grade but not installed by default in official templates; pulled in when
`schema.ag` declares a `worker` or a feature enables it.

Normative design points (full detail in ADR-0013 and the module doc
`docs/modules/ag-workers/README.md`):

- Job lifecycle is an explicit state machine; the persistence layer rejects illegal
  transitions (`scheduled`, `queued`, `leased`, `running`, `succeeded`,
  `retry_scheduled`, `failed`, `dead_lettered`, `cancelled`, `expired`).
- Payloads are `rmp-serde`-encoded `BYTEA` with a `PAYLOAD_VERSION`; decode/version
  failure routes to the DLQ as `InvalidPayload`, never panics, never drops.
- Poison-job circuit breaker: the attempt counter increments at lease time, and a
  job exceeding `panic_guard_attempts` is routed straight to the DLQ without
  executing — converting an infinite crash loop (a job that aborts the process every
  time, under `panic = "abort"`) into a bounded, observable DLQ entry. This is the
  most important production-safety property.
- CPU-bound jobs run on `tokio::task::spawn_blocking` gated by a fixed-size
  `Semaphore` (not `rayon`), so they never starve HTTP latency.
- Scheduling uses a `FOR UPDATE SKIP LOCKED` singleton claim on `ag_worker_schedules`
  so interval jobs fire once under horizontal scaling.
- Observability metric labels are bounded (`queue`, `kind`, `status`, `outcome`,
  `reason`); `tenant_id` is never a metric label (cardinality).

## 5. Phased plan (additive, non-regressive; fixed scope, ordered landing)

The scope of §3.1 is fixed and does not grow. What is sequenced is the landing order,
because `CONTRIBUTING.md` requires small, single-purpose pull requests. Each stage is
independently mergeable, fully tested, and leaves the tree green.

```text
S1 — Foundations: crate skeleton; ids, job (envelope + state machine + priority),
     payload (rmp + versioning + migrate hook), error/outcome, handler, registry.
     No runtime yet.
S2 — Memory runtime: MemoryQueue; static workers; dispatch loop; retry/backoff;
     per-job timeout; in-memory DLQ; poison guard; graceful shutdown; telemetry.
     Default for `ag dev`.
S3 — Persistence: PostgresQueue via ag-data; embedded migrations; FOR UPDATE SKIP
     LOCKED leasing; heartbeat + reaper; persistent DLQ; enqueue_in_tx;
     admission/backpressure. Gated by the `postgres` feature.
S4 — Scheduling + dynamic: interval jobs with distributed-singleton claim; bounded
     dynamic pools; CPU-bound pool (spawn_blocking + semaphore).
S5 — Surfaces: Anti-DSL `worker` declarations + generators; `ag workers ...` CLI
     (feature-gated); examples.
S6 — Producer mode + edge: `producer` feature; enqueue-only wiring for
     ag-edge/serverless.
S7 — Mail migration: M0 document the overlap (DEBT) -> M1 ship ag-workers without
     touching ag-mail -> M2 optional `workers` feature in ag-mail routing generic
     delivery scheduling through ag-workers (mail-specific SMTP/MTA/DKIM/bounce stays
     in ag-mail) -> M3 reconcile tables under test -> M4 deprecate the duplicated
     generic queue once parity is proven. Close the DEBT entry on M4.
```

Acceptance (RFC §, mirrors the standard-module bar): RFC-0012 accepted; ADR-0013
recorded; crate added with validated acyclic dependency direction; `cargo fmt` clean;
`cargo clippy --workspace --all-targets --all-features -- -D warnings` clean;
`cargo test --workspace --all-features` green (Postgres integration via the existing
`testcontainers-modules` postgres); `cargo audit` and `cargo deny check` clean;
coverage >= 80% on the crate; fuzz targets for the worker DSL grammar and payload
decoder; reproducible `criterion` benchmarks; examples build and run; docs match code
(no claims of unimplemented features); no production-readiness/GA claim before the
release gate allows it.

## 6. Backward compatibility

`ag-workers` is purely additive. No existing crate's public contract changes. The
`ag-mail` queue (`queue/mod.rs`, `queue/store.rs`) is untouched at introduction and
keeps working; the migration (§5, S7/M0-M4) is feature-gated and only deprecates the
duplicated generic queue after tests prove parity. `ag-data` is consumed through its
existing public API (`DbPool`, `connect`, `run_migrations`) plus a raw
`sqlx::Transaction` handle for `enqueue_in_tx`; `ag-data` is not modified. The new
DSL `worker` declaration is additive to the grammar (parallel to `event`, v0.6); a
schema with no `worker` block generates exactly as before. The `ag workers`
subcommands are additive and feature-gated; minimal CLI builds are unaffected.

## 7. Risks

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| Crash loop under `panic = "abort"` from a poison job | Medium | High | Poison guard (§4): attempt increment at lease time + `panic_guard_attempts` -> direct DLQ. Property/integration tested. |
| Scope drift into a workflow/orchestration platform | Medium | High | Boundaries in §3.2; workflow/saga and control plane are explicit non-goals and deferred. |
| Two persistent Postgres queues (ag-mail + ag-workers) coexisting forever | Medium | Medium | Staged migration §5 S7 with a DEBT entry retired only after parity tests. |
| New dependency (`tokio-util`) increases surface | Low | Low | Single, minimal, first-party dep for `CancellationToken`; justified here (§9). |
| `enqueue_in_tx` leaks `sqlx` into call sites (no AgTx in ag-data) | Low | Low | Confined behind the `postgres` feature; matches ag-mail's existing raw-sqlx usage; tracked as a GitHub Issue (label `tech-debt`, per CLAUDE.md rule 29). A future canonical `AgTx` is a separate RFC. |
| Implementing before the pre-Phase-5 gate closes | Medium | Medium | Owner-authorized in ADR-0013; no GA claim; additive and feature-gated; reversible (§8). |

## 8. Rollback

`ag-workers` is an additive workspace member with feature-gated optional deps and a
native in-memory default. If rejected or deferred after partial landing, removal is a
workspace-member deletion plus reverting the DSL/CLI additions and the doc rows; no
migration is destructive (the default backend is in-memory). The `ag-mail` queue is
never removed before its migration's parity tests pass, so reverting the migration
restores the status quo. If the whole RFC is rejected, module-specific queues remain
and the duplication risk is recorded in `docs/DEBT.md`.

## 9. Decision

- Decider: Angel Nereira (BDFL).
- Date: 2026-06-08.
- Result: accepted (owner-authorized extraction; implementation authorized before the
  pre-Phase-5 gate closes, with no GA claim until the gate permits).
- Locked design decisions (recorded in ADR-0013):
  1. Delivery: documentation lands first; code lands in the sequenced stages S1-S7,
     each green (`fmt`/`clippy`/`test`).
  2. Cancellation (§4): add `tokio-util` (0.7, `default-features = false`) to the
     workspace and use `CancellationToken`. This is the only genuinely new dependency;
     everything else reuses existing workspace deps (`rmp-serde`, `sqlx`, `chrono`,
     `uuid`, `tokio`, `async-trait`, `tracing`, `metrics`, `thiserror`, `proptest`,
     `criterion`, `testcontainers`).
  3. `enqueue_in_tx` accepts `&mut sqlx::Transaction<'_, Postgres>` behind the
     `postgres` feature (no `AgTx` in `ag-data` today; matches `ag-mail`'s raw-sqlx
     usage). The "no canonical AgTx handle" gap is tracked as a GitHub Issue
     (label `tech-debt`), per CLAUDE.md rule 29; `docs/DEBT.md` is frozen.
- Justification: recorded per CLAUDE.md rules 22 and 28. ADR-0013 records the
  architectural decision, the consequences and the authorization.

## 10. References

- ADR-0007 — ag-mail / ag-domains decision (introduced the estándar diferido
  classification; `ag-mail` precedent).
- ADR-0013 — ag-workers execution model and authorization (governing ADR).
- ADR-0009 — native-first / real-state governance.
- ADR-0008 — language policy (English canonical for technical docs and code).
- `crates/ag-mail/src/queue/mod.rs`, `crates/ag-mail/src/queue/store.rs` — the proven
  queue pattern being extracted.
- `docs/audits/PRE_FASE5_RELEASE_GATE.md` — the open release gate.
- `docs/DEBT.md` — frozen historical register; DEBT-019/021/023 (ag-mail MTA queue).
  New ag-workers tech-debt is tracked as GitHub Issues (label `tech-debt`), per
  CLAUDE.md rule 29.
- PostgreSQL `FOR UPDATE SKIP LOCKED` (concurrent work-queue leasing).
- Transactional outbox pattern (the `enqueue_in_tx` property without a separate
  outbox table).
