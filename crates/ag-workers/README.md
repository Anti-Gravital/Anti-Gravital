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

Scheduling, dynamic/CPU-bound pools, the DSL `worker` declaration, the CLI, producer
mode and the `ag-mail` migration land in the following stages (S4-S7).

## Features

- `memory` (default): in-memory backend.
- `metrics` (default): emit `ag_workers_*` metrics through the `metrics` crate.
- `postgres`: durable PostgreSQL backend via `ag-data`.
- `producer`: enqueue-only mode for edge/serverless targets.
- `scheduler`: delayed and fixed-interval scheduling.
- `test-utils`: helpers (NullBackend, deterministic clock, fixtures) for downstream
  crate tests.
