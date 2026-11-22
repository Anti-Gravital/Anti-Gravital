# workers-producer-edge

Producer-only (edge / serverless) usage of `ag-workers` (RFC-0012 section 17.4).

Edge and serverless targets cannot host long-lived workers. In producer-only mode a
process links the enqueue API only (`enqueue` / `enqueue_in_tx`) and hosts no leasing
loop, scheduler, or worker runtime; a separate worker fleet consumes the jobs.

This example runs both sides against one in-memory backend so it works end to end. In
production the producer and the fleet are different processes pointing at the shared
PostgreSQL backend.

The edge side emits through `ag-edge`'s dependency-free `producer` port
(`ag-edge` does not depend on `ag-workers`, and the reverse dependency is
forbidden by RFC-0012). `src/bridge.rs` is the adapter that translates an
`EnqueueRequest` into an `ag-workers` `NewJob`. A unit test in that module
verifies that a port emission reaches `MemoryQueue` and is consumed by a
separate lease path.

No external service is required.

```bash
cargo run -p workers-producer-edge
```
