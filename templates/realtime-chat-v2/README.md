# realtime-chat-v2

Multi-room chat with per-room presence and server-sent event (SSE) fanout, built on
`ag-realtime`. This is a template for `ag new <name> --template realtime-chat-v2` once
the catalog registry wires it; it is also usable standalone.

## What it demonstrates

- Rooms: every message is published on the subject `chat.room.<room>.message`, so
  rooms are isolated without separate servers.
- Presence: an in-process `HashMap<room, Set<user>>` tracks who is in each room. Join
  and leave emit `chat.room.<room>.presence` events.
- Typed events: messages are serialized with `serde` via `broadcast_json`.
- Native realtime: the default `ag-realtime` backend is an in-process broadcast bus,
  so no broker is needed locally.

## Run

```sh
# After scaffolding (the {{name}} placeholder is replaced by the project name):
cargo run

# In another terminal, join and watch a room:
curl -X POST localhost:8080/rooms/general/join \
  -H 'content-type: application/json' -d '{"user":"alice"}'
curl -N localhost:8080/rooms/general/events
curl -X POST localhost:8080/rooms/general/messages \
  -H 'content-type: application/json' -d '{"user":"alice","text":"hello"}'
curl localhost:8080/rooms/general/presence
```

Set `BIND` to override the listen address (`127.0.0.1:8080` by default) and `RUST_LOG`
to change log verbosity.

## External services

- Required to build or run: none. The in-process bus is the default.
- Optional, production only: NATS. To fan messages out across multiple instances,
  enable the `nats-external` feature on `ag-realtime`, set `NATS_MODE=external` and
  provide `NATS_URL`. This is a deployment choice, not a build requirement.

## Limitations

- Presence is process-local: it is not shared between replicas. Use a shared store
  (for example Redis) and a pub/sub backplane for multi-instance presence.
- The in-process bus is a `tokio::sync::broadcast` channel: slow SSE clients that lag
  drop old messages rather than blocking publishers.
- Messages are not persisted; this template is ephemeral by design.
- Inputs are length-checked but no authentication is applied; add `ag-auth` before
  exposing this publicly.
