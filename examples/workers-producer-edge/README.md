# workers-producer-edge

Producer-only (edge / serverless) usage of `ag-workers` (RFC-0012 section 17.4).

Edge and serverless targets cannot host long-lived workers. In producer-only mode a
process links the enqueue API only (`enqueue` / `enqueue_in_tx`) and hosts no leasing
loop, scheduler, or worker runtime; a separate worker fleet consumes the jobs.

This example runs both sides against one in-memory backend so it works end to end. In
production the producer and the fleet are different processes pointing at the shared
PostgreSQL backend.

No external service is required.

```bash
cargo run -p workers-producer-edge
```
