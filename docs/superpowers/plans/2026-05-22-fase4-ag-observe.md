# ag-observe — Plan de Implementacion

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implementar la capa transversal de observabilidad: tracing estructurado, exportacion OpenTelemetry, metricas Prometheus y dashboards Grafana pre-configurados.

**Architecture:** Un unico `init()` configura el subscriber global de `tracing` con tres layers (fmt, OTLP, metrics). La API es minima; el resto del ecosistema solo usa `tracing::info!()` etc. Rama: `fase-4/ag-observe`.

**Tech Stack:** `tracing`, `opentelemetry` + `opentelemetry-otlp` + `opentelemetry_sdk`, `tracing-opentelemetry`, `metrics` + `metrics-exporter-prometheus`, `console-subscriber` (feature `dev-console`).

**Spec:** `docs/superpowers/specs/2026-05-22-fase4-modulos-estandar-design.md` seccion ag-observe.
**Arquitectura:** `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` seccion 14.

---

## Mapa de archivos

### Crear
- `crates/ag-observe/Cargo.toml`
- `crates/ag-observe/src/lib.rs` — `init()`, re-exports
- `crates/ag-observe/src/config.rs` — `ObserveConfig` con `from_env()`
- `crates/ag-observe/src/layer.rs` — construccion del subscriber compuesto
- `crates/ag-observe/src/metrics.rs` — handler Axum `/metrics` + metricas estandar
- `crates/ag-observe/dashboards/overview.json`
- `crates/ag-observe/dashboards/database.json`
- `crates/ag-observe/dashboards/runtime.json`

### Modificar
- `Cargo.toml` (root) — dependencias nuevas al workspace + miembro `crates/ag-observe`

---

## Task 1: Cargo.toml del workspace y del crate

- [ ] Agregar a `[workspace.members]` en `Cargo.toml` root: `"crates/ag-observe"`
- [ ] Agregar a `[workspace.dependencies]`:
  ```toml
  opentelemetry            = { version = "0.27", features = ["trace"] }
  opentelemetry-otlp       = { version = "0.27", features = ["grpc-tonic", "trace"] }
  opentelemetry_sdk        = { version = "0.27", features = ["rt-tokio"] }
  tracing-opentelemetry    = "0.28"
  metrics                  = "0.24"
  metrics-exporter-prometheus = "0.16"
  console-subscriber       = { version = "0.4", optional = true }
  ```
- [ ] Crear `crates/ag-observe/Cargo.toml`:
  ```toml
  [package]
  name = "ag-observe"
  version.workspace = true
  edition.workspace = true
  rust-version.workspace = true
  license.workspace = true
  authors.workspace = true
  repository.workspace = true
  description = "Observabilidad nativa: tracing, OpenTelemetry, Prometheus, dashboards Grafana"
  publish = false

  [features]
  dev-console = ["dep:console-subscriber"]

  [dependencies]
  tracing.workspace = true
  tracing-subscriber.workspace = true
  tracing-opentelemetry.workspace = true
  opentelemetry.workspace = true
  opentelemetry-otlp.workspace = true
  opentelemetry_sdk.workspace = true
  metrics.workspace = true
  metrics-exporter-prometheus.workspace = true
  axum.workspace = true
  tokio.workspace = true
  serde.workspace = true
  console-subscriber = { workspace = true, optional = true }

  [lints]
  workspace = true
  ```
- [ ] Verificar: `cargo check -p ag-observe 2>&1 | grep "^error" | head -5`
- [ ] Commit: `chore(observe): Cargo.toml crate y workspace deps`

---

## Task 2: ObserveConfig

**Files:** `crates/ag-observe/src/config.rs`

- [ ] TDD: test `observe_config_defaults_from_env` — verificar que `LOG_FORMAT=json` produce `LogFormat::Json` y que sin variables el default es `LogFormat::Pretty`
- [ ] Correr para verificar que falla
- [ ] Implementar:
  ```rust
  #[derive(Debug, Clone, serde::Deserialize)]
  pub struct ObserveConfig {
      pub otlp_endpoint: Option<String>,   // OTEL_EXPORTER_OTLP_ENDPOINT
      pub prometheus_port: u16,             // PROMETHEUS_PORT, default 9090
      pub log_format: LogFormat,            // LOG_FORMAT
      pub dev_console: bool,                // AG_DEV_CONSOLE
  }

  #[derive(Debug, Clone, Default, serde::Deserialize)]
  #[serde(rename_all = "lowercase")]
  pub enum LogFormat { #[default] Pretty, Json }

  impl ObserveConfig {
      pub fn from_env() -> Self { /* leer variables de entorno con std::env::var */ }
  }
  ```
