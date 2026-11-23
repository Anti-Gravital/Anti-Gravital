# Capitulo 15. Modelo de seguridad

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 15
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [14-observabilidad-ag-observe.md](./14-observabilidad-ag-observe.md)
> Siguiente: [16-rendimiento-y-validacion.md](./16-rendimiento-y-validacion.md)

Sections 14.1 to 14.3 describe the target design. The following is the scope actually implemented in `crates/ag-observe`:

- Metrics: `ag_requests_total`, `ag_request_duration_seconds`, `ag_db_pool_size`, `ag_db_pool_idle` and `ag_active_connections`, exposed by `/metrics`. Percentiles, error rate by code, Redis pool metrics and process/runtime gauges are **not** recorded by the crate; the corresponding dashboard panels are aspirational until implemented.
- Logs: structured `pretty` or `json` output to stdout with an `info` default filter. Export to Loki or Datadog and a correlation ID on every log line are **not** implemented. The OTLP trace layer is opt-in (feature `otlp` plus endpoint), uses HTTP/protobuf, needs no `protoc` and requires no mandatory collector (ADR-0019, RFC-0028).
- `tokio-console` is opt-in via the `dev-console` feature and `AG_DEV_CONSOLE`, not automatic.

Project scope: `AG_PROJECT_ID` is an optional bounded label (1-64 characters from `[A-Za-z0-9_.-]`), validated at startup. An invalid value returns `ObserveError::InvalidScope` without printing the value. When set, it becomes a constant `project` label on every Prometheus metric and a `project.id` attribute on the OTLP resource. It is a logical label, not a security boundary or a tenant identifier.

Component health: `HealthRegistry` holds named synchronous checks. `liveness_handler` returns 200 without evaluating dependencies; `readiness_handler` returns 200 or 503 with a JSON body containing `status`, `project` and each component result. Evaluation is on demand when the endpoint is scraped: no poller, scheduler or external service is added (ADR-0009).

---

## 15. Security model

Security is a cross-cutting concern, not a module. This section documents the project's guarantees and practices.

### 15.1 Guarantees by construction

Rust eliminates by construction four categories of bugs that historically represent more than 70% of the critical vulnerabilities in systems software: use-after-free, buffer overflows, data races, and null pointer dereferences. These guarantees are at the compiler level, not at runtime; they do not require GC or runtime checks.

Anti-Gravital prohibits the use of `unsafe` in all the framework code except in blocks that are explicitly justified, documented, and reviewed by at least two maintainers. Each `unsafe` block comes accompanied by a comment that explains why it is necessary and which invariants it preserves.

### 15.2 Cryptography practices

The cryptographic primitives are imported from the `ring` crate, maintained by members of Google's BoringSSL team. No custom cryptography is rolled. The default algorithms are Ed25519 for signatures, ChaCha20-Poly1305 for AEAD, Argon2id for password hashing, and TLS 1.3 for transport. Legacy algorithms (RSA, AES-CBC, SHA-1) are available only for explicit interoperability.

### 15.3 Responsible disclosure policy

