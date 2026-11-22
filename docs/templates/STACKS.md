# Local stack decision table

This table helps choose a local support stack for a generated template. Stacks
are under `deploy/stacks/`; each is an independent Compose project with a
`.env.example`, named volumes and healthchecks. Use only the stacks a use case
needs. Native or in-memory defaults remain available in every template.

For template discovery use the template catalog documentation under
`docs/templates/`; for the stack layout, ports and run commands see
`deploy/stacks/README.md`.

## Decision table

| Use case | Stack | Template family | External dependency | Host ports |
| --- | --- | --- | --- | --- |
| Durable SQL persistence for an API | `postgres-api` (`postgres-api`) | api (`database-api`, `multi-tenant-saas`, `ecommerce-lite`), fullstack | PostgreSQL 16 | 5432 (API 8080 with profile `app`) |
| Cache with an external backend instead of the in-process cache | `redis-cache` | api, realtime, infrastructure | Redis 8 | 6379 |
| Typed events, jobs or a JetStream stream | `nats-events` | realtime (`websocket-service`, `worker-service`, `scheduled-jobs`, `iot-ingest`) | NATS 2 with JetStream | 4222, 8222 (local monitoring) |
| Verify magic links, recovery or SMTP delivery without a real inbox | `mail-sink` | api (`auth-service`), infrastructure (`mail-service`) | Local SMTP capture | 1025 (SMTP), 8025 (UI) |
| Scrape metrics and inspect them locally | `monitoring` | infrastructure (`monitoring-service`), any API exposing Prometheus metrics | Prometheus + Grafana | 9090, 3000 |
| Upload or serve objects through an S3 API | `object-storage` | api (`file-upload-service`), infrastructure (`object-storage-api`) | S3-compatible endpoint | 9090 |

"External dependency" names the service the stack provides. It is not required
by the template for scaffolding or offline `cargo check`; it is needed only for
the corresponding runtime integration.

## Choosing rules

1. Start with no stack. Add one only when a template's runtime needs the
   service, and point it at the stack through the documented environment
   variable (`DATABASE_URL`, `REDIS_URL`, `NATS_URL`, `AG_SMTP_HOST`,
   `STORAGE_ENDPOINT`, `PROMETHEUS_PORT`).
2. Combine stacks by running each from its own directory. Host ports are
   independent per Compose project; change them in `.env` if two stacks need
   the same port.
3. Never put real credentials in a stack file. Copy `.env.example` to `.env`;
   `.env` is gitignored.
4. Do not treat a local stack as production evidence. `mail-sink` and
   `object-storage` are testing endpoints without authentication; `nats-events`
   disables auth by default.

## Honest limitations

- These stacks were parse-validated with `docker compose config`. Services were
  not started during verification because the Docker daemon was unavailable;
  see `deploy/stacks/VERIFICATION.md`.
- `ag deploy` is not available in the alpha CLI; these files are consumed
  directly with `docker compose`, not through the CLI.
- Only the `fullstack` template exposes an HTTP `/health` route in the current
  templates, so the optional API service in `postgres-api` has no in-container
  health probe.
