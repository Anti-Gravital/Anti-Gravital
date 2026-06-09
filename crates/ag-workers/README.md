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

Implemented so far (S1 — foundations):

- Job identity and value types (`JobId`, `WorkerId`, `QueueName`, `JobKind`,
  `TenantId`).
- Job model: `JobEnvelope`, `NewJob`, `JobStatus` with a normative transition table,
  and `JobPriority`.
- Versioned payload encoding (`rmp-serde`) with a `migrate` hook; decode/version
  failures are reported as `InvalidPayload` (never panic, never silently drop).
- Handler contract: `JobHandler`, `JobContext` (cooperative cancellation via
  `tokio_util::sync::CancellationToken`), `JobOutcome`, `WorkerError` taxonomy.
- Closed handler registry with type-erased dispatch and kind resolution.

The runtime (queue backends, dispatch loop, poison guard, scheduling, CLI, DSL) lands
in the following stages.

## Features

- `memory` (default): in-memory backend.
- `metrics` (default): emit `ag_workers_*` metrics through the `metrics` crate.
- `postgres`: durable PostgreSQL backend via `ag-data`.
- `producer`: enqueue-only mode for edge/serverless targets.
- `scheduler`: delayed and fixed-interval scheduling.
- `test-utils`: helpers (NullBackend, deterministic clock, fixtures) for downstream
  crate tests.
