# worker-service

A background job service built on `ag-workers` with the in-memory queue backend.
It shows typed payloads, a closed handler registry, retries, worker pools, and a
dead-letter queue (DLQ), all without an external service.

## What it demonstrates

- Typed jobs: `SendEmail` and `GenerateReport` implement `JobPayload` with an
  explicit payload version.
- Closed registry: handlers are registered by kind; unknown kinds are rejected.
- Retries: `GenerateReport` returns a retriable error on its first attempt, then
  completes through the configured backoff policy.
- Dead-letter queue: a poison `SendEmail` is sent to the DLQ with a `Requested`
  reason via `JobOutcome::DeadLetter`; preserved entries are inspected at the end.
- Graceful shutdown: a `Shutdown` token stops the pool and `join_with_timeout`
  bounds the drain.

## Run

```sh
cargo run
```

Expected output includes three sent emails, a report that succeeds on attempt 2,
and one DLQ entry. `RUST_LOG` controls log verbosity.

## External services

- Required to build or run: none. The memory backend is the default.
- Optional, production only: PostgreSQL with the `postgres` feature and
  `DATABASE_URL` for durable jobs and a persistent DLQ. Opt-in, not a build
  requirement.

## Limitations

- The memory backend is process-local: queued jobs and DLQ entries are lost on
  restart and are not shared between replicas.
- The retry policy in this example is intentionally fast for demonstration; use
  realistic delays in production.
- This is a single-process pool; use the PostgreSQL backend for a distributed
  worker fleet.
