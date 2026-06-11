# Capitulo 5. Arquitectura del ecosistema: modulos y responsabilidades

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 5
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [04-estado-del-arte.md](./04-estado-del-arte.md)
> Siguiente: [06-nucleo-shield-y-core.md](./06-nucleo-shield-y-core.md)

## 5. Ecosystem architecture: modules and responsibilities

The most important architectural decision derived from the critical analysis of v3.0 was to separate the core from the ecosystems. The v3.0 tried to be simultaneously a backend framework, an SSR engine, a DevOps platform, an AI orchestrator, an observability layer, a mobile framework, and a plugin system. This is unmanageable. The v4.0 reorganizes the project as an ecosystem of independent Rust crates, each with its own domain, a responsible maintainer, independent semantic versioning, and a minimal API surface.

### 5.1 Ecosystem map

| Crate              | Domain                                                           | Criticality status |
|--------------------|------------------------------------------------------------------|----------------------|
| `ag-core`          | HTTP runtime, router, extractors, error types, Shield/Core       | Core                 |
| `ag-dsl`           | Lexer, parser, AST, semantic analysis and codegen of the Anti-DSL | Core                |
| `ag-cli`           | `ag` binary: new, generate, dev, build, deploy, migrate          | Core                 |
| `ag-auth`          | WebAuthn, JWT Ed25519, OAuth2, RBAC, rate limiting               | Standard             |
| `ag-data`          | sqlx with compile-time verification, migrations, typed ORM       | Standard             |
| `ag-realtime`      | WebSocket, SSE, embedded NATS, pub/sub                           | Standard             |
| `ag-cache`         | in-memory moka, Redis adapter, event-based invalidation          | Standard             |
| `ag-storage`       | S3, MinIO, local filesystem, signed URLs, image processing       | Standard             |
| `ag-observe`       | tracing, OpenTelemetry, Prometheus, Grafana dashboards           | Standard             |
| `ag-lsp`           | Anti-DSL Language Server (diagnostics, completion, hover) for `.ag` files | Core         |
| `ag-mail`          | outbound SMTP, typed templates, send queues with retries, relay SMTP nativo, SPF/DKIM/DMARC helpers | Deferred standard |
| `ag-workers`       | background execution engine: typed jobs, retries, DLQ, scheduling, worker pools | Deferred standard |
| `ag-ui`            | SSR with askama, selective hydration, HTMX integration           | Optional             |
| `ag-cloud`         | Railway-like deployment orchestration, Dockerfile gen            | Optional             |
| `ag-domains`       | DNS management via `DnsProvider` trait, adapters (Cloudflare), ACME certificates, deployment domains | Optional infra |
| `ag-edge`          | request-time edge data plane: hostname routing, SNI certificate selection, canonical/redirect policy | Optional infra |
| `ag-ai`            | Doc generation, schema suggestions, knowledge graph              | Optional             |
| `ag-mobile`        | Dart SDK generation, native Flutter auth, offline sync           | Optional             |
| `ag-migrate`       | OpenAPI, Prisma, Django, FastAPI, Sequelize importers            | Optional             |
| `ag-wasm-host`     | WASI plugin runtime on wasmtime                                  | Core                 |

The distinction between **core**, **standard**, **deferred standard**, and **optional** is important. The core is the minimal set that defines what Anti-Gravital is. The standard modules cover 90% of the production needs of any backend service and are installed by default in the official templates. A **deferred standard** module (introduced by `ADR-0007`) has the maturity and scope of a standard but is NOT installed by default in the templates: it is incorporated when the project explicitly needs it. `ag-mail` is a deferred standard because most backends end up sending transactional mail (verification, recovery, magic links via `ag-auth`), but not every project uses it from minute zero. `ag-workers` (introduced by `RFC-0012` / `ADR-0013` in Phase 4.6-D) is the second deferred standard: most backends eventually need background execution (jobs, retries, scheduling), but not every project uses it from day one, so it has standard maturity without being installed by default in the templates. The optional modules are added when the project needs them; `ag-domains` is an infrastructure optional (it is consumed by `ag-cloud` during deployment) and `ag-cloud -> ag-domains` is a dependency documented in section 5.3. The ecosystem reached **17 crates** with the introduction of Phase 4.5 and has grown additively to **20** with `ag-lsp` (Phase 3 DSL tooling), `ag-edge` (`ADR-0012`) and `ag-workers` (`ADR-0013`).

