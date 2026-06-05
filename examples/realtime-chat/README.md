# realtime-chat

Single-room real-time chat. Demonstrates `ag-realtime` with an in-process
EventBus and SSE streaming. No database, no authentication, no external
services.

## Running

```bash
cargo run -p realtime-chat
```

Open **http://localhost:3000** in two browser windows to see the messages in
real time.

## Environment variables

| Variable     | Default  | Description           |
|--------------|----------|-----------------------|
| `PORT`       | `3000`   | Server port           |
| `LOG_FORMAT` | `pretty` | `pretty` or `json`    |

## API

| Method | Path        | Description                   |
|--------|-------------|-------------------------------|
| GET    | `/`         | Chat UI (embedded HTML)       |
| GET    | `/events`   | SSE stream of messages        |
| POST   | `/messages` | Publish a message             |
| GET    | `/health`   | Health check                  |

### Publish with curl

```bash
curl -X POST http://localhost:3000/messages \
  -H "Content-Type: application/json" \
  -d '{"user":"alice","text":"hello from curl"}'
```

### Listen for events with curl

```bash
curl -N http://localhost:3000/events
```

## Crates demonstrated

- `ag-realtime`: in-process EventBus pub/sub
- `ag-observe`: structured logging
