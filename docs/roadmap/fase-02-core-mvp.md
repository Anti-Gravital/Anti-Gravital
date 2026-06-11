# Fase 2 - The Core MVP y roundtrip completo

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-01-shield-mvp.md](./fase-01-shield-mvp.md)
> Siguiente: [fase-03-anti-dsl-alpha.md](./fase-03-anti-dsl-alpha.md)

## Phase 2 — The Core MVP and complete roundtrip

**Objective.** Complete the core with the Core layer: Axum router, typed extractors, error system, shared state. Implement the complete roundtrip Request → Shield → Core → Handler → Response. Connect to real PostgreSQL for a minimal CRUD. The product is a binary that serves a real API, although written manually without DSL.

### 2.1 Entry criteria

- [ ] Phase 1 completed with all of its exit criteria checked.
- [ ] The `ag-data` crate has been started with sqlx as a dependency.

### 2.2 Deliverables

- [ ] `ag-core` crate with operational `core` module.
- [ ] Axum router integrated with the Shield.
- [ ] Extractors: `State<T>`, `ValidatedBody<T>`, `Claims<T>`, `Path<T>`, `Query<T>`.
- [ ] `AgError` error system with automatic conversion to HTTP response.
- [ ] Response system: JSON, plaintext, streams.
- [ ] `ag-data` crate with PostgreSQL connection pool via sqlx.
- [ ] Embedded migrations system with `sqlx::migrate!`.
- [ ] Example app `todo-api` in `examples/` with complete CRUD.
- [ ] Executable CRUD + DB benchmark.
- [ ] `ag-cli` crate with commands `new` (creates project from template), `dev` (starts server with hot reload via `cargo-watch`), `build` (compiles release).
- [ ] Three templates: `rest`, `realtime`, `fullstack`.

### 2.3 Exit criteria (gate before Phase 3)

- [ ] CRUD + PostgreSQL benchmark reaches ≥ 40 K req/s on reference hardware.
- [ ] CRUD p99 latency ≤ 5 ms.
- [ ] The `todo-api` app runs successfully with `ag new` + `ag dev`.
- [ ] The `todo-api` app deploys as a single binary (`FROM scratch` Docker).
- [ ] The release binary of `todo-api` occupies ≤ 20 MB.
- [ ] Documentation: "Your first API with Anti-Gravital" published.
- [ ] At least 50 stars on the repository.
- [ ] At least three external contributors with merged PRs.

### 2.4 Phase risks

The main risk is scope drift: wanting to add features not strictly necessary for the Core MVP. The mitigation is an explicit scope declaration in the phase ticket: the Core of this phase does not include complex RBAC authorization, does not include events, does not include cache, does not include complete observability. Those arrive in later phases.

---

