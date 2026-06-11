# workers-postgres

Durable `ag-workers` backend with transactional enqueue (RFC-0012 sections 13.2-13.3).

Demonstrates the PostgreSQL backend and `enqueue_in_tx`: the job insert participates in
the caller's `ag_data::AgTx` transaction, so a domain write and its follow-up job commit
atomically (the transactional-outbox property without a separate outbox table). A worker
pool then leases and runs the durably-enqueued job.

Requires a reachable PostgreSQL instance via `DATABASE_URL`; without it the example prints
how to run and exits cleanly.

```bash
DATABASE_URL=postgres://user:pass@localhost/ag_workers_example \
  cargo run -p workers-postgres
```
