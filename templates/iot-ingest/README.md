# iot-ingest

Batched device-event ingestion with explicit backpressure, built on `axum` and
`ag-realtime`. No external service is required to build or run.

## What it demonstrates

- `POST /events` accepts a JSON array of `{device_id, ts, value}` events.
- A bounded `tokio::sync::mpsc` channel (capacity 128) is the backpressure
  boundary. The HTTP handler awaits `send().await`, so a slow consumer slows
  producers instead of growing memory without bound. Oversized requests are
  rejected with 422; a closed receiver yields 503.
- A background task batches events by count (32) or time interval (200 ms) and
  publishes a `BatchSummary` on the `iot.batch` subject via `ag-realtime`.
- `GET /stats` returns per-device counts and last-seen values;
  `GET /events/stream` is an SSE feed of batch summaries.

## Run

```sh
cargo run
curl -X POST localhost:8080/events -H 'content-type: application/json' \
  -d '[{"device_id":"sensor-1","ts":1,"value":21.5},{"device_id":"sensor-1","ts":2,"value":22.0}]'
curl localhost:8080/stats
curl -N localhost:8080/events/stream
```

`BIND` overrides the address; `RUST_LOG` controls log verbosity. Channel capacity,
batch size, flush interval and maximum batch length are constants in `src/main.rs`
and mirrored in `config.toml`.

## External services

- Required to build or run: none. The in-memory channel and bus are sufficient.
- Optional, production only:
  - NATS (`nats-external` feature, `NATS_MODE=external`) to fan batch summaries
    out across instances.
  - Redis Streams or NATS JetStream for a durable, shared ingestion queue when a
    single process is not enough. Both are deployment choices, never build
    requirements.

## Limitations

- The channel and stats are process-local; a restart loses buffered events and
  aggregates. There is no persistence or acknowledgement to producers.
- Backpressure is in-memory only. A sustained overload returns 503 rather than
  durably spooling events.
- Event payloads are not authenticated or schema-validated beyond JSON shape;
  add validation and `ag-auth` before public use.
- Floating-point values are accepted as-is; no range checks are applied.