### 5.2 Ecosystem diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                       Anti-Gravital Ecosystem                    │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌───────────────────────┐    ┌───────────────────────┐         │
│   │       ag-cli          │    │       ag-dsl          │         │
│   │  new · generate · dev │◄──►│  lexer · parser · AST │         │
│   │  build · deploy       │    │  semantic · codegen   │         │
│   └───────────┬───────────┘    └───────────┬───────────┘         │
│               │                            │                     │
│               ▼                            ▼                     │
│   ┌──────────────────────────────────────────────────┐           │
│   │                    ag-core                       │           │
│   │  Shield (Tower middleware) + Core (Axum router)  │           │
│   │  Extractores · Error types · Runtime Tokio       │           │
│   └────────┬───────────────────────┬─────────────────┘           │
│            │                       │                             │
│   ┌────────▼─────────┐    ┌────────▼─────────┐                   │
│   │  Módulos estándar │    │ ag-wasm-host    │                   │
│   │  ag-auth ────────►│    │ wasmtime + WASI │                   │
│   │  ag-data         │    │ plugin lifecycle│                   │
│   │  ag-realtime     │    └─────────────────┘                   │
│   │  ag-cache        │                                          │
│   │  ag-storage      │                                          │
│   │  ag-observe      │                                          │
│   └────────┬─────────┘                                          │
│            │                                                    │
│   ┌────────▼─────────────────┐                                  │
│   │  Estándar diferido       │                                  │
│   │  ag-mail (◄── ag-auth)   │ ──► cooperación SPF/DKIM/DMARC   │
│   │  outbound + adapters     │                                  │
│   │  (relay SMTP nativo)     │                                  │
│   └────────┬─────────────────┘                                  │
│            │                                                    │
│   ┌────────▼──────────────────────────────────────────┐         │
│   │              Módulos opcionales                   │         │
│   │  ag-ui    ag-cloud ─► ag-domains    ag-ai         │         │
│   │  ag-mobile    ag-migrate                          │         │
│   │                                                   │         │
│   │  ag-domains: DnsProvider + ACME + adapters        │         │
│   └───────────────────────────────────────────────────┘         │
└─────────────────────────────────────────────────────────────────┘
```

### 5.3 Dependency rules between crates

To keep the ecosystem healthy, strict dependency rules apply:

First rule: `ag-core` does not depend on any other crate of the Anti-Gravital ecosystem. It is the base on which everything else is built. Any functionality considered sufficiently generic that needs another module must be extracted to `ag-core` or turned into a trait that the module implements.

Second rule: the standard modules can depend on `ag-core` and on other standard modules as long as there are no cycles. For example, `ag-auth` can depend on `ag-data` for session persistence, but `ag-data` cannot depend on `ag-auth`.

Third rule: the optional modules can depend on any core or standard crate. They cannot depend on each other except in explicitly justified cases (for example, `ag-mobile` can depend on `ag-ai` for assisted code generation).

Fourth rule: `ag-cli` depends on all the other crates (it is the orchestrator), but only through Cargo features, so that the `ag` binary can be compiled with a reduced subset.

Fifth rule: all crates publish independent semantic versions. A breaking change in `ag-cache` does not force `ag-core` to bump major. This is essential for the sustainability of an open source project.

Sixth rule (introduced by `ADR-0007`, Phase 4.5): the direction of the `ag-auth <-> ag-mail` dependency is strictly unidirectional. `ag-auth` **consumes** `ag-mail` to send verification, password recovery, and magic link emails, defining a small trait that `ag-auth` invokes. `ag-mail` does **NOT** depend on `ag-auth`. This directionality preserves the second rule (no cycles) and keeps `ag-mail` reusable in isolation in any Rust project. The `ag-mail <-> ag-domains` cooperation (to materialize SPF/DKIM/DMARC) is optional, via a Cargo feature: if a project uses `ag-mail` with a external provider (via SMTP) and does not administer its own DNS, `ag-domains` is not necessary.

Seventh rule (introduced by `ADR-0007`, Phase 4.5): the optional module `ag-cloud` **consumes** `ag-domains` during `ag deploy` to configure DNS and TLS, without the dependency being rigid in all targets. If the project does not declare domains in its `schema.ag`, the flow is omitted. `ag-domains` can be used independently from the CLI without `ag-cloud`.

### 5.4 Monorepo structure

```
anti-gravital/
├── Cargo.toml                  # Workspace root
├── LICENSE                     # Apache 2.0
├── README.md                   # Español + Inglés
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── SECURITY.md                 # Política de divulgación responsable
├── crates/
│   ├── ag-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── shield/         # Middleware Tower
│   │       │   ├── tls.rs
│   │       │   ├── auth.rs
│   │       │   ├── rate_limit.rs
│   │       │   ├── validation.rs
│   │       │   ├── rbac.rs
│   │       │   └── cors.rs
│   │       ├── core/           # Router y handlers
│   │       │   ├── router.rs
│   │       │   ├── extractors.rs
│   │       │   ├── error.rs
│   │       │   └── state.rs
│   │       └── runtime/        # Configuración Tokio
│   │           └── mod.rs
│   ├── ag-dsl/
│   │   └── src/
│   │       ├── lexer.rs
│   │       ├── parser.rs
│   │       ├── ast.rs
│   │       ├── semantic.rs
│   │       ├── diagnostics.rs
│   │       └── codegen/
│   │           ├── rust_gen.rs
│   │           ├── ts_gen.rs
│   │           ├── dart_gen.rs
│   │           ├── openapi_gen.rs
│   │           └── sql_gen.rs
│   ├── ag-cli/
│   ├── ag-lsp/                 # Fase 3 — tooling DSL (núcleo)
│   ├── ag-auth/
│   ├── ag-data/
│   ├── ag-realtime/
│   ├── ag-cache/
│   ├── ag-storage/
│   ├── ag-observe/
│   ├── ag-mail/                # Fase 4.5 — estándar diferido
│   ├── ag-workers/             # Fase 4.6-D — estándar diferido
│   ├── ag-ui/
│   ├── ag-cloud/
│   ├── ag-domains/             # Fase 4.5 — opcional infra
│   ├── ag-edge/                # Fase 4.5 — opcional infra (data plane)
│   ├── ag-ai/
│   ├── ag-mobile/
│   ├── ag-migrate/
│   └── ag-wasm-host/
├── docs/                       # Documentación bilingüe
│   ├── es/
│   └── en/
├── examples/
│   ├── todo-api/
│   ├── ecommerce-api/
│   ├── realtime-chat/
│   ├── ai-backend/
│   └── flutter-fullstack/
├── templates/                  # Templates de `ag new`
│   ├── rest/
│   ├── realtime/
│   ├── fullstack/
│   └── mobile-backend/
├── plugins/                    # Plugins WASM oficiales
│   ├── prometheus-exporter/
│   ├── datadog-exporter/
│   └── sentry/
└── benchmarks/                 # Suite TechEmpower + comparaciones
    ├── hello-world/
    ├── json-crud/
    └── plaintext/
```

---

