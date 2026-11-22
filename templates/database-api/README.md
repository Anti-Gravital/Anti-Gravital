# {{name}} - database API

A compact CRUD REST API that shows the `ag-data` integration path: a connection
pool, embedded SQL migrations and handlers that share one state. The default
build is fully offline: it uses an in-memory store seeded with one record.

## Stack

| Concern | Choice |
| --- | --- |
| HTTP runtime and security | `ag-core` (`Shield`), axum 0.7 |
| Persistence | `ag-data` + `sqlx` 0.8, gated behind the `postgres` feature |
| Migrations | `migrations/0001_init.sql`, applied with `ag_data::run_migrations` |
| Ids | `uuid` v4 |

## Layout

```
Cargo.toml
config.toml
schema.ag                 Anti-DSL source of truth for models and endpoints
migrations/0001_init.sql  embedded by sqlx::migrate!
src/main.rs               composition root, store enum and handlers
template.toml             catalog metadata
```

## Run offline (default)

```bash
cargo run
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/records
curl -X POST http://127.0.0.1:8080/records \
  -H 'content-type: application/json' \
  -d '{"name":"widget","value":7}'
```

`GET /health` reports `{"status":"ok","backend":"memory"}`.

## Run with PostgreSQL

Requires a reachable server and `DATABASE_URL`:

```bash
export DATABASE_URL='postgresql://postgres:postgres@localhost/{{name}}'
cargo run --features postgres
```

With the feature enabled the startup sequence is: `ag_data::connect` ->
`ag_data::run_migrations` -> serve. Queries use runtime `sqlx::query` calls, so
there is no compile-time database dependency. If `DATABASE_URL` is absent the
service falls back to memory even with the feature on.

`external_requirements` in `template.toml` declares `DATABASE_URL`; a plain
`cargo check --offline` never opens a connection and never needs the variable.

## DSL

`schema.ag` describes the same model and endpoints in the canonical DSL. After
`ag generate`, the generated artifacts can replace the hand-written structs.
Edit `schema.ag` first, then regenerate; do not edit generated files by hand.

## Limitations

- No authentication on the CRUD routes; add `ag-auth` or a Shield `auth-jwt`
  layer before exposing the service publicly.
- The in-memory store is per process and loses data on restart.
- `list` has no pagination; add `LIMIT`/`OFFSET` for production datasets.
- PostgreSQL errors are surfaced as 400 responses; map them to 5xx/409 by
  class in a real deployment.
