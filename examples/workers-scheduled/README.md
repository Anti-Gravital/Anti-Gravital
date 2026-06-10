# workers-scheduled

Interval (scheduled) jobs with `ag-workers` (RFC-0012 section 21).

A `Schedule` is registered to enqueue a `cleanup_expired_sessions` job every 100ms.
The scheduler loop (`run_scheduler`) claims each due tick as a singleton and enqueues
it; a one-worker pool executes the jobs. With the in-memory store a single process is
trivially singleton; with the PostgreSQL store the same interval fires once across N
processes (`claim_due` uses `FOR UPDATE SKIP LOCKED`).

No external service is required.

```bash
cargo run -p workers-scheduled
```

Expected output: a handful of `[tick N] cleanup ran` lines, then a final count.
