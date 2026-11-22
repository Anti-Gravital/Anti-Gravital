# Digest de conocimiento: oss-observability-specialist

- **Observability scopes and health** — `docs/rfc/RFC-0038-observability-scopes-and-health.md` (sha256 0cc89fc53ca6)
- **OTLP transport ADR** — `docs/adr/0019-ag-observe-otlp-transport.md` (sha256 378692b27381)

## RFC-0038-observability-scopes-and-health.md
# RFC-0038 - Observability scopes and component health in ag-observe

- Status: draft (pending BDFL approval)
- Draft date: 2026-11-22
- Target phase: additive, no phase change
- Affected modules/crates: `ag-observe` only (plus documentation)
- Predecessor: ADR-0019 (OTLP transport), RFC-0028 (OTLP feature CI verification)
- Comment period: open

## 1. Motivation

Users need to understand what a project is doing underneath: performance, events, logs, traces, process load, system state and whether each component is healthy. `ag-observe` already provides native logging, Prometheus metrics, Grafana dashboards and opt-in OTLP traces. What is missing is a project dimension and real health state, so a user does not work blind.

## 2. Problem (verified on 2026-11-22)

1. `ObserveConfig` has only `otlp_endpoint`, `prometheus_port`, `log_format` and `dev_console` (`crates/ag-observe/src/config.rs:16-34,47-68`). There is no project/tenant field.
2. The OTLP resource carries only `service.name` (`crates/ag-observe/src/layer.rs:66-69`); metrics are labelled `method`, `endpoint`, `status` (`crates/ag-observe/src/metrics.rs:31-45`). There is no way to separate projects in a shared collector or dashboard.
3. `init()` installs a global subscriber once (`layer.rs:104-158`). It is a single-shot process-level API, not a per-project factory.
4. Health/ready exist only as constant JSON in `templates/monitoring-service/src/main.rs:45-51,104-108`, not in the crate. No component can register a real check.
5. `dashboards/runtime.json` presupposes process/runtime series that the crate never records.
6. `docs/architecture/14-observabilidad-ag-observe.md:14-24` promises p50/p95/p99 percentiles, Redis pool metrics, Shield/SQL spans, Loki export and correlation IDs on every log that the implementation does not deliver. The crate README documents `LOG_LEVEL` and `LOG_FORMAT=text` (`README.md:85-87`) while code uses an `info` EnvFilter default (`layer.rs:114`) and `Pretty` default (`config.rs:41`).
7. The word "project" is not defined in the repository. Multi-tenant concepts exist elsewhere (`ag-edge` tenant bindings, the `multi-tenant-saas` template) but are not wired to observability.

## 3. Alternatives considered

- **A. Deploy a full collector, Grafana and Prometheus stack as a default dependency.** Rejected: violates ADR-0009 rule 2 (an external service must never be a requirement) and rule 13 (operational simplicity). Kept as opt-in adapters.
- **B. Create a separate `ag-observability` crate.** Rejected: one consumer, no second use case; rules 5/14/28 and the same reasoning as RFC-0018 for `ag-tui`.
- **C. Extend `ag-observe` with an optional scope and a native health registry (chosen).** Native defaults stay collector-free; exporters remain feature-gated.

## 4. Proposed design

### 4.1 Scope

Add an optional project/scope value to `ObserveConfig` (proposed environment variable `AG_PROJECT_ID`, name subject to approval). When set:
- it is injected as an OTLP resource attribute in `build_otlp_layer` (`layer.rs:36-81`);
- it becomes a constant Prometheus label at recorder installation.

The value must be bounded and validated. Unbounded identifiers are rejected at startup with a clear message that never prints a secret.

### 4.2 Component health

## 0019-ag-observe-otlp-transport.md
# ADR-0019 - ag-observe OTLP transport without protoc

- Status: accepted
- Date: 2026-11-21
- Decider: project maintainer (issue #149 resolution)

## Context

`ag-observe` kept its OTLP exporter deferred. Two `TECH-DEBT` markers claimed
that `opentelemetry-otlp` needs `protoc` at build time through `grpc-tonic`,
that this breaks `ci/build windows-x64`, and that `opentelemetry_sdk` needs an
active Tokio runtime. Both markers described an older release, not the
versions actually pinned in the workspace (`opentelemetry-otlp` 0.33.x,
`opentelemetry-proto` 0.33.x, `opentelemetry_sdk` 0.33.x).

Inspection of those crate sources shows:

1. `opentelemetry-otlp` 0.33.x does not enable `grpc-tonic` by default. Its
   defaults are `http-proto`, `reqwest-blocking-client`, `trace`, `metrics`,
   `logs` and `internal-logs`.
2. `opentelemetry-proto` 0.33.x has no `build.rs`. The generated Rust modules
   are committed to the crate and `tonic-prost-build` is a dev-dependency only.
   No feature runs a protobuf compiler at build time.
3. `opentelemetry_sdk` 0.33.x uses a thread-based `BatchSpanProcessor` by
   default. Its own documentation states that `reqwest-blocking-client` works
   with a regular `main` as well as with `#[tokio::main]`; only `grpc-tonic`
   requires the provider to be created inside a Tokio runtime.

The `protoc` and runtime constraints were therefore real for the historical
gRPC path but are not present in the pinned versions. Keeping the markers was
inaccurate technical debt.

## Decision

1. Re-integrate the OTLP exporter behind an opt-in Cargo feature named `otlp`,
   disabled by default.
2. Use the HTTP/protobuf transport: `opentelemetry-otlp` with
   `default-features = false` and `features = ["http-proto",
   "reqwest-blocking-client"]`. The `grpc-tonic` feature is never enabled.
3. `init()` adds a `tracing-opentelemetry` layer backed by an