- [ ] Correr test
- [ ] Commit: `feat(observe): ObserveConfig from_env`

---

## Task 3: Layer compuesto (fmt + OTLP + metrics)

**Files:** `crates/ag-observe/src/layer.rs`, `crates/ag-observe/src/lib.rs`

- [ ] TDD: test `init_does_not_panic_without_otlp` — llamar `init(ObserveConfig::default())` sin servidor OTLP disponible no debe panic (el exporter OTLP falla silenciosamente o se omite)
- [ ] Correr para verificar que falla
- [ ] Implementar `init(config: ObserveConfig) -> Result<(), ObserveError>`:
  - `tracing_subscriber::registry()` con `fmt::Layer` (JSON o pretty segun config)
  - Si `otlp_endpoint` presente: agregar `OpenTelemetryLayer` con exporter OTLP gRPC
  - Inicializar `metrics-exporter-prometheus` con `PrometheusBuilder::new().install()`
  - Si feature `dev-console` y `config.dev_console`: `console_subscriber::init()`
  - Registrar el subscriber global con `set_global_default`
- [ ] Definir `ObserveError` con variantes `AlreadyInitialized`, `OtlpSetup(String)`
- [ ] Correr test
- [ ] Commit: `feat(observe): init() con layers fmt/OTLP/metrics`

---

## Task 4: Metricas estandar y handler /metrics

**Files:** `crates/ag-observe/src/metrics.rs`

- [ ] TDD: test `metrics_handler_returns_prometheus_text` — llamar `record_request("GET", "/health", 200, 1.0)` y luego obtener el snapshot de metricas, verificar que contiene `ag_requests_total`
- [ ] Correr para verificar que falla
- [ ] Implementar:
  ```rust
  pub fn record_request(method: &str, endpoint: &str, status: u16, duration_secs: f64) {
      metrics::counter!("ag_requests_total",
          "method" => method.to_string(),
          "endpoint" => endpoint.to_string(),
          "status" => status.to_string()
      ).increment(1);
      metrics::histogram!("ag_request_duration_seconds",
          "method" => method.to_string(),
          "endpoint" => endpoint.to_string()
      ).record(duration_secs);
  }

  pub fn set_db_pool(size: u32, idle: u32) {
      metrics::gauge!("ag_db_pool_size").set(size as f64);
      metrics::gauge!("ag_db_pool_idle").set(idle as f64);
  }

  /// Handler Axum para GET /metrics — retorna texto Prometheus.
  pub async fn metrics_handler() -> impl axum::response::IntoResponse {
      // Obtener snapshot del exporter instalado globalmente
      // Retornar con Content-Type: text/plain; version=0.0.4
  }
  ```
- [ ] Correr test
- [ ] Commit: `feat(observe): metricas estandar + handler /metrics`

---

## Task 5: Dashboards Grafana

**Files:** `crates/ag-observe/dashboards/overview.json`, `database.json`, `runtime.json`

- [ ] Crear `overview.json` con paneles: latencia p50/p95/p99 (`ag_request_duration_seconds`), throughput (`ag_requests_total`), tasa de errores (status >= 500)
- [ ] Crear `database.json` con paneles: `ag_db_pool_size`, `ag_db_pool_idle`
- [ ] Crear `runtime.json` con paneles: `ag_active_connections`, memoria del proceso (si disponible)
- [ ] Verificar que los tres JSON son validos: `python3 -c "import json,glob; [json.load(open(f)) for f in glob.glob('crates/ag-observe/dashboards/*.json')]" && echo OK`
- [ ] Commit: `feat(observe): dashboards Grafana — overview, database, runtime`

---

## Task 6: Documentacion y verificacion final

- [ ] Escribir docstring de crate en `lib.rs` con ejemplo de uso minimo
- [ ] `cargo fmt --all && cargo clippy -p ag-observe -- -D warnings`
- [ ] `cargo test -p ag-observe 2>&1 | tail -5`
- [ ] `cargo doc -p ag-observe --no-deps 2>&1 | grep "^error" | head -5`
- [ ] Commit: `docs(observe): docstrings y ejemplo de uso`
- [ ] Merge: `git checkout fase-4 && git merge --no-ff fase-4/ag-observe -m "feat(observe): ag-observe completo — tracing/OTLP/Prometheus/Grafana"`
