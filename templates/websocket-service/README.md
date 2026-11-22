# websocket-service

A small WebSocket service with an echo endpoint, a broadcast endpoint, and a
per-connection rate limiter. Both socket routes use the native `ag-realtime`
in-process bus, so no external broker is needed.

## What it demonstrates

- `GET /ws/echo`: each text frame is returned prefixed with `echo:`.
- `GET /ws/broadcast`: text frames are published on `ws.broadcast` and relayed to
  every connected broadcast client through `ag-realtime`.
- Per-connection token bucket: a 20-token bucket refilling at 10 tokens/second.
  Excess inbound messages receive a `rate limit exceeded` text frame instead of
  being processed.
- Graceful handling of lagged broadcast receivers (frames are skipped, the socket
  stays open).

## Run

```sh
cargo run
# Connect with any WebSocket client, e.g. `websocat`:
websocat ws://127.0.0.1:8080/ws/echo
websocat ws://127.0.0.1:8080/ws/broadcast
```

`BIND` overrides the address; `RUST_LOG` controls log verbosity. Capacity and refill
rate are defined in `src/main.rs` and mirrored in `config.toml`.

## External services

- Required to build or run: none.
- Optional, production only: NATS with the `nats-external` feature and
  `NATS_MODE=external` to broadcast across multiple instances. This is a deployment
  choice, not a build requirement.

## Limitations

- Rate-limit state is per connection and in memory; it is not shared across
  replicas. Use a shared limiter (for example Redis) for fleet-wide quotas.
- The broadcast bus is in-process only; messages are not persisted.
- Only text frames are handled; binary frames are ignored.
- No authentication or origin checking is applied; add those before public use.
