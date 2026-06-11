# ag-workers

Rust-native background execution engine for Anti-Gravital: typed jobs, retries, a
dead-letter queue, scheduling and worker pools. In-memory backend by default;
PostgreSQL durable backend opt-in behind the `postgres` feature. No external service is
required to use the crate (native-first, ADR-0009).

Classification: estándar diferido (deferred standard). Decision: `RFC-0012` /
`ADR-0013`. Architecture and module docs: `docs/modules/ag-workers/README.md`.

## Status

Delivered in sequenced stages S1-S7 (RFC-0012 §5). S1-S5 are implemented and verified
by CI; S6 is partial and S7 has milestones M0-M2 done — the remainder needs a live
PostgreSQL database and is tracked as GitHub Issues (label `tech-debt`, CLAUDE.md
rule 29): #108 (run the `#[ignore]` Postgres integration tests), #109 (S7/M3 mail-job
parity), #103 (S7/M4 removal of the deprecated `ag-mail` queue), #112 (dedicated
`ag-edge` producer wiring). This README and the crate's `//!` headers state the real
status of what is implemented. No production-readiness or GA claim is made until the
pre-Phase-5 release gate (`docs/audits/PRE_FASE5_RELEASE_GATE.md`) closes.

Implemented (S1-S5):

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
- S5 surfaces: the Anti-DSL `worker` declaration (DSL v0.8) with the `worker_gen`
  generator (typed payloads, `JobHandler` stubs, `register_workers` wiring); the
  operational `ag workers` CLI behind the `workers-runtime` feature of `ag-cli`
  (`list`, `run`, `enqueue`, `queues`, `dlq list|inspect|retry|purge`, `doctor`;
  durable-backend subcommands require `DATABASE_URL`); and the five runnable examples
  listed below.
- DLQ recovery (RFC-0017): `dlq retry` and `dlq purge` take `--queue` (required for
  bulk), optional `--kind`, a bounded `--limit`, and `--dry-run` to preview the
  affected count and an id sample before acting. The single-ID `retry JOB_ID` and
  the age-only `purge --older-than` paths are unchanged. The bulk API
  (`preview_dead_letters`, `redrive_dead_letters`, `purge_dead_letters_filtered`)
  is shared by the in-memory and PostgreSQL backends.

Partial / tracked in Issues:

- S6 producer + edge: the `producer` feature (enqueue-only, no worker runtime) exists
  and is exemplified by `examples/workers-producer-edge` (RFC-0012 §17.4). The
  dedicated `ag-edge` wiring is deferred until a concrete consumer exists (#112).
- S7 `ag-mail` migration: M0-M2 done (`workers`/`workers-postgres` features in
  `ag-mail`, `MailDeliveryHandler`, `WorkersMailQueue`; the duplicated
  `queue-persistent` queue is `#[deprecated]`). M3 parity verification needs a live
  database (#109); M4 removal follows it (#103).

Verification: unit, integration, property and runtime tests on the memory backend run
in CI; PostgreSQL integration tests exist but are `#[ignore]` and need `DATABASE_URL`
(#108). Criterion benchmarks live in `benches/` (see `benches/README.md`) and the
`fuzz_workers_payload` target fuzzes the payload-decode boundary in the CI fuzz-smoke
job. The workspace coverage gate is green at >= 80%.

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

## Examples

Runnable examples live in `examples/` (RFC-0012 §38):

- `workers-basic` — in-memory backend, static worker pool.
- `workers-scheduled` — interval jobs via the singleton scheduler (§21).
- `workers-producer-edge` — producer-only enqueue for edge/serverless, consumed by a
  separate fleet (§17.4).
- `workers-mail-integration` — routing `ag-mail` delivery through `ag-workers` (S7/M2).
- `workers-postgres` — durable backend with transactional `enqueue_in_tx` (§13.3);
  needs `DATABASE_URL`.

```bash
cargo run -p workers-scheduled
cargo run -p workers-producer-edge
cargo run -p workers-mail-integration
DATABASE_URL=postgres://... cargo run -p workers-postgres
```
