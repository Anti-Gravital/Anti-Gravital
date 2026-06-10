# ag-workers

> Architecture chapter: `docs/architecture/08-modulos-batteries-included.md`, section 8.10.
> Decision RFC: `docs/rfc/RFC-0012-ag-workers.md`.
> Decision ADR: `docs/adr/0013-ag-workers-execution-model.md`.
> Roadmap phase: Phase 4.6-D (pre-Phase-5 extraction/hardening).
> Crate README: `crates/ag-workers/README.md`.
> Criticality: Estándar diferido (deferred standard, ADR-0007 precedent).
> Implementation status: S1-S5 implemented and CI-verified; S6 partial (#112);
> S7 M0-M2 done, M3/M4 blocked on a live PostgreSQL database (#109/#103, with #108
> for the `#[ignore]` integration tests). No GA claim until the pre-Phase-5
> release gate (`docs/audits/PRE_FASE5_RELEASE_GATE.md`) closes.

## Domain

`ag-workers` is the Rust-native background execution engine for Anti-Gravital. It moves
non-request work out of HTTP handlers: background jobs, retries, dead-letter handling,
delayed and fixed-interval scheduling, and worker pools. It is the shared substrate that
replaces per-module queues, extracted from the proven pattern already in `ag-mail`
(`crates/ag-mail/src/queue/`).

## Scope (Phase 4.6-D)

### Yes

- One job model: a typed payload, a queue, a retry policy and an identity.
- Typed `JobHandler` trait with a closed registry.
- Two first-class backends: in-memory (default) and PostgreSQL (durable, via `ag-data`).
- Transactional enqueue (`enqueue_in_tx`) committing job + caller writes atomically.
- Retry with deterministic backoff/jitter, max attempts and max age.
- A durable Dead-Letter Queue that never silently drops data.
- Poison-job circuit breaker, correct under `panic = "abort"`.
- Static and bounded-dynamic worker pools; async I/O and bounded CPU-bound execution.
- Delayed and fixed-interval scheduling with distributed-singleton semantics.
- Backpressure / admission control on every enqueue path.
- At-least-once execution with idempotency support and payload versioning.
- Embedded, standalone, distributed and producer-only runtime modes.
- Observability via `ag-observe`; Anti-DSL `worker` declarations; `ag workers ...` CLI.

### No

- Not a Kubernetes/Docker/NATS/PostgreSQL/Redis replacement.
- Not a workflow / saga / job-graph engine (deferred).
- No mandatory external broker; no separate worker process by default.
- No arbitrary untrusted-code execution; no runtime plugin model in v1.
- No exactly-once promise; no false panic-isolation claim under `panic = "abort"`.
- No control plane (`scale`/`pause`/`resume`/`drain`) in v1.

## Technical stack

| Component | Library | Notes |
|---|---|---|
| Async trait | `async-trait` | `JobHandler`, `QueueBackend`. |
| Payload encoding | `rmp-serde` | Compact, schema-tolerant `BYTEA`; versioned. |
| Durable backend | `ag-data` (`sqlx`, Postgres) | `postgres` feature; `FOR UPDATE SKIP LOCKED`. Reuses `ag-data` pool + migrations. |
| Timestamps / ids | `chrono`, `uuid` | `postgres` feature. |
| Cancellation | `tokio-util` | `CancellationToken` for cooperative cancellation (new workspace dep, ADR-0013). |
| Concurrency | `tokio` | Tasks, `spawn_blocking` + `Semaphore` for CPU-bound jobs. |
| Observability | `ag-observe` (`metrics`, `tracing`) | Bounded metric labels; `tenant_id` is never a label. |
| Errors | `thiserror` | `WorkerError` taxonomy. |

The only genuinely new workspace dependency is `tokio-util`; everything else reuses
existing workspace deps. Per CLAUDE.md rule 15.

## Public API (shape)

```rust
#[async_trait::async_trait]
pub trait JobHandler: Send + Sync + 'static {
    type Payload: serde::de::DeserializeOwned + serde::Serialize + Send + Sync + 'static;

    /// Stable, unique identifier for this job kind. Closed set in the registry.
    fn kind(&self) -> &'static str;

    /// Payload schema version. Bump on incompatible payload changes.
    fn payload_version(&self) -> u16 { 1 }

    async fn handle(&self, ctx: JobContext, payload: Self::Payload)
        -> Result<JobOutcome, WorkerError>;
}

#[async_trait::async_trait]
pub trait QueueBackend: Send + Sync + 'static {
    async fn enqueue(&self, job: NewJob) -> Result<JobId, QueueError>;
    async fn lease(&self, queue: &QueueName, worker: &WorkerId, limit: usize)
        -> Result<Vec<JobEnvelope>, QueueError>;
    async fn ack(&self, job_id: JobId) -> Result<(), QueueError>;
    async fn retry(&self, job_id: JobId, decision: RetryDecision) -> Result<(), QueueError>;
    async fn dead_letter(&self, job_id: JobId, reason: DeadLetterReason)
        -> Result<(), QueueError>;
    // heartbeat, release_expired_leases, depth, claim_due_schedule, enqueue_in_tx ...
}
```

Small, versioned, with contract tests every backend must pass. Backends:
`MemoryQueue` (default) and `PostgresQueue` (durable, `postgres` feature). NATS/JetStream
and Redis backends are deferred and feature-gated.

## Allowed internal dependencies

- May depend on `ag-data` (`postgres` feature), `ag-observe` (metrics/tracing),
  `ag-cache` (optional idempotency hints) and `ag-realtime` (optional future backend),
  all feature-gated.
- **Must not depend on** `ag-mail`, `ag-auth`, `ag-cloud`, `ag-domains`, `ag-edge`,
  `ag-ui` or `ag-cli` (would create cycles or violate layering).
- `ag-cli` depends on `ag-workers` only through a Cargo feature (fourth dependency rule).
- `ag-mail` may later consume `ag-workers` through an adapter (RFC-0012 §5, S7), never the
  reverse.
- `ag-edge` is a producer-only consumer of the enqueue API; it never hosts a worker
  runtime.

## DSL — `worker` block

```ag
worker SendWelcomeEmail {
  queue "mail"
  mode static
  concurrency 4
  timeout "30s"
  retry {
    max_attempts 5
    backoff exponential
    initial_delay "1s"
    max_delay "5m"
    jitter true
  }
  input {
    user_id uuid
    email string @email
  }
  emits "mail.sent"
  on_failure dlq
}
```

The compiler generates: typed payload structs (with `PAYLOAD_VERSION`), `JobHandler`
stubs (intentional stubs the developer implements), closed-registry wiring, and SQL
migrations for persistent queues when the `postgres` feature is enabled. The grammar is
additive (parallel to the `event` declaration, DSL v0.6).

## Runtime modes

- **Embedded** — workers run inside the app process; default for development.
- **Standalone** — workers run as a separate process (`ag workers run --queue mail`).
- **Distributed** — multiple worker processes lease from the PostgreSQL backend.
- **Producer-only** — enqueue API only (no leasing loop/scheduler); for `ag-edge` and
  serverless targets that cannot host long-lived workers.

## Exit criteria (Phase 4.6-D gate)

Live tracking: `docs/roadmap/STATUS.md`, Phase 4.6-D. Summary of the current state:

- [x] Typed jobs execute on the memory backend (`ag dev`) with retry, backoff and DLQ
      (`tests/runtime_outcomes.rs`, `tests/retry_policy.rs`).
- [/] PostgreSQL backend leases with `FOR UPDATE SKIP LOCKED`, survives restart, and
      `enqueue_in_tx` commits job + caller writes atomically. Code and tests exist
      (`tests/postgres_queue.rs`) but are `#[ignore]` and need a live `DATABASE_URL`;
      running them is tracked in Issue #108.
- [x] Poison guard converts a crash-looping job into a bounded DLQ entry
      (`tests/poison_guard.rs`).
- [x] Interval jobs fire once via the singleton claim on the memory backend
      (`tests/scheduler_dynamic.rs`); the cross-process PostgreSQL verification is
      part of Issue #108.
- [x] `worker` DSL declaration compiles and generates payloads + handler stubs
      (DSL v0.8, `codegen/worker_gen.rs`).
- [x] `ag workers ...` CLI compiles and passes CI (feature-gated `workers-runtime`).
- [x] Coverage >= 80% on the `quality` gate; the `worker` grammar is fuzzed through
      the unified DSL targets and the payload decoder through `fuzz_workers_payload`
      (CI fuzz-smoke job).
- [x] No circular dependencies (CI green).
- [x] `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo audit`,
      `cargo deny check` green.
- [x] No production-readiness/GA claim before the pre-Phase-5 release gate allows it.

## Watched risks

- **Crash loop under `panic = "abort"`.** Mitigation: poison guard (attempt increment at
  lease time + `panic_guard_attempts` -> direct DLQ).
- **Scope drift into an orchestration platform.** Mitigation: boundaries fixed in
  RFC-0012 §3.2; workflow/saga and control plane are explicit non-goals.
- **Two persistent Postgres queues coexisting.** Mitigation: staged `ag-mail` migration
  (RFC-0012 §5, S7), retired only after parity tests.
