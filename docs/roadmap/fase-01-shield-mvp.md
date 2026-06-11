# Fase 1 - The Shield MVP

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-00-fundaciones-y-gobernanza.md](./fase-00-fundaciones-y-gobernanza.md)
> Siguiente: [fase-02-core-mvp.md](./fase-02-core-mvp.md)

## Phase 1 — The Shield MVP

**Status: Technical implementation complete.** Detail of boxes in `docs/roadmap/STATUS.md`.

**Objective.** Implement the core's Shield layer: a Tower middleware pipeline that validates, performs basic authentication, applies rate limiting and delivers requests to a placeholder handler. No complete Core yet. No DSL yet. The product is a binary that responds over HTTP with basic security and a publishable benchmark.

### 1.1 Entry criteria

- [ ] Phase 0 completed with all of its exit criteria checked.
- [ ] At least one contributor in addition to the main maintainer is active in the repository.

### 1.2 Deliverables

- [ ] `ag-core` crate with operational `shield` module.
- [ ] HTTP/1.1 and HTTP/2 support via Axum + Tokio.
- [ ] TLS 1.3 termination with rustls.
- [ ] Basic payload validation middleware (deserialization with serde and simple constraints).
- [ ] JWT authentication middleware with Ed25519 verification.
- [ ] Rate limiting middleware with governor (token bucket per IP).
- [ ] CORS and CSRF middleware with secure defaults.
- [ ] Structured logging middleware with `tracing`.
- [ ] Minimal configuration from a TOML file.
- [ ] Unit tests with coverage ≥ 80% of the `ag-core` crate.
- [ ] End-to-end integration tests of the Shield pipeline.
- [ ] Executable Hello World benchmark: `cargo bench` produces reproducible figures.
- [ ] Crate API documentation generated with `cargo doc`, published on `docs.rs`.
- [ ] User manual chapter explaining how to use the Shield directly as a library.

### 1.3 Exit criteria (gate before Phase 2)

- [ ] Hello World benchmark reaches ≥ 300 K req/s on documented reference hardware.
- [ ] Shield pipeline p99 latency ≤ 1 ms at 100 K req/s.
- [ ] Idle process memory ≤ 15 MB.
- [ ] Startup time ≤ 100 ms.
- [ ] CI passes on the four target platforms.
- [ ] Static analysis with `clippy` without warnings.
- [ ] Dependency analysis with `cargo-audit` without known vulnerabilities.
- [ ] Zero undocumented `unsafe` blocks.
- [ ] At least one technical blog post published about the Shield architecture.
- [ ] At least ten stars on the repository.

### 1.4 Phase risks

The main risk is underestimating the complexity of TLS and rate limiting in production. The mitigation is to use exclusively proven crates (rustls, governor) and not to roll our own implementations. The secondary risk is that the benchmark figures do not reach the target; the mitigation is to publish what is measured with honesty and document the shortfall.

---

