# workers-basic

Smallest useful `ag-workers` setup on the in-memory backend: a typed payload, a
`JobHandler`, a registry, a static worker pool, enqueueing jobs and a graceful shutdown.
No external service is required (native-first, ADR-0009).

## Run

```bash
cargo run -p workers-basic
```

Expected output (order may vary across the two workers):

```text
[attempt 1] sending welcome to user 1 <user1@example.com>
[attempt 1] sending welcome to user 2 <user2@example.com>
...
done: 5 jobs sent, 0 remaining in queue
```

## What it shows

- Defining a typed payload (`SendWelcome`) implementing `JobPayload`.
- Implementing `JobHandler` for a job kind (`send_welcome`).
- Building a closed `WorkerRegistry`.
- The in-memory `MemoryQueue` backend (default; durable PostgreSQL is opt-in behind the
  `postgres` feature).
- A static `WorkerPool` in embedded mode (workers run inside the app process).
- Enqueueing jobs and observing them execute, then a graceful shutdown.

## Producer-only mode

A process that only needs to *enqueue* (for example an `ag-edge` or serverless target
that cannot host long-lived workers) constructs the backend and calls `enqueue` /
`enqueue_in_tx` without ever starting a `WorkerPool`. A separate standalone or
distributed worker fleet (sharing the PostgreSQL backend) consumes the jobs. See
`docs/modules/ag-workers/README.md` and RFC-0012 §17.4.

## Decision

`RFC-0012` / `ADR-0013`. Module docs: `docs/modules/ag-workers/README.md`.
