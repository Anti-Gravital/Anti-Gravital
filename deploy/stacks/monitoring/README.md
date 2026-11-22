# Local metrics stack

Prometheus plus Grafana for generated projects that expose Prometheus-format
metrics on `PROMETHEUS_PORT`. This stack scrapes metrics; it does not ingest
OTLP directly (an OTLP collector would be a separate component and is not
included here).

## Ports

| Host port | Service | Purpose |
| --- | --- | --- |
| 9090 | prometheus | Prometheus UI and query API |
| 3000 | grafana | Grafana UI |

Override with `PROMETHEUS_PORT` and `GRAFANA_PORT` in `.env`.

## Run

```sh
cd deploy/stacks/monitoring
cp .env.example .env
docker compose up -d
```

Prometheus scrapes itself and `host.docker.internal:8080` by default. Edit
`prometheus.yml` to change the `ag-api` target to your generated project's
metrics port. The target stays red until a scrappable endpoint exists.

## Point a generated project at the stack

```sh
export PROMETHEUS_PORT=8080   # port the generated app exposes metrics on
export SERVICE_NAME=my-api    # used by ag-observe metric labels
```

`ag-observe` reads `PROMETHEUS_PORT`, `SERVICE_NAME`, `LOG_FORMAT` and
`OTEL_EXPORTER_OTLP_ENDPOINT`. Metrics must be exposed in Prometheus text
format for this stack to scrape them. Grafana admin credentials come from
`GRAFANA_ADMIN_USER` / `GRAFANA_ADMIN_PASSWORD` in `.env`.

## Health and persistence

Prometheus checks `/-/healthy`, Grafana checks `/api/health`. Metrics and
dashboards persist in the named volumes `promdata` and `grafanadata`.

## Limits and security

The admin values in `.env.example` are local placeholders, not secrets. Change
them before exposing port 3000 beyond localhost. The Prometheus config file is
mounted read-only; reloading it requires a container restart.
