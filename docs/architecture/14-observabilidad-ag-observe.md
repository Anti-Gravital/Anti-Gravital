# Capitulo 14. Observabilidad (ag-observe)

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 14
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [13-mobile-ag-mobile.md](./13-mobile-ag-mobile.md)
> Siguiente: [15-seguridad.md](./15-seguridad.md)

## 14. Observability (`ag-observe`)

Although already covered briefly as a standard module, observability deserves its own section because it is probably the most visible difference between a toy framework and a production framework.

### 14.1 Three pillars

`ag-observe` covers the three classic pillars: metrics, traces, and logs. The stack is OpenTelemetry as the abstraction layer, with configurable exporters.

Metrics are exposed at `/metrics` in Prometheus format by default. They include latency per endpoint (p50, p95, p99, p999), throughput, error rate per HTTP code, database pool usage, Redis pool usage, active WebSocket connections, and custom metrics registered by the application.

Traces are exported via OTLP to any compatible backend (Tempo, Jaeger, Datadog, Honeycomb, Lightstep). Each request generates a trace with spans for the Shield, the handlers, the SQL queries, the external calls, and the event emission.

Logs are structured (JSON by default) and always include the correlation ID. They are exported to stdout (standard for cloud-native environments) and optionally to backends such as Loki or Datadog.

### 14.2 Included Grafana dashboards

The repository includes pre-configured Grafana dashboards in JSON that the operator imports directly. They cover: service overview, latency and throughput per endpoint, errors and exceptions, database health, cache health, and Rust runtime metrics (memory usage, number of Tokio tasks, GC pauses — which will always be zero, but the dashboard confirms it).

### 14.3 Live inspection with tokio-console

In development mode, `tokio-console` is enabled automatically. It allows the developer to connect to the process and see in real time which tasks are running, which are blocked, where resources are being consumed. It is a tremendously useful debugging tool that exists only in Rust with Tokio.

### 14.4 Implemented scope and project scopes (RFC-0038, verified 2026-11-22)

Sections 14.1 to 14.3 describe the target design. The following is the scope actually implemented in `crates/ag-observe`:

- Metrics: `ag_requests_total`, `ag_request_duration_seconds`, `ag_db_pool_size`, `ag_db_pool_idle` and `ag_active_connections`, exposed by `/metrics`. Percentiles, error rate by code, Redis pool metrics and process/runtime gauges are **not** recorded by the crate; the corresponding dashboard panels are aspirational until implemented.
- Logs: structured `pretty` or `json` output to stdout. The `info` filter is the default. Export to Loki or Datadog and a correlation ID on every log line are **not** implemented; the OTLP layer is opt-in and requires the `otlp` Cargo feature plus an endpoint.
- Traces: OTLP/HTTP with pre-generated protobufs, off by default, no `protoc` and no mandatory collector (ADR-0019, RFC-0028).
- `tokio-console` is opt-in via the `dev-console` feature and `AG_DEV_CONSOLE`, not automatic.

Project scope: `AG_PROJECT_ID` is an optional bounded label (1-64 characters from `[A-Za-z0-9_.-]`). It is validated at startup; an invalid value returns `ObserveError::InvalidScope` without printing the value. When set, it becomes a constant `project` label on every Prometheus metric and a `project.id` attribute on the OTLP resource. It is a logical label, not a security boundary or a tenant identifier.

Component health: `HealthRegistry` holds named synchronous checks. `liveness_handler` returns 200 without evaluating dependencies; `readiness_handler` returns 200 or 503 with a JSON body containing `status`, `project` and each component result. Evaluation is on demand when the endpoint is scraped: no poller, scheduler or external service is added (ADR-0009).

---

