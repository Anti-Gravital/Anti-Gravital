# PostgreSQL + API stack

Local PostgreSQL for the API templates (`database-api`, `multi-tenant-saas`,
`ecommerce-lite`, `fullstack`) plus an optional service that builds a generated
project. Native `cargo run` / `ag dev` with an in-memory fallback stays the
default; this stack is opt-in.

## Ports

| Host port | Service | Purpose |
| --- | --- | --- |
| 5432 | postgres | PostgreSQL wire protocol |
| 8080 | api (profile `app`) | Generated HTTP API |

Host ports are overridable with `POSTGRES_PORT` and `API_PORT` in `.env`.

## Run

```sh
cd deploy/stacks/postgres-api
cp .env.example .env          # adjust local values if needed
docker compose up -d          # database only
docker compose --profile app up --build -d   # database + generated API
```

The `api` service is behind the `app` profile because it needs a generated
project. Set `API_BUILD_CONTEXT` to that project's directory and
`API_DOCKERFILE` to its Dockerfile (default `Dockerfile`) in `.env`.

## Point a generated project at the stack

For a project running on the host, export the same URL the in-container service
uses (replace the host with `127.0.0.1`):

```sh
export DATABASE_URL='postgres://ag:ag-local-only@127.0.0.1:5432/ag'
```

If the API runs inside this compose stack, `DATABASE_URL` is injected
automatically with `postgres` as the host and waits for the database healthcheck.

## Health

- `postgres` has a `pg_isready` healthcheck and a named volume `pgdata`.
- The generated `api` intentionally has no in-container healthcheck: templates
  may build scratch images with no shell or curl. From the host, only
  `fullstack` exposes `GET /health`; other templates need a client request to
  their own route or a TCP check.

## Limits and security

The password in `.env.example` is a local placeholder, not a secret. Override
it before exposing port 5432 beyond localhost. No credentials are committed.
