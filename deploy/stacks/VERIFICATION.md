# Stack verification record

Evidence for the files under `deploy/stacks/`. English canonical (ADR-0008).

## Environment

| Field | Value |
| --- | --- |
| Date (UTC) | 2026-11-22T04:52:24Z |
| Host | Linux 7.0.0-44-generic x86_64 |
| Python | 3.14.4 |
| PyYAML | 6.0.3 |
| Docker Compose CLI | v5.5.1 |
| Docker daemon | DOWN (`docker info` failed) |
| Branch | community7/stacks |
| Base commit | e9347ab |

The Compose CLI parses files without a daemon, so config validation is real.
No image was pulled and no container was started. Service-start and HTTP
probe checks are recorded as SKIP, never as PASS.

## 1. YAML parse (real parser)

Command pattern:

```sh
python3 -c "import yaml,sys; d=yaml.safe_load(open(sys.argv[1])); print('PASS', sys.argv[1], 'top-keys:', ','.join(sorted(d.keys())))" FILE
```

| File | Result | Top-level keys |
| --- | --- | --- |
| `deploy/stacks/postgres-api/compose.yml` | PASS | services, volumes |
| `deploy/stacks/redis-cache/compose.yml` | PASS | services, volumes |
| `deploy/stacks/nats-events/compose.yml` | PASS | services, volumes |
| `deploy/stacks/mail-sink/compose.yml` | PASS | services, volumes |
| `deploy/stacks/monitoring/compose.yml` | PASS | services, volumes |
| `deploy/stacks/object-storage/compose.yml` | PASS | services, volumes |
| `deploy/stacks/monitoring/prometheus.yml` | PASS | global, scrape_configs |

## 2. Compose config validation (real CLI)

For each stack directory: `docker compose -f compose.yml config`. The
`postgres-api` stack was additionally validated with `--profile app`, because
plain `config` omits profile-gated services by design.

| Stack | Command | rc | stderr | Services in resolved config |
| --- | --- | --- | --- | --- |
| postgres-api | `docker compose -f compose.yml config` | 0 | 0 bytes | postgres |
| postgres-api | `docker compose --profile app -f compose.yml config` | 0 | 0 bytes | postgres, api |
| redis-cache | `docker compose -f compose.yml config` | 0 | 0 bytes | redis |
| nats-events | `docker compose -f compose.yml config` | 0 | 0 bytes | nats |
| mail-sink | `docker compose -f compose.yml config` | 0 | 0 bytes | mailpit |
| monitoring | `docker compose -f compose.yml config` | 0 | 0 bytes | prometheus, grafana |
| object-storage | `docker compose -f compose.yml config` | 0 | 0 bytes | s3mock |

Every resolved config contains the expected named volumes, published ports and
healthchecks. Resolved fields are summarized below; the appendix shows the
postgres-api output and a field-level summary of the remaining stacks.

## 3. Checks that were not performed

| Check | Status | Reason |
| --- | --- | --- |
| `docker compose up -d` for any stack | SKIP | Docker daemon DOWN; no service can be started |
| `docker compose ps` health status | SKIP | requires running containers |
| Image pull / digest verification | SKIP | requires daemon and network pulls; tags were checked against the Docker Hub tags API only |
| HTTP probes (`/health`, `/metrics`, SMTP send) | SKIP | no running services |
| Resource limits / `read_only` hardening | SKIP | not claimed; local-only stacks, documented in stack READMEs |

No statement in this repository claims that any of these services were
started, that images were pulled, or that endpoints answered.

## 4. Image tags checked

Tags were confirmed to exist through the Docker Hub tags API on 2026-11-22:

- `postgres:16.16-alpine`
- `redis:8.6.8-alpine`
- `nats:2.15.3-alpine`
- `axllent/mailpit:v1.27.11`
- `prom/prometheus:v3.16.0-busybox`
- `grafana/grafana:13.2`
- `adobe/s3mock:5.2.3`

Healthcheck commands were taken from upstream documentation: `pg_isready`
(PostgreSQL), `redis-cli ping` (Redis), `GET /healthz` on the NATS monitoring
port, `mailpit readyz` (image HEALTHCHECK), `GET /-/healthy` (Prometheus),
`GET /api/health` (Grafana) and `GET /favicon.ico` (S3Mock).

## Appendix: normalized `docker compose config` output

### postgres-api (`--profile app`)

```yaml
name: postgres-api
services:
  api:
    profiles: [app]
    build:
      context: /home/angel-nereira/Github-local-angelnereira/worktrees7/stacks/deploy/stacks/postgres-api
      dockerfile: Dockerfile
    depends_on:
      postgres: {condition: service_healthy, required: true}
    environment:
      BIND: 0.0.0.0:8080
      DATABASE_URL: postgres://ag:ag-local-only@postgres:5432/ag
      PORT: "8080"
    image: ag-api:local
    ports: [{mode: ingress, target: 8080, published: "8080", protocol: tcp}]
  postgres:
    environment:
      POSTGRES_DB: ag
      POSTGRES_INITDB_ARGS: --data-checksums
      POSTGRES_PASSWORD: ag-local-only
      POSTGRES_USER: ag
    healthcheck:
      test: [CMD-SHELL, "pg_isready -U ag -d ag"]
      timeout: 5s
      interval: 5s
      retries: 10
      start_period: 10s
    image: postgres:16.16-alpine
    ports: [{mode: ingress, target: 5432, published: "5432", protocol: tcp}]
    restart: unless-stopped
    volumes: [{type: volume, source: pgdata, target: /var/lib/postgresql/data}]
volumes:
  pgdata: {name: postgres-api_pgdata}
```

The appendix above is abbreviated to the fields relevant to this record
(network keys and empty `volume: {}` markers removed). Re-run the exact
command from section 2 to obtain the unmodified CLI output.

### Other stacks

The same validation was applied to `redis-cache`, `nats-events`, `mail-sink`,
`monitoring` and `object-storage`. Each resolved config contained:

| Stack | Image | Published ports | Named volume(s) | Healthcheck |
| --- | --- | --- | --- | --- |
| redis-cache | `redis:8.6.8-alpine` | 6379 | `redisdata` -> `/data` | `redis-cli ping` |
| nats-events | `nats:2.15.3-alpine` | 4222, 8222 (127.0.0.1) | `natsdata` -> `/data` | `wget --spider http://localhost:8222/healthz` |
| mail-sink | `axllent/mailpit:v1.27.11` | 1025, 8025 | `maildata` -> `/data` | `/mailpit readyz` |
| monitoring | `prom/prometheus:v3.16.0-busybox`, `grafana/grafana:13.2` | 9090, 3000 | `promdata`, `grafanadata` | `/-/healthy`, `/api/health` |
| object-storage | `adobe/s3mock:5.2.3` | 9090 | `s3data` -> `/s3mockroot` | `/favicon.ico` |

The `monitoring` config also resolves the bind mount of `prometheus.yml` as
read-only and `extra_hosts` as `host.docker.internal=host-gateway`.
