# Fase 4 - Modulos estandar

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-03-anti-dsl-alpha.md](./fase-03-anti-dsl-alpha.md)
> Siguiente: [fase-04-5-ag-mail-y-ag-domains.md](./fase-04-5-ag-mail-y-ag-domains.md)

## Phase 4 — Standard modules

**Objective.** Complete the batteries-included modules: auth, realtime, cache, storage, observe. Each as an independent crate, with tests, documentation and examples.

### 4.1 Entry criteria

- [ ] Phase 3 completed.
- [ ] DSL version 0.5 (auth and policies) started.

### 4.2 Deliverables

- [ ] DSL version 0.5: auth declaration and RBAC policies.
- [ ] DSL version 0.6: events declaration.
- [ ] Complete `ag-auth` crate: WebAuthn, JWT Ed25519, OAuth2 (Google, GitHub), API keys, refresh tokens with rotation.
- [ ] Complete `ag-realtime` crate: binary WebSocket, SSE fallback, embedded NATS for small cases, external NATS client for production.
- [ ] Complete `ag-cache` crate: moka L1 + Redis L2 with fred, event-based invalidation.
- [ ] Complete `ag-storage` crate: S3, MinIO, local filesystem adapters. Signed URLs. Image processing.
- [ ] Complete `ag-observe` crate: tracing, OpenTelemetry exporter, Prometheus metrics, Grafana JSON dashboards included.
- [ ] tokio-console integration in dev mode.
- [ ] `realtime-chat` example in `examples/`.
- [ ] `ai-backend` example in `examples/` that demonstrates SSE streaming.
- [ ] Cross-module integration tests.

### 4.3 Exit criteria (gate before Phase 5)

- [ ] The five modules published on crates.io with their respective independent releases.
- [ ] Test coverage ≥ 80% in each module.
- [ ] Documentation for each module: README, usage guide, API reference.
- [ ] Performance: the `ag-realtime` module sustains 50 K WebSocket connections on a 2 vCPU instance without degradation.
- [ ] Performance: the `ag-cache` module shows ≥ 1 M ops/second in L1.
- [ ] At least five bug report issues closed by the community.
- [ ] At least 500 stars on the repository.

### 4.4 Phase risks

The main risk is the fragmentation of effort among five parallel modules. The mitigation is to sequence the implementation: first auth (blocks many use cases), then advanced data, then realtime, then cache, then storage, then observe.

---

