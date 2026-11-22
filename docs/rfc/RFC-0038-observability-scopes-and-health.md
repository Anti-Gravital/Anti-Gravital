# RFC-0038 - Observability scopes and component health in ag-observe

- Status: accepted
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

Add a native `HealthRegistry` with named checks and an Axum handler that aggregates them into an overall readiness response. No external service, no polling daemon: the native default stays a pull `/metrics` and `/health` endpoint, consistent with ADR-0009. A failing registered component flips aggregate readiness.

### 4.3 Process and runtime metrics

Record process/runtime gauges natively behind a separate opt-in feature only if a concrete consumer exists. Otherwise first reconcile `dashboards/runtime.json` with the series the crate actually emits. Documentation claims in architecture ch.14 are either implemented in the same PR or corrected to describe reality (K-1).

### 4.4 Stack compatibility

OpenTelemetry, Prometheus and Grafana are open-source technologies the project builds on, not component names (ADR-0011 permits this). Exporters remain opt-in adapters: `otlp` is already feature-gated (`crates/ag-observe/Cargo.toml:17-21`); nothing makes a collector mandatory.

## 5. KPI and verifiable acceptance criteria

- K1: `cargo test -p ag-observe` green with no collector and no `protoc` installed.
- K2: `cargo test -p ag-observe --features otlp` green under the RFC-0028 CI job; no collector required.
- K3: with a scope configured, rendered `/metrics` contains the `project="..."` label and an OTLP resource test asserts the attribute.
- K4: a registered failing component changes aggregate readiness in a test.
- K5: `dashboards/runtime.json` and architecture ch.14 match the series actually emitted; crate README env vars match code.
- K6: documentation (architecture ch.14, master Arquitectura Tecnica section 14, crate README, CHANGELOG) updated in the same PR (rules 19, 26).

## 6. Risks

- "Project" semantics and label cardinality are undecided: a human must define whether this is a tenant, an application or a deployment scope. No invention (rule 5).
- Deploying collectors/Grafana and accepting infrastructure cost needs human approval (ADR-0009 rule 5, rule 13).
- Any new dependency for process metrics needs maturity and security justification (rule 15) and `cargo audit`/`deny` clearance.
- Correcting aspirational documentation is a doc change with no code risk, but must not be used to hide a missing capability (rule 26).

## 7. Impact

- Scope: `ag-observe` API and dashboards.
- Public APIs: additive field on `ObserveConfig`, additive `HealthRegistry`; SemVer impact additive.
- Documentation: architecture ch.14, master technical architecture, crate README, CHANGELOG.

## 8. Rollback

The scope field and health registry are additive and default-off. Removing them restores current behavior. No collector or data migration is introduced.

## 9. Decision

- Decider: BDFL (Angel Nereira).
- Decision date: 2026-11-22.
- Result: accepted. Comment period waived by BDFL decision.
- Open design inputs retained: the concrete definition of the "project" scope and any collector deployment follow the constraints agreed here (ADR-0009); no external service becomes mandatory.
