# scheduled-jobs

Cron-like recurring jobs with the `ag-workers` singleton scheduler and in-memory
schedule store. No external service is required.

## What it demonstrates

- Multiple independent schedules (`cleanup_sessions`, `aggregate_metrics`,
  `health_ping`) at different intervals, all enqueued into one queue.
- A closed handler registry: each schedule's kind selects its typed handler.
- Singleton scheduling: `run_scheduler` claims due ticks so each interval fires
  at most once per tick window. With the PostgreSQL backend this holds across N
  processes; the in-memory store keeps a single process correct.
- Graceful shutdown of both the scheduler task and the worker pool.

## Run

```sh
cargo run
```

The demo runs for about one second and prints a per-schedule run count. `RUST_LOG`
controls log verbosity. Intervals are defined in `src/main.rs` and mirrored in
`config.toml`.

## External services

- Required to build or run: none.
- Optional, production only: PostgreSQL with the `postgres` feature and
  `DATABASE_URL` for durable schedules and cross-process singleton guarantees.
  Opt-in, not a build requirement.

## Limitations

- v1 supports fixed `Duration` intervals, not full cron expressions (no `*/5 * * * *`
  syntax). Intervals are "cron-like" in intent: named recurring jobs with a
  misfire policy. A cron parser is a separate future enhancement requiring an RFC.
- The in-memory schedule store is process-local; schedules are lost on restart.
- Overlapping runs are not prevented beyond the singleton tick claim; handlers
  must be idempotent.
- Misfire policy is `Skip`: ticks missed while the process was down are dropped.
