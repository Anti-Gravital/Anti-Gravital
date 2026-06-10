# ag-workers

Rust-native background execution engine for Anti-Gravital: typed jobs, retries, a
dead-letter queue, scheduling and worker pools. In-memory backend by default;
PostgreSQL durable backend opt-in behind the `postgres` feature. No external service is
required to use the crate (native-first, ADR-0009).

Classification: estándar diferido (deferred standard). Decision: `RFC-0012` /
`ADR-0013`. Architecture and module docs: `docs/modules/ag-workers/README.md`.

## Status

In progress. Delivered in sequenced stages S1-S7 (RFC-0012 §5). This README and the
crate's `//!` headers state the real status of what is implemented; outstanding work is
tracked as GitHub Issues (label `tech-debt`, CLAUDE.md rule 29). No production-readiness
or GA claim is made until the pre-Phase-5 release gate
(`docs/audits/PRE_FASE5_RELEASE_GATE.md`) closes.

Implemented so far (S1-S3):

- S1 foundations: job identity types; `JobEnvelope`/`NewJob`/`JobStatus` with a
  normative transition table and `JobPriority`; versioned `rmp-serde` payload encoding
  with a `migrate` hook (decode/version failures become `InvalidPayload`, never a panic
  or silent drop); the `JobHandler`/`JobContext` contract with cooperative cancellation;
  the `WorkerError` taxonomy and `JobOutcome`; the closed type-erased registry.
- S2 memory runtime: `QueueBackend` trait + in-memory backend (bounded depth, payload
  limit, priority leasing, expired-lease reclaim); the dispatch core with the poison-job
  circuit breaker correct under `panic = "abort"`; exponential backoff with
  `max_delay`/`max_age`; a durable-never-drop DLQ; admission outcomes; graceful
  shutdown; the static worker pool; feature-gated `ag_workers_*` telemetry.
- S3 persistence (`postgres` feature): `PostgresQueue` over `ag-data` with embedded
  migrations, `FOR UPDATE SKIP LOCKED` leasing, heartbeat, a reaper for expired leases,
  transactional `enqueue_in_tx`, and a persistent dead-letter table. Integration tests
  are `#[ignore]` and require `DATABASE_URL`.

- S4 scheduling + pools: delayed and fixed-interval scheduling with a distributed
  singleton claim (`MemoryScheduleStore`; `PostgresScheduleStore` via `FOR UPDATE SKIP
  LOCKED` on `ag_worker_schedules`); a bounded dynamic worker pool (`DynamicPool`,
  scaling between `min`/`max` from queue depth); and a bounded CPU-bound pool (`CpuPool`,
  `spawn_blocking` + semaphore, not rayon).

The DSL `worker` declaration, the CLI, producer mode and the `ag-mail` migration land in
the following stages (S5-S7).

## Features

- `memory` (default): in-memory backend.
- `metrics` (default): emit `ag_workers_*` metrics through the `metrics` crate.
- `postgres`: durable PostgreSQL backend via `ag-data`.
- `producer`: enqueue-only mode for edge/serverless targets.
- `scheduler`: delayed and fixed-interval scheduling.
- `test-utils`: helpers (NullBackend, deterministic clock, fixtures) for downstream
  crate tests.

## Configuration

`WorkersConfig` (RFC-0012 §28) is the deployment-level configuration. It loads from a
`[workers]` TOML section and is overridable through `AG_WORKERS_*` environment
variables (environment wins over TOML wins over defaults):

```toml
[workers]
enabled = true
mode = "embedded"          # embedded | standalone | distributed | producer
backend = "memory"         # memory | postgres
shutdown_timeout = "30s"
max_payload_bytes = 262144
poison_guard_attempts = 3

[workers.queue.mail]
min_workers = 2
max_workers = 4
max_depth = 10000
lease_timeout = "60s"
heartbeat_interval = "15s"

[workers.queue.media]
kind = "cpu"               # CPU-bound queue (blocking pool)
max_workers = 4
```

```rust
use ag_workers::WorkersConfig;

// TOML + AG_WORKERS_* overrides; then derive the dispatch-loop config.
let cfg = WorkersConfig::load(toml_src)?;
let runtime = cfg.runtime_config();
```

Environment overrides: `AG_WORKERS_ENABLED`, `AG_WORKERS_MODE`, `AG_WORKERS_BACKEND`,
`AG_WORKERS_SHUTDOWN_TIMEOUT`, `AG_WORKERS_MAX_PAYLOAD_BYTES`,
`AG_WORKERS_POISON_GUARD_ATTEMPTS`, and per-queue `AG_WORKERS_QUEUE_<NAME>_MIN` /
`_MAX`.
