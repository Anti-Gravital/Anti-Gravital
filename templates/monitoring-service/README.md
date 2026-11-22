# monitoring-service

A metrics and health endpoint scaffold aligned with
[ag-observe](../../../crates/ag-observe). The default build is fully offline:
it installs a tracing subscriber and serves a Prometheus text endpoint backed
by atomic request counters. The opt-in `ag-observe` feature swaps in
ag-observe's `init`, `record_request` and `metrics_handler`. Both paths expose
the same `/health`, `/ready` and `/metrics` routes.

## Files

```
Cargo.toml      package manifest (ag-observe pinned by rev, optional)
config.toml     server and observability settings
src/main.rs     health/ready routes, metrics route and middleware
template.toml   catalog metadata
```

## Run

```sh
ag new mymonitor --template monitoring-service
cd mymonitor
ag dev
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/ready
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/metrics
```

Without the CLI, replace `{{name}}` in the source files and run `cargo run`.
`BIND` overrides the listen address (default `127.0.0.1:8080`).

## ag-observe integration

```sh
cargo run --features ag-observe
```

The `ag-observe` feature wires the real crate from the pinned Anti-Gravital
revision. Its transitive OpenTelemetry crates are not part of the offline
cache, so this opt-in requires a networked build; the default feature set never
downloads them. This keeps `cargo check --offline` green while preserving the
documented production path.

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `BIND` | `127.0.0.1:8080` | HTTP listen address. |
| `LOG_FORMAT` | `pretty` | `pretty` or `json` logs (used by ag-observe). |
| `PROMETHEUS_PORT` | `9090` | Port used by the ag-observe recorder. |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | unset | Reserved; setting it makes ag-observe `init()` fail explicitly. |

## External requirements

None for the default scaffold. A production deployment normally points a
Prometheus scraper at `/metrics` and ships logs to a collector; neither is
needed to build or run this template. The `ag-observe` feature needs network
access on first build to fetch its OpenTelemetry dependencies.

## Limitations

- This is a scaffold, not a full observability platform: no dashboards, alerts
  or remote write are configured here.
- Default metrics are process-local counters and reset on restart. Use the
  `ag-observe` feature for the standard recorder and exporter wiring.
- OTLP export is reserved and deliberately fails fast; leave the variable
  unset until a real exporter exists upstream.
- Stability: scaffold / alpha. Pin the `ag-*` revision before production use.
