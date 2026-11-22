# NATS event stack

Local NATS with JetStream enabled for `ag-realtime` and `ag-workers` event
flows. The in-memory native modes remain the default in generated projects.

## Ports

| Host port | Service | Purpose |
| --- | --- | --- |
| 4222 | nats | Client protocol |
| 8222 | nats | Monitoring HTTP (bound to 127.0.0.1) |

Override with `NATS_PORT` and `NATS_HTTP_PORT` in `.env`.

## Run

```sh
cd deploy/stacks/nats-events
cp .env.example .env
docker compose up -d
```

## Point a generated project at the stack

```sh
export NATS_URL='nats://127.0.0.1:4222'
export NATS_JETSTREAM='true'
export NATS_JS_STREAM='AG_EVENTS'   # any stream name your project declares
```

JetStream state persists in the named volume `natsdata`. The container
healthcheck queries `http://localhost:8222/healthz`.

## Limits and security

Authentication and TLS are off by default. The monitoring port is bound to
localhost and is unauthenticated; check it with
`curl http://127.0.0.1:8222/healthz`. For anything beyond local development,
configure NKeys/credentials and TLS before exposing port 4222.
