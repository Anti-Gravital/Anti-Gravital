# Local service stacks

Prebuilt Compose stacks that developers can combine with generated templates to
run real services locally. They are opt-in: the default generated project keeps
running with `cargo run` / `ag dev` and native or in-memory backends (ADR-0009,
rule 2). These stacks follow rule 12: they integrate dominant tools instead of
replacing them.

This directory is not a `docker compose` project on its own. Each subdirectory
is an independent project with its own `compose.yml` and `.env.example`.

## Available stacks

| Directory | Services | Host ports | Primary template families |
| --- | --- | --- | --- |
| `postgres-api/` | PostgreSQL + optional generated API | 5432, 8080 | api, fullstack |
| `redis-cache/` | Redis | 6379 | api, realtime, infrastructure |
| `nats-events/` | NATS with JetStream | 4222, 8222 | realtime, infrastructure |
| `mail-sink/` | Local SMTP capture and web UI | 1025, 8025 | api (auth, mail), infrastructure |
| `monitoring/` | Prometheus + Grafana | 9090, 3000 | infrastructure, any HTTP API |
| `object-storage/` | S3-compatible endpoint | 9090 | api (file upload), infrastructure |

The decision table in `docs/templates/STACKS.md` maps use cases to stacks,
template families, external dependencies and ports.

## General usage

```sh
cd deploy/stacks/<stack>
cp .env.example .env     # .env is gitignored
docker compose up -d
docker compose ps        # confirm health status
docker compose down      # named volumes are kept
docker compose down -v   # removes named volumes and their data
```

Create a `.env` from `.env.example` before use, but `docker compose config`
also works without it because every value has an interpolation default.
Ports can overlap between stacks because each is a separate Compose project;
run only the stacks you need, or change the host port in `.env` when combining.

## Secrets and configuration policy

- No real credentials are committed. `.env.example` contains local-only
  placeholders; `.env` is gitignored.
- Every external integration is opt-in through environment variables, and the
  generated project keeps a native/in-memory default.
- Image tags are pinned. Healthchecks and named volumes are defined per service.
- The monitoring port of NATS, the mail UI and the S3 endpoint have no
  authentication. Keep them on localhost.

## Verification

`deploy/stacks/VERIFICATION.md` records real YAML-parse and
`docker compose config` output. No service was started during that check:
the Docker daemon was unavailable in the verification environment.
