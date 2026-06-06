# Anti-Gravital Framework — Technical Architecture and Implementation

**[English](#english) | [Espanol](#espanol)**

> This is a bilingual document. English is the canonical, normative version. The Spanish section that follows is a faithful counterpart. Per ADR-0008, when the two diverge, the English text governs.

---

## English

# Anti-Gravital Framework — Technical Architecture and Implementation

**Version:** 4.0 — May 2026
**Organization:** Gravital Labs — Nereira Technology and Business Solutions
**Origin:** Republic of Panama
**License:** Apache 2.0
**Status:** Pre-launch. Active roadmap.

> Master technical document. It specifies the architecture, the components, the implementation contracts, the DSL type system, the plugin model, the security guarantees, and the performance objectives of the Anti-Gravital framework. This document supersedes Blueprint v3.0 and consolidates the architectural decisions derived from the critical external review analysis.

---

## Table of contents

1. Executive summary
2. Manifesto and positioning
3. What Anti-Gravital is and is not (scope and limits)
4. Analysis of the state of the art
5. Ecosystem architecture: modules and responsibilities
6. Core architecture (`ag-core`): Shield and Core
7. The Anti-DSL language (`ag-dsl`): specification and incremental implementation
8. Batteries-included modules
9. WASI plugin system (`ag-wasm-host`)
10. Deployment subsystem (`ag-cloud`)
11. Artificial Intelligence integration (`ag-ai`) and the Knowledge Graph
12. Migration framework (`ag-migrate`): importers
13. Native application bridge (`ag-mobile`): Flutter and generated clients
14. Observability (`ag-observe`)
15. Security model
16. Performance objectives and validation methodology
17. Open Source governance model
18. Risk analysis and mitigations
19. Technical glossary
20. Appendix: market comparison

---

## 1. Executive summary

Anti-Gravital is a free software ecosystem for building high-performance backend applications, written in pure Rust, with three fundamental properties that distinguish it from the rest of the current web framework market.

The first is the total absence of an external runtime: the result of an Anti-Gravital project is a self-contained static binary that runs on the operating system without an interpreter or virtual machine in between. This eliminates the JVM, CPython, Node.js, and the CLR from the execution path and, with them, the garbage collection pauses, the cold-start seconds, and the hundreds of megabytes of base memory that those runtimes consume before processing the first request.

The second is the schema-first approach supported by a domain definition language called Anti-DSL, files with the `.ag` extension. A single file describes models, endpoints, policies, validations, errors, and relationships; the DSL compiler derives from there the Rust code, the typed clients for frontend and mobile applications, the OpenAPI documentation, and the database migrations. The contract is a single source of truth, and schema drift ceases to be a possible class of problem by construction.

The third is a modular architecture conceived as an ecosystem, not as a monolithic framework. The core is deliberately small and is composed with independently published modules. Each module (`ag-auth`, `ag-data`, `ag-realtime`, `ag-cache`, `ag-storage`, `ag-observe`, `ag-cloud`, `ag-ai`, `ag-mobile`, `ag-migrate`) has its own domain, its own versioning, and can be used in isolation in any Rust project. This separation makes the ecosystem sustainable at community scale and eliminates the "framework that tries to solve everything" syndrome.

The project is born in Panama, with bilingual Spanish/English documentation from day zero. The first adoption focus is Latin America; the horizon is global. The Apache 2.0 license guarantees that there will never be a closed Enterprise version or features reserved for paying customers: the entirety of the ecosystem is and will be open source.

This document describes in detail each component, each architectural decision, and the technical commitments that underpin the project. The complementary document *Roadmap and Verification Gates* defines the temporal sequence of deliverables and the blocking criteria between phases.

---

## 2. Manifesto and positioning

For the last twenty years, the software industry has accepted a trade-off that is no longer necessary: choosing between performance and productivity. The world's most widely adopted frameworks thrived by solving only one of the two extremes. Spring Boot and .NET imposed enterprise structure at the price of heavy virtual machines and multi-second startups. Django and FastAPI made it possible for small teams to build APIs in hours, at the price of the GIL and an interpreter that places an invisible ceiling on performance. Node.js brought isomorphic development at the price of a single-threaded event loop and an npm ecosystem chronically vulnerable to supply chain attacks.

None of these frameworks is bad. They all solve real problems. But they were all designed in an era prior to three converging phenomena that change the calculation: the production maturity of the Rust ecosystem, the arrival of AI agents capable of writing quality code at superhuman speeds, and the industry's disenchantment with the operational complexity of multi-language stacks.

Anti-Gravital is built on the premise that systems performance and developer productivity are not opposing forces, but design problems. A correctly designed framework can offer both simultaneously without hidden trade-offs.

The name describes the thesis. Current frameworks have *gravity*: they tie you to interpreters, virtual machines, external runtimes, and abstraction layers that charge in latency, memory, and operational complexity. Anti-Gravital breaks with that gravity from the foundations: no JVM, no GC, no interpreter, no external runtime. Only native machine code, memory safety guaranteed at compile time, and massive concurrency without garbage collection cost.

**Explicit positioning.** Anti-Gravital does not position itself against any language or any framework. It positions itself as the modern unified backend and runtime layer for applications that need three things simultaneously: systems performance, high-level framework productivity, and operational deployment simplicity. The target audience is not the team that already has a Spring stack running in production and has no pain — it is the team that is starting a new project, or that has reached the structural limits of Python/Node.js, or that needs to reduce the memory footprint of its service fleet.

The adoption strategy is built on interoperability and gradual migration, not on the aggressive replacement of existing stacks. The official importers (OpenAPI, Prisma, Sequelize, Django ORM, FastAPI/Pydantic models) are first-class citizens, not an afterthought.

---

## 3. What Anti-Gravital is and is not (scope and limits)

The clear definition of scope is probably the most important architectural decision of this project. A framework that tries to be everything ends up being nothing. This section establishes the explicit limits of the project.

### 3.1 What Anti-Gravital is

Anti-Gravital is:

- A high-performance **Rust backend runtime** for HTTP, WebSocket, and SSE services.
- A **domain definition language** (Anti-DSL, `.ag` files) and its compiler.
- A **unified CLI** (`ag`) for creation, generation, development, build, deployment, and administration.
- A **set of optional modules** published as independent Rust crates (auth, data, realtime, cache, storage, observe, mail —deferred standard—).
- A **domain and TLS management layer** (`ag-domains`, optional infra) that integrates DNS via adapters, ACME for certificates, and SPF/DKIM/DMARC for transactional mail.
- A **WASI plugin system** for isolated multi-language extensibility.
- A **deployment orchestration layer** simplified in the Railway/Fly.io style for common cases (not a Kubernetes replacement).
- A **typed SDK generator** for TypeScript, Dart, and other client languages.
- A **set of migration importers** from legacy frameworks.
- An auto-generated **knowledge graph** that keeps the architectural documentation synchronized with the code.

### 3.2 What Anti-Gravital is NOT

This list is equally important. Anti-Gravital does **not** intend and will not intend to:

- **Replace Kubernetes.** For workloads that justify Kubernetes, Anti-Gravital deploys *on top of* Kubernetes like any other containerized binary. `ag-cloud` covers the range from Docker Compose up to Fly.io. When a team needs orchestration at the scale of hundreds of nodes, it uses Kubernetes and that is that.
- **Replace Flutter or React Native.** Anti-Gravital is not a cross-platform UI framework. It is the ideal native backend *for* Flutter and React Native applications, with automatic generation of typed client SDKs, native authentication, realtime, offline sync, and streaming.
- **Replace React, Vue, Svelte, or Next.js.** The `ag-ui` module offers SSR + HTMX for cases where a full JS stack is excessive, but it does not compete with established frontend frameworks. For SPA or rich SSR applications, the recommended pattern is Anti-Gravital as backend + Next.js (or equivalent) as frontend, communicating via the generated TypeScript client.
- **Replace Docker.** It generates Dockerfiles. It runs in containers. It does not reinvent the OCI format.
- **Replace PostgreSQL, Redis, MinIO, or NATS.** It integrates with them as standard external dependencies.
- **Replace Terraform or Pulumi.** `ag-cloud` orchestrates simple deployments; for complex multi-cloud infrastructure with policies, declarative IaC, and shared modules, Terraform remains the correct tool. `ag-domains` (Phase 4.5) also does not replace Terraform: it orchestrates DNS and TLS for the domains declared in the project's `schema.ag`, it does not manage arbitrary DNS zones or shared infrastructure.
- **Be a complete mail server.** `ag-mail` (Phase 4.5) sends outbound transactional mail (verification, recovery, magic links, alerts) via native SMTP or the native SMTP relay (pointable at any external provider). It is NOT an MTA, it does NOT receive mail (no IMAP/POP), it does NOT offer mailboxes, it does NOT implement antispam or IP reputation management. For inbound or a complete mail server, use Postfix, Stalwart, or another specialized project.
- **Be a domain registrar.** `ag-domains` (Phase 4.5) consumes the domain that the operator already bought (Namecheap, Cloudflare Registrar, etc.) and configures it through an adapter (Cloudflare initially). It does NOT register domains, it does NOT act as a domain marketplace.
- **Be a game engine, a scientific computing framework, or an alternative to Unreal Engine, Unity, NumPy, PyTorch, or TensorFlow.** These domains have specialized tools that Anti-Gravital does not intend to replicate.

### 3.3 The interoperability rule

When a dominant tool exists in an adjacent domain, the strategy is to integrate, not to replace. This rule prevents the project from growing in unmanageable directions and keeps the scope defensible.

---

## 4. Analysis of the state of the art

This section documents the competitive context in technical terms, without adversarial rhetoric. Each framework analyzed solves a real set of problems; the analysis identifies the structural limitations that Anti-Gravital intends to address.

### 4.1 Spring Boot and the JVM ecosystem

Spring Boot dominates enterprise Java and Kotlin development with two decades of mature ecosystem. Its structural weaknesses derive from the JVM: a base memory consumption of 256-512 MB before serving the first request, startup times of 6-8 seconds incompatible with serverless computing, and configuration verbosity. GraalVM Native Image partially mitigates startup and memory, but introduces its own limitations (limited reflection, incomplete library compatibility, long compilation times). The fundamental trade-off — a managed runtime with GC — remains.

### 4.2 ASP.NET Core and .NET

Technically one of the fastest managed frameworks on the market, with modern and expressive C#. The CLR with GC keeps measurable pauses at p99 under sustained load. The technical direction of the ecosystem is unilaterally Microsoft's. Memory safety is not guaranteed by the compiler; race condition bugs and null reference exceptions are possible. Adoption outside the Microsoft ecosystem remains limited for cultural rather than technical reasons.

### 4.3 Django and FastAPI

Django maintains the best prototyping experience in the Python world, with a rich ecosystem for administration, authentication, and templates. FastAPI raised the DX standard for Python APIs with Pydantic types, automatic OpenAPI generation, and native async support. Both share the structural ceiling of CPython: the Global Interpreter Lock prevents real CPU-bound concurrency within a process, which forces scaling with multiple processes (Gunicorn, Uvicorn workers) multiplying memory consumption. Django's async support remains partial; many libraries in the ecosystem remain synchronous.

### 4.4 Node.js, Express, and NestJS

Node.js brought JavaScript to the server and the npm ecosystem is the broadest in the industry. V8's single-threaded event loop is optimal for concurrent I/O but degrades with any CPU-bound work. The npm supply chain is chronically vulnerable: the average transitive dependency of a modern Node.js project exceeds 200 libraries, and incidents of compromised packages are recurrent. TypeScript adds type safety in development, but at runtime it remains JavaScript.

### 4.5 Next.js and the JS fullstack frameworks

Next.js represents the frontend/backend convergence in JavaScript. Server Components and Server Actions reduce the boilerplate of internal APIs. The structural weaknesses are inherited from Node.js: serverless cold starts, de facto coupling with Vercel, inadequacy for persistent WebSockets, shared state, and long-running processing. Next.js is an excellent presentation layer; it is not a robust backend.

### 4.6 Axum, Actix-Web, Rocket (Rust)

The current Rust frameworks are technically excellent in performance (consistently in the TechEmpower top 10) but offer what the community calls a *low-level* experience: the developer builds authentication, the data layer, observability, client generation, and the migration system from scratch. Anti-Gravital is built on Axum, Tokio, and Tower as internal dependencies — it does not compete with them, but packages them into a complete framework experience with DSL, CLI, and opinionated modules.

### 4.7 Conclusion of the analysis

There is a real market space: a dominant enterprise-grade Rust framework does not yet exist. Spring Boot pays the historical cost of the JVM. Node.js has structural event loop limits. Python has concurrency problems. Go sacrifices the type system. Rust has runtime and performance, but lacks a complete framework experience. Anti-Gravital intends to fill that gap.

---

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
| `ag-mail`          | outbound SMTP, typed templates, send queues with retries, relay SMTP nativo, SPF/DKIM/DMARC helpers | Deferred standard |
| `ag-ui`            | SSR with askama, selective hydration, HTMX integration           | Optional             |
| `ag-cloud`         | Railway-like deployment orchestration, Dockerfile gen            | Optional             |
| `ag-domains`       | DNS management via `DnsProvider` trait, adapters (Cloudflare), ACME certificates, deployment domains | Optional infra |
| `ag-ai`            | Doc generation, schema suggestions, knowledge graph              | Optional             |
| `ag-mobile`        | Dart SDK generation, native Flutter auth, offline sync           | Optional             |
| `ag-migrate`       | OpenAPI, Prisma, Django, FastAPI, Sequelize importers            | Optional             |
| `ag-wasm-host`     | WASI plugin runtime on wasmtime                                  | Core                 |

The distinction between **core**, **standard**, **deferred standard**, and **optional** is important. The core is the minimal set that defines what Anti-Gravital is. The standard modules cover 90% of the production needs of any backend service and are installed by default in the official templates. A **deferred standard** module (introduced by `ADR-0007`) has the maturity and scope of a standard but is NOT installed by default in the templates: it is incorporated when the project explicitly needs it. `ag-mail` is a deferred standard because most backends end up sending transactional mail (verification, recovery, magic links via `ag-auth`), but not every project uses it from minute zero. The optional modules are added when the project needs them; `ag-domains` is an infrastructure optional (it is consumed by `ag-cloud` during deployment) and `ag-cloud -> ag-domains` is a dependency documented in section 5.3. The ecosystem reaches **17 crates** with the introduction of Phase 4.5.

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
│   ├── ag-auth/
│   ├── ag-data/
│   ├── ag-realtime/
│   ├── ag-cache/
│   ├── ag-storage/
│   ├── ag-observe/
│   ├── ag-mail/                # Fase 4.5 — estándar diferido
│   ├── ag-ui/
│   ├── ag-cloud/
│   ├── ag-domains/             # Fase 4.5 — opcional infra
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

## 6. Core architecture (`ag-core`): Shield and Core

The core of Anti-Gravital is organized into two conceptual layers within a single Rust process. The separation is not physical: there is no IPC, no FFI, no shared memory between runtimes. The two layers communicate through ordinary Rust function calls, with zero measurable overhead. The separation is logical and exists for two reasons: architectural clarity for the developer, and the future possibility of extracting the Shield as an independent gateway if a use case justifies it.

### 6.1 Core diagram

```
┌─────────────────────────────────────────────────────────────┐
│                  ag-core · Single Process                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌─────────────────────────────────────────────────┐       │
│   │            CAPA A — The Shield                  │       │
│   │  (Tower middleware composable pipeline)         │       │
│   │                                                 │       │
│   │   ┌─────────┐  ┌─────────┐  ┌─────────────┐     │       │
│   │   │ TLS 1.3 │─►│   JWT   │─►│ Rate Limit  │     │       │
│   │   │ rustls  │  │ Ed25519 │  │  governor   │     │       │
│   │   └─────────┘  └─────────┘  └──────┬──────┘     │       │
│   │                                    │            │       │
│   │   ┌─────────┐  ┌─────────┐  ┌──────▼──────┐     │       │
│   │   │  CORS   │◄─│  RBAC   │◄─│ Validación  │     │       │
│   │   │  CSRF   │  │ Guards  │  │  Schema     │     │       │
│   │   └─────────┘  └─────────┘  └─────────────┘     │       │
│   └─────────────────────┬───────────────────────────┘       │
│                         │                                   │
│              Llamada de función Rust (0ns)                  │
│                         ▼                                   │
│   ┌─────────────────────────────────────────────────┐       │
│   │            CAPA B — The Core                    │       │
│   │  (Axum router · Handlers · Estado)              │       │
│   │                                                 │       │
│   │   ┌────────────┐   ┌─────────────────────┐      │       │
│   │   │  Router    │   │  Business handlers  │      │       │
│   │   │  Axum      │──►│  (generados por DSL)│      │       │
│   │   └────────────┘   └──────────┬──────────┘      │       │
│   │                               │                 │       │
│   │   ┌────────────┐   ┌──────────▼──────────┐      │       │
│   │   │ Extractores│   │  Estado compartido  │      │       │
│   │   │  tipados   │   │  AppState           │      │       │
│   │   └────────────┘   └─────────────────────┘      │       │
│   └─────────────────────────────────────────────────┘       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
                            │
                  cargo build --release
                            ▼
                ┌────────────────────────┐
                │  Single Static Binary  │
                │  FROM scratch Docker   │
                └────────────────────────┘
```

### 6.2 The Shield: the trust layer

The Shield is responsible for everything that happens before a request is considered trusted and delivered to the business code. It is implemented as a pipeline of Tower layers, the same composable model that Axum uses internally. Each layer is optional and is configured from the project's `schema.ag`.

The technical stack of the Shield is: Tokio as the M:N async runtime (it multiplexes millions of tasks over a fixed thread pool of size equal to available CPUs), Tower as the composable middleware model, rustls for TLS 1.3 without an OpenSSL dependency, serde and serde_json for zero-copy serialization where possible, ring for low-level cryptographic primitives, governor for rate limiting with a lock-free token bucket algorithm.

The standard layers of the Shield, in execution order over an incoming request:

The first layer is TLS termination, managed by rustls. It supports TLS 1.3 with modern cipher suites, OCSP stapling, and ALPN for HTTP/1.1 vs HTTP/2 negotiation. For environments where TLS termination is performed by an external load balancer (Cloudflare, AWS ALB, Nginx), this layer is disabled with an option in the schema.

The second layer is payload deserialization and validation. For requests with a body, the contract defined in the `.ag` is applied: types, length constraints, email format, regex, numeric ranges. A violation produces a 422 error with structured detail about which field failed and why.

The third layer is authentication. It supports JWT signed with Ed25519 (Edwards25519 curve, faster and more secure than RS256), Passkeys/WebAuthn (FIDO2), API keys, and cookie-based sessions. Verification is eager for endpoints marked as `auth required`.

The fourth layer is rate limiting. Implemented with governor over a token bucket algorithm, it supports limits per IP, per authenticated user, per endpoint, and per combinations. The limits are declared in the schema.

The fifth layer is RBAC authorization. The policies are declared in the `.ag` as expressions that are evaluated against the JWT claims and the request parameters. For example: `policy "user.role == ADMIN || user.id == params.id"`.

The sixth layer is CORS and CSRF. Configured by default with secure values (no wildcard); any deviation requires explicit declaration.

### 6.3 The Core: the business logic layer

The Core is where 80% of the application code that the developer writes lives. It is Axum with a thin layer of conventions on top.

The handlers have a signature generated by the DSL compiler from the declared endpoint:

```rust
// Generado automáticamente por `ag generate` desde schema.ag
// El desarrollador solo escribe el cuerpo del handler.
pub async fn create_user(
    State(state): State<AppState>,
    ValidatedBody(req): ValidatedBody<CreateUserRequest>,
    Claims(claims): Claims<AuthClaims>,
) -> Result<Json<User>, AgError> {
    // El desarrollador solo escribe esto:
    let user = state.db.users()
        .create(CreateUserParams {
            email: req.email,
            name: req.name,
            created_by: claims.user_id,
        })
        .await?;

    state.events.emit("user.created", &user).await?;
    Ok(Json(user))
}
```

The type `ValidatedBody<T>` guarantees that the body already passed the Shield's validation. The type `Claims<T>` guarantees that the JWT was already verified. The type `AgError` is an enum that covers all the errors declared in the endpoint, and the conversion to an HTTP response is automatic via `IntoResponse`.

The application state (`AppState`) is a generated struct that contains clients to the project's resources: the database pool, the NATS client, the Redis client, the S3 client. It is built at binary startup and shared by reference (cheap `Arc` clones) among all handlers.

### 6.4 Error handling

The Anti-Gravital error system follows three principles. The first: each endpoint explicitly declares which errors it can produce in its `.ag` definition. This produces a typed `EndpointError` enum in which each variant is a specific error. The second: errors propagate with Rust's `?` operator, and the conversion to an HTTP response is automatic and consistent. The third: no error is silently discarded. Unexpected errors produce a structured 500 with a correlation ID that is linked to the stack trace in the tracing system.

### 6.5 Runtime and Tokio configuration

Anti-Gravital uses Tokio in multi-thread mode with default configuration: one worker per available CPU, a blocking pool of 512 threads. For standard IO-bound workloads this configuration is optimal. The schema allows adjustments:

```yaml
runtime:
  workers: auto              # Por defecto = núm CPUs
  blocking_threads: 512
  thread_stack: 2MB
  shutdown_timeout: 30s
```

---

## 7. The Anti-DSL language (`ag-dsl`): specification and incremental implementation

The DSL compiler is, together with the runtime, the technically most demanding component of the project. It is essentially a complete compiler: lexer, parser, semantic analysis, type system, code generation to multiple targets, formatter, linter, and LSP server. This section defines the language specification and the incremental implementation strategy.

### 7.1 Language philosophy

The Anti-DSL is declarative, not imperative. It describes contracts, not flows. The premise is that most of the value of a backend framework resides in the consistency of its external contract: which models exist, which endpoints expose them, which rules apply, which errors are returned. The internal logic stays in pure Rust.

The language draws inspiration from Prisma for the model syntax, from GraphQL SDL for the clarity of definitions, from sqlc for the integration with SQL, and from protobuf for the multi-target codegen. It is not a Turing-complete language and does not intend to be.

### 7.2 Incremental implementation by DSL versions

Probably the most important decision for making the compiler viable is to admit that the complete language cannot be delivered in the first version. The specification is delivered in incremental phases, each with a stable grammar that does not break the previous one. The DSL versions are independent of the framework versions and follow their own semver.

| DSL version | Grammatical capability                                                                                              | Milestone                  |
|-------------|-------------------------------------------------------------------------------------------------------------------|----------------------------|
| v0.1        | Basic models: fields, primitive types, annotations `@primary`, `@unique`, `@auto`                                  | End of Phase 3 (delivered) |
| v0.2        | Endpoints: method, path, body, response, errors                                                                    | End of Phase 3 (delivered) |
| v0.3        | Validations: `@min`, `@max`, `@email`, `@regex`, `@length`                                                         | End of Phase 3 (delivered) |
| v0.4        | Relationships between models: `1:1`, `1:N`, `N:M`, cascades                                                        | End of Phase 3 (delivered) |
| v0.5        | Authentication and authorization: `auth required`, `policy "..."`                                                  | End of Phase 4 (delivered) |
| v0.6        | Events: declaration of events emitted per endpoint, subscribers                                                    | End of Phase 4 (delivered) |
| v0.7        | Mail and declarative domains: `mail`, `domain`, `dns`, `tls`                                                       | End of Phase 4.5           |
| v0.8        | Plugin hooks (lifecycle, decorators)                                                                               | End of Phase 9             |
| v1.0        | Stable grammar, frozen under semver. Any subsequent extension will be additive.                                    | End of Phase 10            |

This table is realigned by `ADR-0007` (Phase 4.5). The multi-tenancy and data migration capabilities planned for intermediate versions of the DSL in earlier revisions are deferred: they will be specified in their own RFCs when the scope justifies it, without occupying a fixed numbered slot until then. This avoids promising features that do not have verified traction.

### 7.3 Complete schema example (v1.0 target)

```ag
# schema.ag — Ejemplo completo objetivo de la v1.0

config {
    project_name "fintech-api"
    database "postgres"
    runtime { workers auto, blocking_threads 512 }
}

enum UserRole {
    ADMIN
    USER
    BANNED
}

model User {
    id           UUID       @primary @auto
    email        String     @unique @max(255) @email
    password_hash String    @max(255)
    name         String     @max(100) @min(2)
    role         UserRole   @default(USER)
    created      Timestamp  @auto
    updated      Timestamp  @auto_update

    accounts     Account[]  @relation(name: "owner")
}

model Account {
    id        UUID      @primary @auto
    owner_id  UUID      @references(User.id) @on_delete(cascade)
    balance   Decimal   @precision(18,2) @default(0)
    currency  String    @length(3)
    created   Timestamp @auto
}

request CreateUserRequest {
    email     String @email
    password  String @min(12) @max(128)
    name      String @min(2) @max(100)
}

response UserResponse {
    id        UUID
    email     String
    name      String
    role      UserRole
    created   Timestamp
}

error EmailTaken      { status 409 message "Email already registered" }
error WeakPassword    { status 422 message "Password does not meet policy" }
error InsufficientFunds { status 402 message "Insufficient balance" }

endpoint CreateUser {
    method   POST
    path     /users
    auth     optional
    body     CreateUserRequest
    response UserResponse
    errors   [EmailTaken, WeakPassword]
    events   [user.created]
    rate_limit "5/min per_ip"
}

endpoint TransferFunds {
    method   POST
    path     /accounts/{from}/transfer
    auth     required
    policy   "user.id == params.from.owner_id"
    body     TransferRequest
    response TransferReceipt
    errors   [InsufficientFunds]
    events   [account.debited, account.credited, transfer.completed]
    rate_limit "10/min per_user"
}

event user.created {
    payload UserResponse
    retain  30d
}

event account.debited {
    payload AccountDelta
    retain  7y
}
```

### 7.4 Artifacts generated from a single schema

A single `schema.ag` produces, via `ag generate`:

| Artifact                           | Path                                | Purpose                                      |
|------------------------------------|-------------------------------------|----------------------------------------------|
| Rust structs with serde and validators | `src/models.rs`                 | Domain types                                 |
| Typed handler stubs                | `src/handlers/*.rs`                 | Ready signatures; the dev writes the body    |
| Compile-time checked sqlx queries  | `src/db/queries.rs`                 | Type-safe database access                    |
| Versioned SQL migrations           | `migrations/NNNN_*.sql`             | Database schema                              |
| TypeScript types                   | `clients/typescript/types.ts`       | Types shared with the frontend               |
| TypeScript HTTP client             | `clients/typescript/client.ts`      | Typed SDK for the frontend                   |
| Dart types                         | `clients/dart/lib/types.dart`       | Types shared with Flutter applications       |
| Dart client with Dio               | `clients/dart/lib/client.dart`      | Typed SDK for Flutter                        |
| OpenAPI 3.1 documentation          | `openapi.yaml`                      | Interactive documentation (Swagger UI)       |
| AsyncAPI specification             | `asyncapi.yaml`                     | Event documentation                          |
| JSON knowledge graph               | `.ag/knowledge-graph.json`          | Input for `ag-ai` and dashboards             |

### 7.5 Compiler architecture

The DSL compiler is organized in a traditional pipeline with well-defined stages:

The **lexer** phase tokenizes the `.ag` input. It is implemented with `logos` (a Rust crate that generates tokenizers from declarative definitions with derive macros). It produces a stream of positional tokens for error reporting with lines and columns.

The **parser** phase consumes tokens and produces an AST. It is implemented with `chumsky` (a parser combinator library with error recovery support), chosen over `nom` for its better handling of error messages readable by the end user.

The **semantic analysis** phase validates the AST: it checks that the references between models exist, that the types are consistent, that the RBAC policies refer to valid fields, that there are no cycles in the relationships, that the names do not collide with reserved words of Rust or SQL. It produces structured diagnostics with suggestions.

The **codegen** phase takes the validated AST and emits code to multiple targets. Each target (Rust, TypeScript, Dart, OpenAPI, SQL) is an independent module. The emission is done with `askama` templates for the textual outputs and with `quote` for the Rust code (which benefits from having a native Rust AST for emission).

### 7.6 Language server (LSP)

From version 0.3 of the DSL, an LSP server is included that offers autocompletion, live diagnostics, go-to-definition, find-references, hover types, and rename. It is distributed as an `ag-lsp` binary and integrates with any editor compatible with the protocol (VS Code, Neovim, Helix, Zed, IntelliJ via plugin).

The official plugin for VS Code is published in the marketplace under the name `Anti-Gravital`.

### 7.7 DSL tooling

The CLI offers three specific commands for the DSL:

`ag schema lint` reviews the `.ag` file and reports warnings about bad practices (models without indexes on foreign key fields, endpoints without rate limit, tautological policies, unhandled errors).

`ag schema diff <ref>` compares the current schema against a reference (git commit, tag, file) and reports breaking vs non-breaking changes. Essential for pull request reviews.

`ag schema migrate` generates the SQL migration needed to bring the database from the current state to the schema state. It includes a safety analysis: it detects destructive operations (drop column, drop table) and demands explicit confirmation.

---

## 8. Batteries-included modules

This section specifies each of the standard modules of the ecosystem. Each subsection documents the purpose, the technical stack, the design decisions, and the extension points.

### 8.1 `ag-auth` — Authentication and authorization

The authentication module implements the modern identity schemes. The central architectural decision is to support Passkeys/WebAuthn as first-class, not as an afterthought; passwords are a legacy mechanism that is supported but not recommended.

The technical stack implemented is custom WebAuthn with `ciborium` (CBOR) and COSE verification (`p256` for ES256, `ed25519-dalek` for EdDSA) instead of `webauthn-rs`, for Apache-2.0 license compatibility (see `ADR-0006`); `jsonwebtoken` for JWT, `argon2` for password hashing (when used), and `oauth2` as the OAuth2 client (Google, GitHub) with the PKCE flow.

The supported flows are: registration and authentication with passkey, authentication with email + password (legacy), OAuth2 with preconfigured providers (Google, GitHub, Microsoft, Gravital ID), API keys for server-to-server integrations, and refresh tokens with rotation.

JWTs are signed with Ed25519 by default (Edwards25519 curve, faster than RSA and more secure than ECDSA-P256 against side-channel attacks). The private key lives in an external secret manager (HashiCorp Vault, AWS Secrets Manager, GCP Secret Manager) or in environment variables with documented rotation.

The RBAC is declared in the schema and compiled to evaluable expressions. The policy is evaluated once per request in the Shield, before reaching the handler. Policies can reference JWT claims, path parameters, and query the database if explicitly declared (with caching to avoid the N+1).

### 8.2 `ag-data` — Data access and migrations

The data module is built on sqlx, with compile-time verification of SQL queries. This means that when `cargo build` runs, sqlx connects to a development database (configurable by environment variable) and verifies that each query is syntactically valid and that the types of the returned columns match the Rust structs that receive them. A SQL error ceases to be a runtime error; it becomes a compile error.

The supported backends are PostgreSQL (recommended for production), SQLite (for development, tests, and edge applications), and MySQL (for legacy environments).

Migrations are embedded in the binary with `sqlx::migrate!`. This means that the binary itself contains the complete history of migrations, and at startup it can automatically apply the pending ones. For environments where this is not desirable (blue-green deployments with migration as a separate step), the `ag migrate apply` command runs the migrations without bringing up the server.

For multi-tenant architectures, `ag-data` natively supports schema-per-tenant in PostgreSQL: each tenant has its own schema with the same tables, and the connection router selects the schema based on the JWT claim. It also supports Row-Level Security (RLS) for cases where schema isolation is excessive.

Read replicas are configured declaratively; the module routes read-only queries to the nearest replica and write queries to the primary.

### 8.3 `ag-realtime` — Events and real-time communication

`ag-realtime` offers three modalities of bidirectional communication: binary WebSocket, Server-Sent Events for unidirectional streams, and a pub/sub event bus.

The event bus uses NATS as the broker. For small cases, NATS runs embedded in the same Anti-Gravital binary (edge mode). For cases at scale, the binary connects to an external NATS cluster. This duality allows starting simple and scaling without rewriting.

For WebSocket, the internal binary protocol (based on msgpack) reduces the overhead compared to JSON. The WebSocket handlers are declared in the schema and receive messages already deserialized to Rust structs.

For SSE, it is used as an automatic fallback in browsers that do not support WebSocket or are behind proxies that block it. The negotiation is transparent.

Event persistence uses JetStream (a component of NATS) when available, which allows event replay for new consumers and durability against broker crashes.

### 8.4 `ag-cache` — Multi-level cache

The cache module offers two levels. The L1 level, already implemented, is an in-memory cache with `moka`, a concurrent implementation without contended locks based on TinyLFU, with tag-based invalidation. The L2 level (distributed cache between instances) is not yet implemented: `RFC-0005` proposes a native L2 compatible with the RESP2 protocol, without a dependency on Redis as an external service, and remains pending approval and implementation.

Invalidation is done by events. When an endpoint emits an event (`user.updated`), `ag-cache` automatically invalidates the related entries at both levels. The invalidation policy is declared in the schema.

The SQL query cache is automatic: queries marked with `@cache(ttl: 5m)` in the schema are cached transparently, and invalidation is triggered when an event touches any of the involved tables.

### 8.5 `ag-storage` — Object storage

`ag-storage` offers an abstraction over three backends: S3 (AWS and compatibles), MinIO (self-hosted), and local filesystem (for development). The backend is selected by configuration; the application code does not notice.

Signed URLs for download and direct upload are generated with a single call: `storage.signed_url(key, Duration::from_mins(15), Permission::Write)`.

Image processing (resize, compress, format conversion) is done with the `image` crate, supporting JPEG, PNG, WebP, and AVIF. Thumbnails are generated automatically on upload if the policy is declared in the schema.

### 8.6 `ag-observe` — Traceability, metrics, and logging

Observability is a first-level concern and not an optional module for production. Its stack is `tracing` for structured spans, `opentelemetry-rust` for export to compatible backends (Jaeger, Tempo, Datadog, Honeycomb), `metrics` for metrics with a Prometheus backend, and pre-configured Grafana dashboards that are included as JSON in the repository.

Each request traverses the whole system with a unique correlation ID that appears in all structured logs, all tracing spans, and all errors returned to the client. This solves the problem of debugging in production: given a support ticket with a correlation ID, the operator can reconstruct the complete path of the request.

`tokio-console` is integrated in development mode for live inspection of the Tokio tasks.

### 8.7 `ag-ui` — Optional Server-Side Rendering

The SSR module exists for cases where a SPA frontend is excessive: internal dashboards, marketing pages, simple forms, and administrative interfaces. It is based on `askama` (build-time compiled templating, with verified types) and native integration with HTMX for interactivity without heavy JavaScript frameworks.

This module is explicitly *not* a competitor of React, Vue, Svelte, or Next.js. For SPA or rich SSR applications, the recommended pattern is Anti-Gravital as backend with a Next.js (or other) frontend that consumes the generated TypeScript client.

### 8.8 `ag-mail` — Transactional communication (deferred standard)

Introduced by `ADR-0007` in Phase 4.5. `ag-mail` is a **deferred** standard module: it has the maturity and scope of a standard, but is NOT installed by default in the official templates. It is incorporated when the project requires outbound transactional mail (account verification, magic links, password recovery, alerts, notifications).

The v1 scope is **exclusively outbound**. `ag-mail` is NOT an MTA, it does NOT receive mail (no IMAP/POP), it does NOT offer persistent mailboxes, it does NOT implement antispam, filtering, or IP reputation management. This restriction is deliberate and is fixed in the ADR: the inbound and complete mail server capabilities are the work of a different project, not of Anti-Gravital.

> **Scope update (`ADR-0010`, 2026-06-03).** The "NOT an MTA / inbound never" restriction above is **superseded**: `ag-mail` is being expanded into a native outbound MTA (MX resolution, ESMTP+STARTTLS delivery, DKIM signing, bounce classification) so a project can send authenticated mail with no third party in the sending path. The expansion is phased and opt-in behind Cargo features (`mta`, `api`, `queue-jetstream`); the outbound-relay baseline described here remains the default and is unchanged. Mailbox hosting, IMAP/POP/JMAP and general inbound stay out of scope; inbound is admitted only as DSN/ARF parsing for bounce processing. Technical plan: `RFC-0009`. The native MTA is forward work and is not claimed as implemented here.

The technical stack is `lettre` with async Tokio transport and `rustls` for the native SMTP sender (coherent with The Shield). External providers are reached with the native SMTP relay; the no-third-party path is the native MTA (`mta` feature). The `MailSender` trait abstracts both:

```rust
#[async_trait::async_trait]
pub trait MailSender: Send + Sync {
    async fn send(&self, msg: &Email) -> Result<MessageId, AgMailError>;
    fn provider_name(&self) -> &'static str;
    fn dns_requirements(&self, domain: &str) -> Vec<DnsRecordSpec>;
}

pub enum AgMail {
    Native(SmtpSender),                // lettre + rustls
    Adapter(Box<dyn MailSender>),      // external providers via SMTP
}
```

The Native | Adapter pattern is identical to the one used by `ag-storage` (`Native | S3`) and to the one planned for the L2 of `ag-cache` (L1 `moka` native today; L2 RESP2 native proposed in `RFC-0005`), reinforcing the project's interoperability rule: integrate dominant providers, do not replace them.

The **templates** are modeled with the `MailTemplate` trait and a `StringTemplate` implementation of `{{var}}` substitution; any external engine (askama, minijinja) can be plugged in by implementing the trait. Variable validation is done against the `schema.ag`: the DSL compiler emits a warning when the `from` of a `mail` block does not reference a declared `domain`, and verifies that the typed `vars` of the template match the markers used. A malformed email ceases to be a runtime bug and approaches a build-detectable error. This is the **real differentiator** (build-time correctness), not deliverability: deliverability is the provider's job; the correctness of the contract is the framework's job.

The **async queue** accepts jobs with retries and exponential backoff. Default backend in memory (Tokio task + channel). Optional persistent backend via `ag-data` (jobs table) to survive restarts. Optional integration with `ag-realtime` for event fan-out. Each job emits metrics towards `ag-observe`: `ag_mail_sent_total`, `ag_mail_failed_total`, `ag_mail_retry_total`, latency histogram.

The **integration with `ag-auth`** is strictly unidirectional: `ag-auth` consumes `ag-mail` by invoking a small trait that `ag-auth` defines. `ag-mail` does NOT know about `ag-auth`. The sixth rule of section 5.3 documents this directionality.

`mail` block of DSL v0.7 (example):

```ag
mail WelcomeEmail {
    from "hello@plenty.market"      # debe referenciar un bloque domain
    subject "Welcome to Plenty"
    template "emails/welcome.html"  # debe existir
    vars {
        name String
        activation_url String        # debe usarse en el HTML
    }
}
```

### 8.9 `ag-domains` — Domain and TLS management (optional infra)

Introduced by `ADR-0007` in Phase 4.5. `ag-domains` is an **infrastructure optional** module: not every backend administers DNS (many deploy behind a proxy or PaaS that already resolves it), but when a project wants `ag deploy` to deliver a URL `https://miapi.example.com` with a valid certificate in a single command, `ag-domains` is the responsible module.

The module is **NOT a domain registrar**: the domain is bought externally (Namecheap, Cloudflare Registrar, etc.) and delegated via nameservers to the configured provider. `ag-domains` also does not replace Terraform or Pulumi: for complex multi-cloud infrastructure or centralized management of arbitrary DNS zones, the project should use the dominant tools. The boundary is fixed in the ADR.

The core of the module is the `DnsProvider` trait:

```rust
#[async_trait::async_trait]
pub trait DnsProvider: Send + Sync {
    async fn list_records(&self, zone: &str) -> Result<Vec<DnsRecord>, AgDomainsError>;
    async fn upsert_record(&self, zone: &str, record: &DnsRecord) -> Result<(), AgDomainsError>;
    async fn delete_record(&self, zone: &str, id: &str) -> Result<(), AgDomainsError>;
    fn provider_name(&self) -> &'static str;
}
```

Small, versioned, with **contract tests** that every adapter must pass. The initial adapter is Cloudflare (authentication by API token). The trait is designed to add Route53, Namecheap, DigitalOcean, etc. in later iterations without touching the public surface.

The **ACME client** (`instant-acme`) issues and renews Let's Encrypt certificates. It supports the DNS-01 challenge (preferred, uses the `DnsProvider` itself to create the required TXT) and HTTP-01 (alternative). The renewal runs as a background Tokio task, watching expiration and renewing before the configured threshold. Certificate storage is filesystem by default, or optional `ag-storage`.

The cooperation with `ag-mail` materializes in `generate_mail_records`: `ag-mail` declares its requirements via `MailSender::dns_requirements` and `ag-domains` materializes them as records (SPF, DKIM, DMARC). This is a **cooperation** relationship, not a control one: `ag-mail` does not depend on `ag-domains`; a project can use `ag-mail` with a external provider (via SMTP) without `ag-domains` participating.

The **propagation verification** uses `hickory-resolver` to query multiple public resolvers and confirm that the records propagated before marking an operation as successful. This blocks `ag deploy` until the domain responds, avoiding delivering URLs that the operator promised but that do not resolve yet.

`domain` block of DSL v0.7 (example):

```ag
domain plenty.market {
    provider "cloudflare"
    tls { mode auto  acme true }
    dns {
        CNAME "api"     -> "ag-cloud-target"
        TXT   "_dmarc"  -> "v=DMARC1; p=quarantine"
    }
    mail { spf auto  dkim auto  dmarc quarantine }
}
```


---

## 9. WASI plugin system (`ag-wasm-host`)

The extensibility of Anti-Gravital is built on the WebAssembly System Interface (WASI), not on the ecosystem of native Rust crates. This decision has three reasons that make it non-negotiable.

The first reason is security. Plugins are third-party code that the server operator runs. If they were native code, a malicious or defective plugin could corrupt the process memory, escape to arbitrary syscalls, or leak secrets. WASI modules run in a sandbox with explicit permissions declared in the plugin manifest; the plugin cannot access the filesystem, the network, or syscalls that are not declared.

The second reason is multi-language. A WASI plugin can be written in Rust, Go (TinyGo), C, C++, AssemblyScript, Zig, or any language that compiles to WebAssembly. This democratizes the ecosystem: a security expert who writes in Go can contribute an exporter for Datadog without having to learn Rust.

The third reason is ABI stability. The interface between the host and the plugin is defined with `wit-bindgen` and the Component Model, which allows a plugin compiled for one version of Anti-Gravital to keep working with future versions without recompilation, as long as the ABI does not change.

### 9.1 Plugin runtime

The runtime is `wasmtime`, embedded as a Rust crate. Each plugin is loaded into an isolated store with memory limits (256 MB by default, configurable), fuel limits (instruction consumption), and execution timeout.

### 9.2 Plugin lifecycle

The lifecycle of a plugin has five states. The first is **discovered**: the `.wasm` file is in the project's plugins directory and appears in the manifest. The second is **validated**: the host inspects the binary, verifies that the component model is compatible, reads the manifest, and confirms that the requested permissions are authorized. The third is **loaded**: the module is compiled ahead-of-time with Cranelift and stored in memory. The fourth is **active**: the plugin receives events and responds to invocations. The fifth is **unloaded**: the plugin is released, whether by server shutdown or by dynamic reload.

### 9.3 Plugin manifest

Each plugin brings a `plugin.toml` file with its metadata and requested permissions:

```toml
[plugin]
name = "datadog-exporter"
version = "1.2.0"
author = "Gravital Labs"
license = "Apache-2.0"
description = "Exports metrics and traces to Datadog"

[abi]
anti_gravital_version = ">= 1.0.0, < 2.0.0"
component_model = "0.5"

[permissions]
network = ["api.datadoghq.com:443", "api.datadoghq.eu:443"]
env = ["DD_API_KEY", "DD_SITE"]
filesystem = []
clock = "read"

[capabilities]
exports = ["metrics_exporter", "trace_exporter"]
imports = ["host_logger", "host_clock"]

[limits]
max_memory = "64MB"
max_execution_time = "5s"
fuel = 100_000_000
```

### 9.4 Host API exposed to plugins

The host exposes a reduced set of capabilities to plugins, defined in WIT (WebAssembly Interface Types) interfaces. The main ones are: logger (write messages to the host's tracing system), clock (get the current time and measure intervals), metrics (register additional metrics), KV (persistent key-value storage per plugin), HTTP client (with an allowlist of hosts from the manifest), and events (subscription to the internal bus).

### 9.5 Framework extension points

Plugins can extend Anti-Gravital at five points: additional middleware in the Shield (request hooks), custom handlers registered in the router, observability exporters (metrics, traces, logs), event processors (subscribers to the internal bus), and custom CLI commands (`ag <plugin-cmd>`).

### 9.6 Official plugins

The repository maintains a set of official plugins under `plugins/`, each with its own crate and release cycle: `prometheus-exporter`, `datadog-exporter`, `sentry`, `honeycomb-exporter`, `slack-notifier`, `discord-webhook`. The existence of official plugins serves as a technical reference and as an implementation example for third parties.

### 9.7 Plugin registry

Starting from version 1.0 of the framework, an official registry is published at `plugins.antigravital.dev`. The registry indexes plugins with verified metadata, basic security scanning, and community reviews. Installation is done with `ag plugin add <name>`. Plugins are downloaded, validated, and registered in the project manifest.

---

## 10. Deployment subsystem (`ag-cloud` + `ag-domains`)

One of the most important structural corrections derived from the critical analysis is that `ag-cloud` is not a competitor of Terraform or of Kubernetes. Its target range is the same one covered by Railway, Fly.io, Render, and Coolify: simplify the deployment of backend applications to typical environments without forcing the team to operate complete infrastructure. Since Phase 4.5 (`ADR-0007`), `ag-cloud` cooperates with `ag-domains` to resolve domain, TLS, and mail records within the `ag deploy` flow itself, without replacing the dominant providers (Let's Encrypt, Cloudflare) and without becoming a hosting panel.

### 10.1 Philosophy of `ag-cloud`

The typical operator of an Anti-Gravital project, especially in its first years of life, does not need or want to operate a Kubernetes cluster. They need to bring up their API on a VPS, connect it to a database, put it behind TLS, and forget about it. `ag-cloud` solves this case.

For more complex cases (multi-region deployments, high availability, centralized secret management, IAM policies, infrastructure shared between multiple applications), `ag-cloud` is not the correct tool and the project must declare it openly: use Terraform, Pulumi, or Helm.

### 10.2 The `deploy.ag` file

The deployment subsystem is controlled with a declarative `deploy.ag` file separate from the project schema:

```yaml
app:
  name: payments-api
  domain: api.example.com

runtime:
  replicas: 3
  port: 8080
  health_check: /health
  resources:
    cpu: 1
    memory: 512MB

database:
  type: postgres
  version: "16"
  size: 20GB
  backup_schedule: "daily"

cache:
  type: redis
  version: "7"
  size: 1GB

storage:
  type: s3
  bucket: payments-api-uploads

secrets:
  source: vault
  path: secret/payments-api

observability:
  metrics: prometheus
  traces: tempo
  logs: loki

deployment:
  target: docker-compose      # opciones: docker-compose, fly, railway, k8s
  strategy: rolling
  max_surge: 1
  max_unavailable: 0
```

### 10.3 Supported deployment targets

`ag-cloud` supports four deployment targets, each with a different level of abstraction.

The **docker-compose** target generates a complete `docker-compose.yml` with services, networks, volumes, healthchecks, secrets loaded from `.env` files or from a secret manager, a reverse proxy (Caddy by default) with automatic TLS via Let's Encrypt, and backup scripts for the database. It is the recommended target for self-hosting on a single VPS.

The **fly** target generates a `fly.toml` and runs the `flyctl` commands needed to deploy to Fly.io. It is the recommended target for global edge computing with low operational overhead.

The **railway** target generates the configuration for Railway and triggers the deployment via its API. It is the recommended target for teams that prefer PaaS without operation.

The **k8s** target generates standard Kubernetes manifests (Deployment, Service, Ingress, ConfigMap, Secret, HorizontalPodAutoscaler) with reasonable values. For advanced configurations, this target is a starting point that the team customizes, not a complete solution.

### 10.4 Deployment pipeline

The `ag deploy` command runs a standardized pipeline: schema validation, compilation with `cargo build --release --target <target>`, construction of the Docker image from a `scratch` or `distroless` base, execution of smoke tests, push of the image to a registry, application of database migrations in order, rolling deployment with healthchecks, and post-deployment verification.

### 10.5 Reverse proxy and TLS

For docker-compose deployments, `ag-cloud` configures Caddy as a reverse proxy with automatic TLS. Caddy obtains and renews Let's Encrypt certificates without explicit configuration. For environments where TLS is managed by an external load balancer (Cloudflare, AWS ALB), Caddy is disabled.

### 10.6 Integration with `ag-domains`

Introduced by `ADR-0007`. When a project declares domains in its `.ag` contract (`domain` block of DSL v0.7), `ag deploy` resolves a six-step flow coordinated with `ag-domains`:

1. **Validate domain control.** Insertion of a verification TXT record via the configured `DnsProvider` and confirmation of its presence.
2. **Configure application DNS.** `upsert_record` to point the domain to the deployment target (CNAME to the Fly/Railway host, or A/AAAA records in docker-compose).
3. **Issue or renew TLS.** ACME client against Let's Encrypt (DNS-01 preferred). The certificate is stored in filesystem or `ag-storage`.
4. **Associate the domain to the target.** Configure the reverse proxy (Caddy in docker-compose, fly cert in Fly, etc.) to serve the domain with the issued certificate.
5. **Materialize SPF/DKIM/DMARC** that `ag-mail` has declared in its `MailSender::dns_requirements`.
6. **Verify propagation** against multiple public resolvers before marking the deployment as successful.

`ag-cloud` does **NOT depend rigidly** on `ag-domains` in all targets: if the project does not declare domains, the flow is omitted. If the target is one where TLS is managed by an external load balancer (Cloudflare in front, AWS ALB), `ag-cloud` can skip step 3 without affecting the rest of the pipeline. This flexibility is what keeps `ag-domains` as an optional module, not as a mandatory piece of the runtime.

---

## 11. Artificial Intelligence integration (`ag-ai`) and the Knowledge Graph

The `ag-ai` module is probably the most significant differentiator of the project in the 2026 context, where AI agents are everyday collaborators in software development. The module has two complementary components.

### 11.1 The Anti-DSL as a contract for agents

The first component is not code: it is the architectural decision that the `schema.ag` serves as a perfect contract for AI agents. An agent that receives an endpoint declared in `.ag` has exactly what it needs to generate a correct handler: precise types, defined errors, access policies, validations, events to emit. And, a critical difference with any other framework, the Rust compiler then verifies that the code generated by the agent is type-safe before it reaches production.

This turns the development flow into a structured collaboration. The engineer designs the schema. The agent implements the handlers. The compiler acts as an automatic second reviewer that rejects any desynchronization. The operator supervises and approves.

### 11.2 The Knowledge Graph

The second component is the project's knowledge graph. `ag-ai` maintains a directed graph that indexes all the project's entities and their relationships: models, endpoints, events, policies, dependencies between handlers, database calls, calls to external services, configurations, installed plugins.

The graph is automatically rebuilt on each `ag generate` and serialized to `.ag/knowledge-graph.json`. From this input the following are automatically produced: architectural documentation in Markdown, C4 diagrams (Context, Container, Component) in Mermaid format, suggested architectural decision records (ADRs), critical dependency lists, impact maps for proposed changes, and an interactive dashboard in the dev server.

### 11.3 Assisted AI capabilities

The module exposes three additional capabilities accessible from the CLI:

`ag ai suggest-schema` analyzes a domain described in natural language and proposes a first draft of `schema.ag`. The engineer refines from there.

`ag ai review-migration` analyzes a proposed SQL migration and reports risks: locks, downtime, data loss, slow queries during the transition. It suggests alternatives with a two-step migration when necessary.

`ag ai analyze-architecture` produces a report on the project's knowledge graph: identification of hotspots (models with too many dependencies), endpoints without tests, events emitted but without consumers, dead code, antipatterns.

### 11.4 Connection with model providers

The module does not embed any language model. It connects to external providers via API: Anthropic Claude, OpenAI, local models served by Ollama or vLLM. The connection is configurable and the operator can choose or self-host. For sensitive environments, an offline mode is supported where the AI functions are disabled but the rest of the framework works normally.

---

## 12. Migration framework (`ag-migrate`): importers

The real adoption of any successful backend framework has always passed through the possibility of migrating from the incumbent. The industry hates rewrites. The `ag-migrate` module is not an afterthought; it is a first-class citizen of the project.

### 12.1 Supported importers

`ag-migrate` offers official importers for the most adopted frameworks on the market.

The **OpenAPI** importer consumes any OpenAPI 3.0 or 3.1 spec and produces a `schema.ag` with models, endpoints, errors, and validations. It is the most generic importer and serves to migrate from any service that documents an OpenAPI, regardless of the language it is written in.

The **Prisma** importer consumes a `schema.prisma` file and translates models, relationships, and migrations to Anti-Gravital. It covers migration from TypeScript applications that use Prisma as an ORM.

The **Django** importer reads Django models (defined as Python classes) and produces the equivalent Anti-Gravital models. It includes translation of relationships, managers, signals, and migrations.

The **FastAPI** importer consumes FastAPI applications by examining the routers and the Pydantic models. It produces Anti-Gravital endpoints and models. It is probably the most natural migration case due to the philosophical similarity between FastAPI and Anti-Gravital.

The **Sequelize** importer reads models from Node.js applications that use the Sequelize ORM. It covers the Express + Sequelize case, very common in the market.

The **GraphQL** importer consumes a GraphQL SDL schema and produces its equivalent in Anti-Gravital.

### 12.2 Honest limitations

The importers cover the translation of the contract (models, endpoints, validations), not the business logic. The logic of the handlers must be written manually or with the assistance of an AI agent. This is documented clearly to avoid erroneous expectations.

### 12.3 Official migration guides

For each supported framework, an official guide is published in the documentation: recommended strategy (big bang vs strangler fig), patterns for coexistence during the transition (reverse proxy that splits traffic between the legacy system and the new one), comparative testing, and real case studies when available.

---

## 13. Native application bridge (`ag-mobile`): Flutter and generated clients

The most important repositioning derived from the critical analysis is that Anti-Gravital does not compete with Flutter. It positions itself as **the ideal native backend for Flutter applications**. This multiplies the strategic value of the project: instead of competing with a mature and very well designed cross-platform UI framework, Anti-Gravital becomes its natural companion.

### 13.1 Dart SDK generation

`ag-mobile` generates a complete Dart package from the `schema.ag`. The package includes types generated with freezed for immutability, an HTTP client based on dio with interceptors for authentication, a WebSocket client for realtime, Server-Sent Events support, and mocks for tests.

### 13.2 Native Flutter authentication

The module includes widgets and services ready for the authentication flows. The WebAuthn integration leverages the native platforms (Android Credential Manager API, iOS Passkeys via AuthenticationServices). The OAuth2 flow uses `flutter_appauth` with auto-generated configuration.

### 13.3 Offline-first and synchronization

`ag-mobile` offers an optional offline synchronization layer. Operations are queued locally in a SQLite database, replicated to the server when there is connectivity, and conflicts are resolved with declarative policies in the schema (last-write-wins, custom merge, server-wins). It is an ambitious functionality that is implemented in a late phase of the roadmap.

### 13.4 Other generated clients

Although Flutter is the priority target for mobile, the codegen system is extensible. Version 1.0 includes generators for Dart (Flutter), TypeScript (React, Vue, Svelte, Next.js), and Kotlin (native Android, optionally Kotlin Multiplatform). A later version may include Swift and Python.

---

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

---

## 15. Security model

Security is a cross-cutting concern, not a module. This section documents the project's guarantees and practices.

### 15.1 Guarantees by construction

Rust eliminates by construction four categories of bugs that historically represent more than 70% of the critical vulnerabilities in systems software: use-after-free, buffer overflows, data races, and null pointer dereferences. These guarantees are at the compiler level, not at runtime; they do not require GC or runtime checks.

Anti-Gravital prohibits the use of `unsafe` in all the framework code except in blocks that are explicitly justified, documented, and reviewed by at least two maintainers. Each `unsafe` block comes accompanied by a comment that explains why it is necessary and which invariants it preserves.

### 15.2 Cryptography practices

The cryptographic primitives are imported from the `ring` crate, maintained by members of Google's BoringSSL team. No custom cryptography is rolled. The default algorithms are Ed25519 for signatures, ChaCha20-Poly1305 for AEAD, Argon2id for password hashing, and TLS 1.3 for transport. Legacy algorithms (RSA, AES-CBC, SHA-1) are available only for explicit interoperability.

### 15.3 Responsible disclosure policy

The repository maintains a `SECURITY.md` file with contact addresses (primary `anti@gravitalcloud.com`, backup `angelnereira@gravitalcloud.com`) and a clear policy: vulnerabilities are reported privately, the team confirms receipt within 48 hours, publishes a patch within 30 days for critical vulnerabilities, and a CVE with credit to the reporter.

### 15.4 Audits

Before the stable 1.0 version, the Shield component of the framework undergoes an external audit by a company specialized in Rust systems security (Trail of Bits, NCC Group, or equivalent). The audit report is published with the release.

### 15.5 Continuous fuzzing

The DSL parser and the HTTP parser undergo continuous fuzzing with `cargo-fuzz`. The CI runs fuzzing corpora on each PR; before 1.0, at least 72 hours of fuzzing without crashes are completed on each parser.

---

## 16. Performance objectives and validation methodology

This section replaces the absolute benchmarks of v3.0. The previous figures were presented as facts when in reality they are extrapolations of individual components. This version rephrases them honestly as **design objectives**, against which the project will measure itself publicly.

### 16.1 Design objectives

| Metric                                                     | Objective                 | Extrapolation basis                          |
|------------------------------------------------------------|---------------------------|----------------------------------------------|
| Hello World throughput (plaintext)                         | >= 300 K req/s            | Axum + Tokio in TechEmpower                  |
| Simple JSON throughput                                     | >= 150 K req/s            | Axum + serde_json in public benchmarks       |
| CRUD throughput with PostgreSQL                            | >= 40 K req/s             | sqlx + connection pool                       |
| p99 latency with DB query                                  | <= 5 ms                   | Measurements of Tokio services in production |
| Base memory (idle process, no traffic)                     | <= 15 MB                  | Size of Rust + Tokio binaries                |
| Cold start time                                            | <= 100 ms                 | Static Rust binaries on Linux                |
| Release binary size with all standard modules              | <= 20 MB                  | Compilations of similar projects             |
| Concurrent WebSocket connections on a 2 vCPU instance      | >= 50 000                 | Tokio stackless tasks                        |

These figures are technical objectives. The project specification requires that they be measured with the `ag bench` suite in the repository, and that each release publish the reproducible results. If a metric is not reached, it is published as such and the deficit is documented. The technical credibility of the project depends on not exaggerating.

### 16.2 Measurement methodology

Any comparison with competing frameworks is done under the TechEmpower Framework Benchmarks, run by the team or by independent third parties. The comparisons published in the documentation include: the exact version of the compared framework, the configuration used, the benchmark hardware, the number of runs, and the standard deviation. Comparisons that do not comply with these rules are not published.

### 16.3 Validation milestones for v1.0

The stable 1.0 version is released only when the following milestones are met:

- Top-10 position in a TechEmpower Round (Plaintext and JSON Serialization categories)
- External security audit without unresolved critical findings
- 72 hours of fuzzing of the DSL parser and the HTTP parser without crashes
- Load test of 500 K req/s sustained for 30 minutes without degradation >5%
- 24 hours of continuous load without detectable memory growth
- Binaries verified on Linux x86-64, Linux ARM64, macOS ARM64, Windows x64
- At least one service in production on Gravital Cloud for 30 days without incidents
- At least three external projects using Anti-Gravital in production

---

## 17. Open Source governance model

### 17.1 License and promise

The license is Apache 2.0 for the entire ecosystem. There is not and will not be a closed Enterprise version with features reserved for paying customers. The commitment is explicit and is documented in the README. Any future license change would require the approval of the entire community of maintainers, and the ecosystem remains forkable.

### 17.2 Maintenance model

The project adopts an initial BDFL model with a transition plan to explicit meritocracy. In the initial phase (0.x versions), Angel Nereira is the principal maintainer. Starting from version 1.0, a technical committee of five people elected among the contributors with the greatest track record is established. The committee approves RFCs (Request For Comments) for major changes.

### 17.3 RFCs

Any change that affects the public API, the DSL, the plugin architecture, or the security model requires an RFC. The process is: the proposer opens an issue in `anti-gravital-rfcs/`, the community debates for at least two weeks, the technical committee votes. Once approved, the RFC is moved to the "Accepted" state and is implemented in a specific version.

### 17.4 Compatibility

After version 1.0, the project commits to strict semver in the public API. Breaking changes only in majors. The LTS versions are announced with a public calendar, with at least 18 months of security support.

### 17.5 Economic sustainability

The project is sustained on three legs. The first is Gravital Labs (Nereira Technology and Business Solutions), which finances the initial development as a strategic investment. The second is professional services: adoption consulting, training, and premium support for companies that want an SLA, without this closing product features. The third, in the future, are corporate sponsors (GitHub Sponsors, Open Collective) of companies that depend on the project.

---

## 18. Risk analysis and mitigations

This section documents the real risks of the project and the planned mitigations. It is deliberately honest; a project that does not enumerate its risks does not deserve trust.

### 18.1 Risk: DSL compiler complexity

The DSL compiler is a several-year project on its own. The mitigation is the incremental implementation by DSL versions described in section 7. Version 0.1 covers only basic models and is deliverable in two months. Each version adds a well-defined subset. The stable 1.0 version of the DSL is the highest-risk milestone of the project and is planned for the end of the schedule.

### 18.2 Risk: Rust learning curve

Rust has a real learning curve. The mitigation is threefold. First, the DSL generates 80% of the scaffolding, so that the handlers the developer writes are simple Rust: a few `await`s, access to shared state, returning a `Result`. Second, the documentation includes a "Rust for Python/Node.js developers" guide with the minimum necessary concepts. Third, the integrated AI assistant can generate handlers that the developer supervises.

### 18.3 Risk: competition with big players

Spring, .NET, Express, and FastAPI have decades-old ecosystems. Anti-Gravital cannot compete frontally with them in breadth. The mitigation is to focus on niches where the incumbents have structural weaknesses: high-load applications, edge services, backends for Flutter, backends for AI applications with streaming.

### 18.4 Risk: bus factor

The initial project has a worryingly low bus factor (one maintainer). The mitigation is active: complete internal documentation from day one, incorporation of external contributors from phase 1, and transition to a technical committee before 1.0.

### 18.5 Risk: changes in the Rust ecosystem

The Rust ecosystem continues to evolve rapidly. Axum, Tokio, and sqlx may make breaking changes in future versions. The mitigation is conservative version pinning, exhaustive integration tests against each new version of the core dependencies, and active participation in their communities to anticipate changes.

### 18.6 Risk: community fragmentation

If the Anti-Gravital community fragments (for example, competing forks with divergent features emerge), the ecosystem weakens. The mitigation is an open RFC process that gives a real voice to the community, predictable releases, and a public roadmap.

### 18.7 Risk: post-launch security vulnerabilities

Although Rust eliminates many categories of vulnerabilities, it does not eliminate the logical ones (broken authorization, information leaks, application-level races). The mitigation is the external audit before 1.0, the responsible disclosure program, continuous fuzzing, and CI with static analysis (clippy, cargo-audit, cargo-deny).

---

## 19. Technical glossary

| Term                          | Definition                                                                                                                |
|-------------------------------|--------------------------------------------------------------------------------------------------------------------------|
| Anti-DSL (.ag)                | Domain definition language of the framework. Schema-first.                                                                |
| Axum                          | Rust HTTP framework built on Tokio and Tower. Base of the Core.                                                           |
| Backpressure                  | Mechanism by which the system rejects new work when it is saturated. Implemented natively in Tower.                       |
| Cargo                         | Rust's build system and package manager.                                                                                  |
| Cargo-fuzz                    | Fuzzing tool integrated with Cargo.                                                                                        |
| Core (layer B)                | Business logic layer of the core. Axum router, handlers, shared state.                                                    |
| Correlation ID                | Unique identifier per request that traverses all the logs, traces, and errors.                                            |
| Ed25519                       | Digital signature algorithm based on the Edwards25519 curve. Default for JWT in Anti-Gravital.                            |
| Flamegraph                    | CPU profiling visualization. With pure Rust it covers the whole application without gaps.                                 |
| Fuel (wasmtime)               | Quota of instructions that a WASM plugin can execute before being interrupted.                                            |
| GIL                           | Global Interpreter Lock. CPython mechanism that prevents real parallel execution.                                         |
| Governor                      | Rust crate for rate limiting based on token bucket. Thread-safe without contended locks.                                  |
| HTMX                          | Small JavaScript library that allows interactivity without SPA frameworks.                                                |
| JetStream                     | NATS message persistence system. Allows replay and durability.                                                            |
| Knowledge Graph               | Directed graph of the Anti-Gravital project. Indexes models, endpoints, events, dependencies.                             |
| LSP                           | Language Server Protocol. The `.ag` DSL offers LSP for autocompletion in editors.                                         |
| Moka                          | Concurrent Rust cache with TinyLFU. Thread-safe without contended locks.                                                  |
| NATS                          | pub/sub messaging system used by `ag-realtime`.                                                                           |
| OpenAPI                       | Standard specification to describe HTTP APIs. Anti-Gravital generates it automatically.                                   |
| Passkeys                      | FIDO2/WebAuthn standard for passwordless authentication.                                                                  |
| Ring                          | Low-level cryptography Rust crate. Maintained by members of the BoringSSL team.                                           |
| Rustls                        | TLS 1.3 implementation in pure Rust, without OpenSSL.                                                                     |
| Schema drift                  | Condition where the definition of a schema becomes desynchronized between layers. Anti-Gravital eliminates it by design.  |
| Schema-per-tenant             | Multi-tenant architecture where each client has its own schema in PostgreSQL.                                             |
| Shield (layer A)              | Trust layer of the core. Tower middleware pipeline: TLS, auth, validation, rate limit, RBAC, CORS.                        |
| sqlx                          | Rust database access crate with compile-time query verification.                                                          |
| TechEmpower                   | Industry-standard benchmark suite for comparing web frameworks.                                                           |
| Tokio                         | Rust async runtime. Provides M:N concurrency through lightweight tasks without GC.                                        |
| tokio-console                 | Live diagnostic tool for Tokio applications.                                                                              |
| Tower                         | Rust crate for composable services and middleware. Architectural base of the Shield.                                     |
| WASI                          | WebAssembly System Interface. Standard for WebAssembly modules with controlled access to the system.                     |
| wasmtime                      | WebAssembly runtime embeddable in Rust. Host of the plugin system.                                                        |
| WebAuthn                      | W3C standard for authentication with hardware factors (passkeys, security keys).                                          |
| Zero-copy                     | Data transfer without copying it in memory. Reduces CPU overhead.                                                         |
| Zero-overhead abstraction     | Rust principle: an abstraction must not cost performance versus the equivalent manual code.                               |
| ACME                          | Automatic Certificate Management Environment. Protocol for automatic issuance and renewal of TLS certificates (Let's Encrypt). Used by `ag-domains` since Phase 4.5. |
| DKIM                          | DomainKeys Identified Mail. Mail authentication mechanism by cryptographic signature of the sender domain. Generated by `ag-domains` for `ag-mail`. |
| SPF                           | Sender Policy Framework. DNS record that enumerates the servers authorized to send mail on behalf of the domain. Generated by `ag-domains` for `ag-mail`. |
| DMARC                         | Domain-based Message Authentication, Reporting and Conformance. Policy that indicates how to treat mail that fails SPF or DKIM. Generated by `ag-domains`. |
| MTA                           | Mail Transfer Agent. Complete mail server (Postfix, Stalwart). `ag-mail` v1 is **NOT an MTA**: it only sends outbound, it does not receive inbound. |
| DnsProvider                   | `ag-domains` trait that abstracts DNS providers through adapters. Initial adapter: Cloudflare. Designed to add Route53, Namecheap, etc. with contract tests. |
| Deferred standard             | Crate classification introduced by `ADR-0007`. Crate with standard maturity that is NOT installed by default in official templates. `ag-mail` is the first case. |

---

## 20. Appendix: market comparison

This comparison is offered as a technical reference. The figures for the competitors are based on verifiable public benchmarks (TechEmpower, GitHub issues, official documentation). Those of Anti-Gravital are design objectives, not measurements.

| Criterion                      | Spring Boot   | .NET Core     | FastAPI      | NestJS       | Anti-Gravital (objective)   |
|--------------------------------|---------------|---------------|--------------|--------------|-----------------------------|
| Runtime                        | JVM           | CLR           | CPython      | Node.js V8   | None (native binary)        |
| Base memory                    | ~350 MB       | ~120 MB       | ~60 MB       | ~80 MB       | <= 15 MB                    |
| Startup time                   | ~6 s          | ~0.8 s        | ~0.8 s       | ~1.2 s       | <= 0.1 s                    |
| Hello World throughput         | ~75 K req/s   | ~200 K req/s  | ~28 K req/s  | ~45 K req/s  | >= 300 K req/s              |
| CRUD + DB throughput           | ~15 K req/s   | ~30 K req/s   | ~5 K req/s   | ~8 K req/s   | >= 40 K req/s               |
| Memory safety                  | Partial       | Partial       | Yes          | No           | Total (Rust compiler)       |
| GC pauses                      | Yes (JVM GC)  | Yes (CLR GC)  | Not applicable | Yes (V8 GC) | No (no GC)                  |
| Deployment as single binary    | No            | Partial       | No           | No           | Yes                         |
| Schema-first DX                | No            | No            | Partial      | No           | Yes (Anti-DSL)              |
| Compile-time verified queries   | No           | No            | No           | No           | Yes (sqlx)                  |
| Native DX for AI agents        | No            | No            | Partial      | No           | Yes                         |
| Native cross-compilation       | No            | No            | No           | No           | Yes                         |
| License                        | Apache 2.0    | MIT           | MIT          | MIT          | Apache 2.0                  |

---

**End of the Technical Architecture document.**
Complementary document: *Roadmap and Verification Gates.*
Unified PDF version: *Anti-Gravital Blueprint v4.0 — Master Document.*

---

## Espanol

# Anti-Gravital Framework — Arquitectura Técnica e Implementación

**Versión:** 4.0 — Mayo 2026
**Organización:** Gravital Labs — Nereira Technology and Business Solutions
**Origen:** República de Panamá
**Licencia:** Apache 2.0
**Estado:** Pre-lanzamiento. Hoja de ruta activa.

> Documento técnico maestro. Especifica la arquitectura, los componentes, los contratos de implementación, el sistema de tipos del DSL, el modelo de plugins, las garantías de seguridad y los objetivos de rendimiento del framework Anti-Gravital. Este documento sustituye el Blueprint v3.0 y consolida las decisiones arquitectónicas derivadas del análisis crítico de revisión externa.

---

## Tabla de contenidos

1. Resumen ejecutivo
2. Manifiesto y posicionamiento
3. Qué es Anti-Gravital y qué no es (alcance y límites)
4. Análisis del estado del arte
5. Arquitectura del ecosistema: módulos y responsabilidades
6. Arquitectura del núcleo (`ag-core`): Shield y Core
7. El lenguaje Anti-DSL (`ag-dsl`): especificación e implementación incremental
8. Módulos batteries-included
9. Sistema de plugins WASI (`ag-wasm-host`)
10. Subsistema de despliegue (`ag-cloud`)
11. Integración con Inteligencia Artificial (`ag-ai`) y el Knowledge Graph
12. Framework de migración (`ag-migrate`): importadores
13. Puente de aplicaciones nativas (`ag-mobile`): Flutter y clientes generados
14. Observabilidad (`ag-observe`)
15. Modelo de seguridad
16. Objetivos de rendimiento y metodología de validación
17. Modelo de gobernanza Open Source
18. Análisis de riesgos y mitigaciones
19. Glosario técnico
20. Apéndice: comparativa de mercado

---

## 1. Resumen ejecutivo

Anti-Gravital es un ecosistema de software libre para construir aplicaciones backend de alto rendimiento, escrito en Rust puro, con tres propiedades fundamentales que lo distinguen del resto del mercado de frameworks web actuales.

La primera es la ausencia total de runtime externo: el resultado de un proyecto Anti-Gravital es un binario estático autocontenido que se ejecuta sobre el sistema operativo sin intérprete ni máquina virtual de por medio. Esto elimina la JVM, CPython, Node.js y CLR del path de ejecución y, con ellos, las pausas de recolección de basura, los segundos de arranque en frío y los cientos de megabytes de memoria base que esos runtimes consumen antes de procesar la primera petición.

La segunda es el enfoque schema-first apoyado en un lenguaje de definición de dominio llamado Anti-DSL, archivos con extensión `.ag`. Un único archivo describe modelos, endpoints, políticas, validaciones, errores y relaciones; el compilador del DSL deriva de allí el código Rust, los clientes tipados para frontend y aplicaciones móviles, la documentación OpenAPI, y las migraciones de base de datos. El contrato es una sola fuente de verdad, y la deriva de esquema (schema drift) deja de ser una clase de problema posible por construcción.

La tercera es una arquitectura modular pensada como un ecosistema, no como un framework monolítico. El núcleo es deliberadamente pequeño y se compone con módulos publicados de forma independiente. Cada módulo (`ag-auth`, `ag-data`, `ag-realtime`, `ag-cache`, `ag-storage`, `ag-observe`, `ag-cloud`, `ag-ai`, `ag-mobile`, `ag-migrate`) tiene un dominio propio, versionado propio, y puede usarse de forma aislada en cualquier proyecto Rust. Esta separación hace al ecosistema sostenible a escala de comunidad y elimina el síndrome del "framework que intenta resolverlo todo".

El proyecto nace desde Panamá, con documentación bilingüe español/inglés desde el día cero. El primer foco de adopción es Latinoamérica; el horizonte es global. La licencia Apache 2.0 garantiza que no existirá nunca una versión Enterprise cerrada ni features reservadas para clientes pagos: la totalidad del ecosistema es y será código abierto.

Este documento describe en detalle cada componente, cada decisión arquitectónica, y los compromisos técnicos que sustentan el proyecto. El documento complementario *Hoja de Ruta y Puertas de Verificación* define la secuencia temporal de entregables y los criterios de bloqueo entre fases.

---

## 2. Manifiesto y posicionamiento

Durante los últimos veinte años, la industria del software ha aceptado un compromiso que ya no es necesario: elegir entre rendimiento y productividad. Los frameworks más adoptados del mundo prosperaron resolviendo solo uno de los dos extremos. Spring Boot y .NET impusieron estructura empresarial al precio de máquinas virtuales pesadas y arranques de varios segundos. Django y FastAPI hicieron posible que equipos pequeños construyeran APIs en horas, al precio del GIL y un intérprete que pone un techo invisible al rendimiento. Node.js trajo desarrollo isomórfico al precio de un event loop monohilo y un ecosistema npm crónicamente vulnerable a ataques de cadena de suministro.

Ninguno de estos frameworks es malo. Todos resuelven problemas reales. Pero todos fueron diseñados en una época anterior a tres fenómenos convergentes que cambian el cálculo: la madurez de producción del ecosistema Rust, la llegada de agentes de IA capaces de escribir código de calidad a velocidades sobrehumanas, y el desencanto de la industria con la complejidad operacional de los stacks multilenguaje.

Anti-Gravital se construye sobre la premisa de que el rendimiento de sistemas y la productividad del desarrollador no son fuerzas opuestas, sino problemas de diseño. Un framework diseñado correctamente puede ofrecer ambos simultáneamente sin compromisos ocultos.

El nombre describe la tesis. Los frameworks actuales tienen *gravedad*: te atan a intérpretes, máquinas virtuales, runtimes externos y capas de abstracción que cobran en latencia, memoria y complejidad operacional. Anti-Gravital rompe con esa gravedad desde los cimientos: sin JVM, sin GC, sin intérprete, sin runtime externo. Solo código máquina nativo, seguridad de memoria garantizada en compilación, y concurrencia masiva sin costo de recolección de basura.

**Posicionamiento explícito.** Anti-Gravital no se posiciona contra ningún lenguaje ni ningún framework. Se posiciona como la capa backend y runtime unificada moderna para aplicaciones que necesitan tres cosas simultáneamente: rendimiento de sistemas, productividad de framework de alto nivel, y simplicidad operacional de despliegue. El público objetivo no es el equipo que ya tiene un stack Spring funcionando en producción y no tiene dolor — es el equipo que está empezando un proyecto nuevo, o que ha alcanzado los límites estructurales de Python/Node.js, o que necesita reducir la huella de memoria de su flota de servicios.

La estrategia de adopción se construye sobre interoperabilidad y migración gradual, no sobre el reemplazo agresivo de stacks existentes. Los importadores oficiales (OpenAPI, Prisma, Sequelize, Django ORM, modelos FastAPI/Pydantic) son ciudadanos de primera clase, no un afterthought.

---

## 3. Qué es Anti-Gravital y qué no es (alcance y límites)

La definición clara del alcance es probablemente la decisión arquitectónica más importante de este proyecto. Un framework que intenta ser todo termina siendo nada. Esta sección establece los límites explícitos del proyecto.

### 3.1 Qué es Anti-Gravital

Anti-Gravital es:

- Un **runtime backend Rust** de alto rendimiento para servicios HTTP, WebSocket y SSE.
- Un **lenguaje de definición de dominio** (Anti-DSL, archivos `.ag`) y su compilador.
- Una **CLI unificada** (`ag`) para creación, generación, desarrollo, build, despliegue y administración.
- Un **conjunto de módulos opcionales** publicados como crates Rust independientes (auth, data, realtime, cache, storage, observe, mail —estándar diferido—).
- Una **capa de gestión de dominios y TLS** (`ag-domains`, opcional infra) que integra DNS vía adapters, ACME para certificados, y SPF/DKIM/DMARC para correo transaccional.
- Un **sistema de plugins WASI** para extensibilidad multilenguaje aislada.
- Una **capa de orquestación de despliegue** simplificada al estilo Railway/Fly.io para casos comunes (no un reemplazo de Kubernetes).
- Un **generador de SDKs tipados** para TypeScript, Dart y otros lenguajes cliente.
- Un **conjunto de importadores de migración** desde frameworks legacy.
- Un **knowledge graph** auto-generado que mantiene la documentación arquitectónica sincronizada con el código.

### 3.2 Qué NO es Anti-Gravital

Esta lista es igualmente importante. Anti-Gravital **no** intenta ni intentará:

- **No reemplaza Kubernetes.** Para cargas que justifican Kubernetes, Anti-Gravital se despliega *sobre* Kubernetes como cualquier otro binario contenedorizado. `ag-cloud` cubre el rango Docker Compose hasta Fly.io. Cuando un equipo necesita orquestación a escala de cientos de nodos, usa Kubernetes y se acabó.
- **No reemplaza Flutter ni React Native.** Anti-Gravital no es un framework de UI multiplataforma. Es el backend nativo ideal *para* aplicaciones Flutter y React Native, con generación automática de SDKs cliente tipados, autenticación nativa, realtime, offline sync y streaming.
- **No reemplaza React, Vue, Svelte ni Next.js.** El módulo `ag-ui` ofrece SSR + HTMX para casos donde un stack JS completo es excesivo, pero no compite con frameworks frontend establecidos. Para aplicaciones SPA o SSR ricas, el patrón recomendado es Anti-Gravital como backend + Next.js (o equivalente) como frontend, comunicándose vía cliente TypeScript generado.
- **No reemplaza Docker.** Genera Dockerfiles. Se ejecuta en contenedores. No reinventa el formato OCI.
- **No reemplaza PostgreSQL, Redis, MinIO ni NATS.** Se integra con ellos como dependencias externas estándar.
- **No reemplaza Terraform ni Pulumi.** `ag-cloud` orquesta despliegues simples; para infraestructura compleja multi-cloud con políticas, IaC declarativa y módulos compartidos, Terraform sigue siendo la herramienta correcta. `ag-domains` (Fase 4.5) tampoco reemplaza Terraform: orquesta DNS y TLS para los dominios declarados en el `schema.ag` del proyecto, no gestiona zonas DNS arbitrarias ni infraestructura compartida.
- **No es un servidor de correo completo.** `ag-mail` (Fase 4.5) envía correo transaccional outbound (verificación, recuperación, magic links, alertas) vía SMTP nativo o relay SMTP nativo (apuntable a cualquier proveedor externo). NO es un MTA, NO recibe correo (sin IMAP/POP), NO ofrece buzones, NO implementa antispam ni gestión de reputación de IP. Para inbound o un servidor de correo completo, usar Postfix, Stalwart u otro proyecto especializado.
- **No es un registrador de dominios.** `ag-domains` (Fase 4.5) consume el dominio que el operador ya compró (Namecheap, Cloudflare Registrar, etc.) y lo configura mediante un adapter (Cloudflare inicialmente). NO registra dominios, NO actúa como mercado de dominios.
- **No es un motor de juegos, ni un framework de cómputo científico, ni una alternativa a Unreal Engine, Unity, NumPy, PyTorch o TensorFlow.** Estos dominios tienen herramientas especializadas que Anti-Gravital no intenta replicar.

### 3.3 La regla de interoperabilidad

Cuando exista una herramienta dominante en un dominio adyacente, la estrategia es integrar, no reemplazar. Esta regla evita que el proyecto crezca en direcciones inmanejables y mantiene el alcance defendible.

---

## 4. Análisis del estado del arte

Esta sección documenta el contexto competitivo en términos técnicos, sin retórica adversarial. Cada framework analizado resuelve un conjunto real de problemas; el análisis identifica las limitaciones estructurales que Anti-Gravital pretende abordar.

### 4.1 Spring Boot y el ecosistema JVM

Spring Boot domina el desarrollo empresarial Java y Kotlin con dos décadas de ecosistema maduro. Sus debilidades estructurales derivan de la JVM: un consumo de memoria base de 256–512 MB antes de servir el primer request, tiempos de arranque de 6–8 segundos incompatibles con cómputo serverless, y verbosidad de configuración. GraalVM Native Image mitiga parcialmente el arranque y la memoria, pero introduce sus propias limitaciones (reflexión limitada, compatibilidad incompleta de librerías, tiempos de compilación largos). El compromiso fundamental — un runtime gestionado con GC — permanece.

### 4.2 ASP.NET Core y .NET

Técnicamente uno de los frameworks gestionados más rápidos del mercado, con C# moderno y expresivo. CLR con GC mantiene pausas medibles en p99 bajo carga sostenida. La dirección técnica del ecosistema es unilateral de Microsoft. La seguridad de memoria no está garantizada por el compilador; los bugs de race conditions y null reference exceptions son posibles. La adopción fuera del ecosistema Microsoft sigue siendo limitada por razones culturales más que técnicas.

### 4.3 Django y FastAPI

Django mantiene la mejor experiencia de prototipado del mundo Python, con un ecosistema rico para administración, autenticación y plantillas. FastAPI elevó el estándar de DX en APIs Python con tipos Pydantic, generación automática de OpenAPI y soporte async nativo. Ambos comparten el techo estructural de CPython: el Global Interpreter Lock impide concurrencia real CPU-bound dentro de un proceso, lo que obliga a escalar con múltiples procesos (Gunicorn, Uvicorn workers) multiplicando el consumo de memoria. El soporte async de Django sigue siendo parcial; muchas librerías del ecosistema permanecen sincrónicas.

### 4.4 Node.js, Express y NestJS

Node.js trajo JavaScript al servidor y el ecosistema npm es el más amplio de la industria. El event loop monohilo de V8 es óptimo para I/O concurrente pero se degrada con cualquier trabajo CPU-bound. La cadena de suministro npm es crónicamente vulnerable: la dependencia transitiva media de un proyecto Node.js moderno excede las 200 librerías, y los incidentes de paquetes comprometidos son recurrentes. TypeScript añade seguridad de tipos en desarrollo, pero en runtime sigue siendo JavaScript.

### 4.5 Next.js y los frameworks fullstack JS

Next.js representa la convergencia frontend/backend en JavaScript. Server Components y Server Actions reducen el boilerplate de APIs internas. Las debilidades estructurales son herencia de Node.js: cold starts en serverless, acoplamiento de facto con Vercel, inadecuación para WebSockets persistentes, estado compartido y procesamiento de larga duración. Next.js es una excelente capa de presentación; no es un backend robusto.

### 4.6 Axum, Actix-Web, Rocket (Rust)

Los frameworks Rust actuales son técnicamente excelentes en rendimiento (top 10 de TechEmpower de forma consistente) pero ofrecen lo que la comunidad llama una experiencia *low-level*: el desarrollador construye desde cero la autenticación, la capa de datos, la observabilidad, la generación de clientes y el sistema de migraciones. Anti-Gravital se construye sobre Axum, Tokio y Tower como dependencias internas — no compite con ellos, sino que los empaqueta en una experiencia de framework completo con DSL, CLI y módulos opinados.

### 4.7 Conclusión del análisis

Existe un espacio de mercado real: un framework Rust enterprise-grade dominante todavía no existe. Spring Boot paga el costo histórico de la JVM. Node.js tiene límites estructurales de event loop. Python tiene problemas de concurrencia. Go sacrifica el sistema de tipos. Rust tiene runtime y rendimiento, pero le falta una experiencia de framework completa. Anti-Gravital pretende llenar ese hueco.

---

## 5. Arquitectura del ecosistema: módulos y responsabilidades

La decisión arquitectónica más importante derivada del análisis crítico del v3.0 fue separar el núcleo de los ecosistemas. El v3.0 intentaba ser simultáneamente framework backend, motor SSR, plataforma DevOps, orquestador AI, capa de observabilidad, framework móvil y sistema de plugins. Esto es inmanejable. La v4.0 reorganiza el proyecto como un ecosistema de crates Rust independientes, cada uno con un dominio propio, un mantenedor responsable, versionado semántico independiente y una superficie de API mínima.

### 5.1 Mapa del ecosistema

| Crate              | Dominio                                                          | Estado de criticidad |
|--------------------|------------------------------------------------------------------|----------------------|
| `ag-core`          | Runtime HTTP, router, extractores, error types, Shield/Core      | Núcleo               |
| `ag-dsl`           | Lexer, parser, AST, análisis semántico y codegen del Anti-DSL    | Núcleo               |
| `ag-cli`           | Binario `ag`: new, generate, dev, build, deploy, migrate         | Núcleo               |
| `ag-auth`          | WebAuthn, JWT Ed25519, OAuth2, RBAC, rate limiting               | Estándar             |
| `ag-data`          | sqlx con verificación compile-time, migraciones, ORM tipado      | Estándar             |
| `ag-realtime`      | WebSocket, SSE, NATS embebido, pub/sub                           | Estándar             |
| `ag-cache`         | moka en memoria, adaptador Redis, invalidación por evento        | Estándar             |
| `ag-storage`       | S3, MinIO, filesystem local, URLs firmadas, procesamiento imagen | Estándar             |
| `ag-observe`       | tracing, OpenTelemetry, Prometheus, dashboards Grafana           | Estándar             |
| `ag-mail`          | SMTP outbound, templates tipados, colas de envío con reintentos, relay SMTP nativo, helpers SPF/DKIM/DMARC | Estándar diferido |
| `ag-ui`            | SSR con askama, hidratación selectiva, integración HTMX          | Opcional             |
| `ag-cloud`         | Orquestación de despliegue Railway-like, Dockerfile gen          | Opcional             |
| `ag-domains`       | Gestión DNS vía trait `DnsProvider`, adapters (Cloudflare), certificados ACME, dominios de despliegue | Opcional infra |
| `ag-ai`            | Doc generation, schema suggestions, knowledge graph              | Opcional             |
| `ag-mobile`        | Generación SDK Dart, auth nativo Flutter, offline sync           | Opcional             |
| `ag-migrate`       | Importadores OpenAPI, Prisma, Django, FastAPI, Sequelize         | Opcional             |
| `ag-wasm-host`     | Runtime de plugins WASI sobre wasmtime                           | Núcleo               |

La distinción entre **núcleo**, **estándar**, **estándar diferido** y **opcional** es importante. El núcleo es el conjunto mínimo que define lo que es Anti-Gravital. Los módulos estándar cubren el 90% de las necesidades de producción de cualquier servicio backend y se instalan por defecto en los templates oficiales. Un módulo **estándar diferido** (introducido por `ADR-0007`) tiene la madurez y el alcance de un estándar pero NO se instala por defecto en los templates: se incorpora cuando el proyecto lo necesita explícitamente. `ag-mail` es estándar diferido porque la mayoría de los backends acaba enviando correo transaccional (verificación, recuperación, magic links vía `ag-auth`), pero no todo proyecto lo usa desde el minuto cero. Los módulos opcionales se añaden cuando el proyecto los necesita; `ag-domains` es opcional de infraestructura (lo consume `ag-cloud` durante el despliegue) y `ag-cloud → ag-domains` es una dependencia documentada en la sección 5.3. El ecosistema pasa de **17 crates** con la introducción de la Fase 4.5.

### 5.2 Diagrama del ecosistema

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

### 5.3 Reglas de dependencia entre crates

Para mantener el ecosistema sano se aplican reglas estrictas de dependencia:

Primera regla: `ag-core` no depende de ningún otro crate del ecosistema Anti-Gravital. Es la base sobre la que todo lo demás se construye. Cualquier funcionalidad considerada genérica suficientemente que necesite otro módulo debe extraerse a `ag-core` o convertirse en un trait que el módulo implementa.

Segunda regla: los módulos estándar pueden depender de `ag-core` y de otros módulos estándar siempre que no haya ciclos. Por ejemplo, `ag-auth` puede depender de `ag-data` para persistencia de sesiones, pero `ag-data` no puede depender de `ag-auth`.

Tercera regla: los módulos opcionales pueden depender de cualquier crate núcleo o estándar. No pueden depender entre sí salvo casos explícitamente justificados (por ejemplo, `ag-mobile` puede depender de `ag-ai` para generación de código asistida).

Cuarta regla: `ag-cli` depende de todos los demás crates (es el orquestador), pero solo a través de features Cargo, de modo que el binario `ag` puede compilarse con un subconjunto reducido.

Quinta regla: todos los crates publican versiones semánticas independientes. Una breaking change en `ag-cache` no fuerza a `ag-core` a subir mayor. Esto es esencial para la sostenibilidad de un proyecto open source.

Sexta regla (introducida por `ADR-0007`, Fase 4.5): la dirección de la
dependencia `ag-auth ↔ ag-mail` es estrictamente unidireccional. `ag-auth`
**consume** `ag-mail` para enviar correos de verificación, recuperación de
contraseña y magic links, definiendo un trait pequeño que `ag-auth` invoca.
`ag-mail` **NO** depende de `ag-auth`. Esta direccionalidad preserva la
segunda regla (no ciclos) y mantiene a `ag-mail` reusable de forma aislada
en cualquier proyecto Rust. La cooperación `ag-mail ↔ ag-domains` (para
materializar SPF/DKIM/DMARC) es opcional, vía feature de Cargo: si un
proyecto usa `ag-mail` con un proveedor externo (via SMTP) y no administra
DNS propio, `ag-domains` no es necesario.

Séptima regla (introducida por `ADR-0007`, Fase 4.5): el módulo opcional
`ag-cloud` **consume** `ag-domains` durante `ag deploy` para configurar DNS
y TLS, sin que la dependencia sea rígida en todos los targets. Si el
proyecto no declara dominios en su `schema.ag`, el flujo se omite.
`ag-domains` puede usarse de forma independiente desde la CLI sin `ag-cloud`.

### 5.4 Estructura del monorepo

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
│   ├── ag-auth/
│   ├── ag-data/
│   ├── ag-realtime/
│   ├── ag-cache/
│   ├── ag-storage/
│   ├── ag-observe/
│   ├── ag-mail/                # Fase 4.5 — estándar diferido
│   ├── ag-ui/
│   ├── ag-cloud/
│   ├── ag-domains/             # Fase 4.5 — opcional infra
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

## 6. Arquitectura del núcleo (`ag-core`): Shield y Core

El núcleo de Anti-Gravital se organiza en dos capas conceptuales dentro de un único proceso Rust. La separación no es física: no hay IPC, no hay FFI, no hay shared memory entre runtimes. Las dos capas se comunican mediante llamadas de función Rust ordinarias, con cero overhead medible. La separación es lógica y existe por dos razones: claridad arquitectónica para el desarrollador, y posibilidad futura de extraer la Shield como gateway independiente si un caso de uso lo justifica.

### 6.1 Diagrama del núcleo

```
┌─────────────────────────────────────────────────────────────┐
│                  ag-core · Single Process                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌─────────────────────────────────────────────────┐       │
│   │            CAPA A — The Shield                  │       │
│   │  (Tower middleware composable pipeline)         │       │
│   │                                                 │       │
│   │   ┌─────────┐  ┌─────────┐  ┌─────────────┐     │       │
│   │   │ TLS 1.3 │─►│   JWT   │─►│ Rate Limit  │     │       │
│   │   │ rustls  │  │ Ed25519 │  │  governor   │     │       │
│   │   └─────────┘  └─────────┘  └──────┬──────┘     │       │
│   │                                    │            │       │
│   │   ┌─────────┐  ┌─────────┐  ┌──────▼──────┐     │       │
│   │   │  CORS   │◄─│  RBAC   │◄─│ Validación  │     │       │
│   │   │  CSRF   │  │ Guards  │  │  Schema     │     │       │
│   │   └─────────┘  └─────────┘  └─────────────┘     │       │
│   └─────────────────────┬───────────────────────────┘       │
│                         │                                   │
│              Llamada de función Rust (0ns)                  │
│                         ▼                                   │
│   ┌─────────────────────────────────────────────────┐       │
│   │            CAPA B — The Core                    │       │
│   │  (Axum router · Handlers · Estado)              │       │
│   │                                                 │       │
│   │   ┌────────────┐   ┌─────────────────────┐      │       │
│   │   │  Router    │   │  Business handlers  │      │       │
│   │   │  Axum      │──►│  (generados por DSL)│      │       │
│   │   └────────────┘   └──────────┬──────────┘      │       │
│   │                               │                 │       │
│   │   ┌────────────┐   ┌──────────▼──────────┐      │       │
│   │   │ Extractores│   │  Estado compartido  │      │       │
│   │   │  tipados   │   │  AppState           │      │       │
│   │   └────────────┘   └─────────────────────┘      │       │
│   └─────────────────────────────────────────────────┘       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
                            │
                  cargo build --release
                            ▼
                ┌────────────────────────┐
                │  Single Static Binary  │
                │  FROM scratch Docker   │
                └────────────────────────┘
```

### 6.2 The Shield: la capa de confianza

La Shield es responsable de todo lo que ocurre antes de que un request sea considerado confiable y entregado al código de negocio. Está implementada como una pipeline de capas Tower, el mismo modelo composable que Axum usa internamente. Cada capa es opcional y se configura desde el `schema.ag` del proyecto.

El stack técnico de la Shield es: Tokio como runtime async M:N (multiplexa millones de tareas sobre un thread pool fijo de tamaño igual a CPUs disponibles), Tower como modelo de middleware composable, rustls para TLS 1.3 sin dependencia de OpenSSL, serde y serde_json para serialización zero-copy donde es posible, ring para primitivas criptográficas de bajo nivel, governor para rate limiting con algoritmo token bucket sin locks contenciosos.

Las capas estándar de la Shield, en orden de ejecución sobre un request entrante:

La primera capa es la terminación TLS, gestionada por rustls. Soporta TLS 1.3 con cipher suites modernas, OCSP stapling y ALPN para negociación HTTP/1.1 vs HTTP/2. Para entornos donde la terminación TLS la realiza un balanceador externo (Cloudflare, AWS ALB, Nginx), esta capa se desactiva con una opción en el schema.

La segunda capa es la deserialización y validación del payload. Para requests con body, se aplica el contrato definido en el `.ag`: tipos, restricciones de longitud, formato de email, regex, rangos numéricos. Una violación produce un error 422 con detalle estructurado de qué campo falló y por qué.

La tercera capa es la autenticación. Soporta JWT firmado con Ed25519 (curva Edwards25519, más rápida y segura que RS256), Passkeys/WebAuthn (FIDO2), API keys, y sesiones cookie-based. La verificación es eager para endpoints marcados como `auth required`.

La cuarta capa es el rate limiting. Implementado con governor sobre algoritmo token bucket, soporta límites por IP, por usuario autenticado, por endpoint, y por combinaciones. Los límites se declaran en el schema.

La quinta capa es la autorización RBAC. Las políticas se declaran en el `.ag` como expresiones que se evalúan contra los claims del JWT y los parámetros del request. Por ejemplo: `policy "user.role == ADMIN || user.id == params.id"`.

La sexta capa es CORS y CSRF. Configurada por defecto con valores seguros (no wildcard); cualquier desviación requiere declaración explícita.

### 6.3 The Core: la capa de lógica de negocio

The Core es donde vive el 80% del código de aplicación que el desarrollador escribe. Es Axum con una capa fina de convenciones encima.

Los handlers tienen una firma generada por el compilador del DSL a partir del endpoint declarado:

```rust
// Generado automáticamente por `ag generate` desde schema.ag
// El desarrollador solo escribe el cuerpo del handler.
pub async fn create_user(
    State(state): State<AppState>,
    ValidatedBody(req): ValidatedBody<CreateUserRequest>,
    Claims(claims): Claims<AuthClaims>,
) -> Result<Json<User>, AgError> {
    // El desarrollador solo escribe esto:
    let user = state.db.users()
        .create(CreateUserParams {
            email: req.email,
            name: req.name,
            created_by: claims.user_id,
        })
        .await?;

    state.events.emit("user.created", &user).await?;
    Ok(Json(user))
}
```

El tipo `ValidatedBody<T>` garantiza que el body ya pasó la validación de la Shield. El tipo `Claims<T>` garantiza que el JWT ya fue verificado. El tipo `AgError` es un enum que cubre todos los errores declarados en el endpoint, y la conversión a respuesta HTTP es automática vía `IntoResponse`.

El estado de la aplicación (`AppState`) es un struct generado que contiene clientes a los recursos del proyecto: el pool de base de datos, el cliente NATS, el cliente Redis, el cliente S3. Se construye en el arranque del binario y se comparte por referencia (clones baratos de `Arc`) entre todos los handlers.

### 6.4 Manejo de errores

El sistema de errores de Anti-Gravital sigue tres principios. El primero: cada endpoint declara explícitamente qué errores puede producir en su definición `.ag`. Esto produce un enum `EndpointError` tipado del que cada variante es un error específico. El segundo: los errores se propagan con el operador `?` de Rust, y la conversión a respuesta HTTP es automática y consistente. El tercero: ningún error se descarta silenciosamente. Los errores no esperados producen un 500 estructurado con un correlation ID que se enlaza al stack trace en el sistema de tracing.

### 6.5 Runtime y configuración Tokio

Anti-Gravital usa Tokio en modo multi-thread con configuración por defecto: un worker por CPU disponible, blocking pool de 512 threads. Para cargas IO-bound estándar esta configuración es óptima. El schema permite ajustes:

```yaml
runtime:
  workers: auto              # Por defecto = núm CPUs
  blocking_threads: 512
  thread_stack: 2MB
  shutdown_timeout: 30s
```

---

## 7. El lenguaje Anti-DSL (`ag-dsl`): especificación e implementación incremental

El compilador del DSL es, junto con el runtime, el componente técnicamente más exigente del proyecto. Es esencialmente un compilador completo: lexer, parser, análisis semántico, sistema de tipos, generación de código a múltiples targets, formatter, linter, y servidor LSP. Esta sección define la especificación del lenguaje y la estrategia de implementación incremental.

### 7.1 Filosofía del lenguaje

El Anti-DSL es declarativo, no imperativo. Describe contratos, no flujos. La premisa es que la mayor parte del valor de un framework backend reside en la consistencia de su contrato externo: qué modelos existen, qué endpoints los exponen, qué reglas se aplican, qué errores se devuelven. La lógica interna queda en Rust puro.

El lenguaje se inspira en Prisma para la sintaxis de modelos, en GraphQL SDL para la claridad de las definiciones, en sqlc para la integración con SQL y en protobuf para el codegen multi-target. No es un lenguaje Turing-completo y no pretende serlo.

### 7.2 Implementación incremental por versiones del DSL

Probablemente la decisión más importante para que el compilador sea viable es admitir que no se puede entregar el lenguaje completo en la primera versión. La especificación se entrega en fases incrementales, cada una con una gramática estable que no rompe la anterior. Las versiones del DSL son independientes de las versiones del framework y siguen su propio semver.

| Versión DSL | Capacidad gramatical                                                                                              | Hito                       |
|-------------|-------------------------------------------------------------------------------------------------------------------|----------------------------|
| v0.1        | Modelos básicos: campos, tipos primitivos, anotaciones `@primary`, `@unique`, `@auto`                              | Fin Fase 3 (entregado)     |
| v0.2        | Endpoints: método, path, body, response, errors                                                                    | Fin Fase 3 (entregado)     |
| v0.3        | Validaciones: `@min`, `@max`, `@email`, `@regex`, `@length`                                                        | Fin Fase 3 (entregado)     |
| v0.4        | Relaciones entre modelos: `1:1`, `1:N`, `N:M`, cascadas                                                            | Fin Fase 3 (entregado)     |
| v0.5        | Autenticación y autorización: `auth required`, `policy "..."`                                                      | Fin Fase 4 (entregado)     |
| v0.6        | Eventos: declaración de eventos emitidos por endpoint, suscriptores                                                | Fin Fase 4 (entregado)     |
| v0.7        | Mail y dominios declarativos: `mail`, `domain`, `dns`, `tls`                                                       | Fin Fase 4.5               |
| v0.8        | Plugin hooks (lifecycle, decoradores)                                                                              | Fin Fase 9                 |
| v1.0        | Gramática estable, congelada bajo semver. Cualquier extensión posterior será aditiva.                              | Fin Fase 10                |

Esta tabla está realineada por `ADR-0007` (Fase 4.5). Las capacidades de
multi-tenancy y migración de datos previstas para versiones intermedias del
DSL en revisiones anteriores quedan diferidas: se especificarán en RFCs
propios cuando el alcance lo justifique, sin ocupar un slot numerado fijo
hasta entonces. Esto evita prometer features que no tienen tracción
verificada.

### 7.3 Ejemplo completo de schema (v1.0 target)

```ag
# schema.ag — Ejemplo completo objetivo de la v1.0

config {
    project_name "fintech-api"
    database "postgres"
    runtime { workers auto, blocking_threads 512 }
}

enum UserRole {
    ADMIN
    USER
    BANNED
}

model User {
    id           UUID       @primary @auto
    email        String     @unique @max(255) @email
    password_hash String    @max(255)
    name         String     @max(100) @min(2)
    role         UserRole   @default(USER)
    created      Timestamp  @auto
    updated      Timestamp  @auto_update

    accounts     Account[]  @relation(name: "owner")
}

model Account {
    id        UUID      @primary @auto
    owner_id  UUID      @references(User.id) @on_delete(cascade)
    balance   Decimal   @precision(18,2) @default(0)
    currency  String    @length(3)
    created   Timestamp @auto
}

request CreateUserRequest {
    email     String @email
    password  String @min(12) @max(128)
    name      String @min(2) @max(100)
}

response UserResponse {
    id        UUID
    email     String
    name      String
    role      UserRole
    created   Timestamp
}

error EmailTaken      { status 409 message "Email already registered" }
error WeakPassword    { status 422 message "Password does not meet policy" }
error InsufficientFunds { status 402 message "Insufficient balance" }

endpoint CreateUser {
    method   POST
    path     /users
    auth     optional
    body     CreateUserRequest
    response UserResponse
    errors   [EmailTaken, WeakPassword]
    events   [user.created]
    rate_limit "5/min per_ip"
}

endpoint TransferFunds {
    method   POST
    path     /accounts/{from}/transfer
    auth     required
    policy   "user.id == params.from.owner_id"
    body     TransferRequest
    response TransferReceipt
    errors   [InsufficientFunds]
    events   [account.debited, account.credited, transfer.completed]
    rate_limit "10/min per_user"
}

event user.created {
    payload UserResponse
    retain  30d
}

event account.debited {
    payload AccountDelta
    retain  7y
}
```

### 7.4 Artefactos generados desde un solo schema

Un único `schema.ag` produce, mediante `ag generate`:

| Artefacto                          | Ruta                                | Propósito                                    |
|------------------------------------|-------------------------------------|----------------------------------------------|
| Structs Rust con serde y validators | `src/models.rs`                     | Tipos del dominio                            |
| Stubs de handlers tipados          | `src/handlers/*.rs`                 | Firmas listas; el dev escribe el cuerpo      |
| Queries sqlx compile-time checked  | `src/db/queries.rs`                 | Acceso a base de datos type-safe             |
| Migraciones SQL versionadas        | `migrations/NNNN_*.sql`             | Esquema de base de datos                     |
| Tipos TypeScript                   | `clients/typescript/types.ts`       | Tipos compartidos con frontend               |
| Cliente TypeScript HTTP            | `clients/typescript/client.ts`      | SDK tipado para frontend                     |
| Tipos Dart                         | `clients/dart/lib/types.dart`       | Tipos compartidos con aplicaciones Flutter   |
| Cliente Dart con Dio               | `clients/dart/lib/client.dart`      | SDK tipado para Flutter                      |
| Documentación OpenAPI 3.1          | `openapi.yaml`                      | Documentación interactiva (Swagger UI)       |
| Especificación AsyncAPI            | `asyncapi.yaml`                     | Documentación de eventos                     |
| Grafo de conocimiento JSON         | `.ag/knowledge-graph.json`          | Insumo para `ag-ai` y dashboards             |

### 7.5 Arquitectura del compilador

El compilador del DSL se organiza en pipeline tradicional con etapas bien definidas:

La fase de **lexer** tokeniza la entrada en `.ag`. Está implementada con `logos` (crate Rust que genera tokenizers a partir de definiciones declarativas con derive macros). Produce un stream de tokens posicionales para reporte de errores con líneas y columnas.

La fase de **parser** consume tokens y produce un AST. Está implementada con `chumsky` (parser combinator library con soporte de recuperación de errores), elegida sobre `nom` por su mejor manejo de errores legibles para el usuario final.

La fase de **análisis semántico** valida el AST: comprueba que las referencias entre modelos existan, que los tipos sean consistentes, que las políticas RBAC se refieran a campos válidos, que no haya ciclos en las relaciones, que los nombres no colisionen con palabras reservadas de Rust o SQL. Produce diagnósticos estructurados con sugerencias.

La fase de **codegen** toma el AST validado y emite código a múltiples targets. Cada target (Rust, TypeScript, Dart, OpenAPI, SQL) es un módulo independiente. La emisión se hace con templates de `askama` para los outputs textuales y con `quote` para el código Rust (que se beneficia de tener un AST nativo Rust para emisión).

### 7.6 Servidor de lenguaje (LSP)

Desde la versión 0.3 del DSL se incluye un servidor LSP que ofrece autocompletado, diagnostics en vivo, go-to-definition, find-references, hover types y rename. Se distribuye como un binario `ag-lsp` y se integra con cualquier editor compatible con el protocolo (VS Code, Neovim, Helix, Zed, IntelliJ vía plugin).

El plugin oficial para VS Code se publica en el marketplace bajo el nombre `Anti-Gravital`.

### 7.7 Herramientas del DSL

La CLI ofrece tres comandos específicos para el DSL:

`ag schema lint` revisa el archivo `.ag` y reporta warnings sobre malas prácticas (modelos sin índices en campos foreign key, endpoints sin rate limit, políticas tautológicas, errores no manejados).

`ag schema diff <ref>` compara el schema actual contra una referencia (commit git, tag, archivo) y reporta cambios breaking vs no-breaking. Esencial para revisiones de pull request.

`ag schema migrate` genera la migración SQL necesaria para llevar la base de datos del estado actual al estado del schema. Incluye análisis de seguridad: detecta operaciones destructivas (drop column, drop table) y exige confirmación explícita.

---

## 8. Módulos batteries-included

Esta sección especifica cada uno de los módulos estándar del ecosistema. Cada subsección documenta el propósito, el stack técnico, las decisiones de diseño y los puntos de extensión.

### 8.1 `ag-auth` — Autenticación y autorización

El módulo de autenticación implementa los esquemas modernos de identidad. La decisión arquitectónica central es soportar Passkeys/WebAuthn como primera clase, no como afterthought; las passwords son un mecanismo legacy soportado pero no recomendado.

El stack técnico implementado es WebAuthn propio con `ciborium` (CBOR) y verificación COSE (`p256` para ES256, `ed25519-dalek` para EdDSA) en lugar de `webauthn-rs`, por compatibilidad de licencia Apache-2.0 (vease `ADR-0006`); `jsonwebtoken` para JWT, `argon2` para hashing de passwords (cuando se usan), y `oauth2` como cliente OAuth2 (Google, GitHub) con flujo PKCE.

Los flujos soportados son: registro y autenticación con passkey, autenticación con email + password (legacy), OAuth2 con providers preconfigurados (Google, GitHub, Microsoft, Gravital ID), API keys para integraciones servidor-servidor, y refresh tokens con rotación.

Los JWT se firman con Ed25519 por defecto (curva Edwards25519, más rápida que RSA y más segura que ECDSA-P256 contra ataques de canal lateral). La clave privada vive en un secret manager externo (HashiCorp Vault, AWS Secrets Manager, GCP Secret Manager) o en variables de entorno con rotación documentada.

El RBAC se declara en el schema y se compila a expresiones evaluables. La política se evalúa una vez por request en la Shield, antes de llegar al handler. Las políticas pueden referenciar claims del JWT, parámetros de path, y consultar la base de datos si se declara explícitamente (con cache para evitar el N+1).

### 8.2 `ag-data` — Acceso a datos y migraciones

El módulo de datos se construye sobre sqlx, con verificación de queries SQL en tiempo de compilación. Esto significa que cuando se ejecuta `cargo build`, sqlx conecta a una base de datos de desarrollo (configurable por variable de entorno) y verifica que cada query sea sintácticamente válida y que los tipos de columnas devueltas coincidan con los structs Rust que las reciben. Un error de SQL deja de ser un error de runtime; se convierte en un error de compilación.

Los backends soportados son PostgreSQL (recomendado para producción), SQLite (para desarrollo, tests, y aplicaciones edge), y MySQL (para entornos heredados).

Las migraciones se embeben en el binario con `sqlx::migrate!`. Esto significa que el binario contiene en sí mismo el historial completo de migraciones, y al arrancar puede aplicar automáticamente las pendientes. Para entornos donde esto no es deseable (despliegues blue-green con migración como step separado), el comando `ag migrate apply` ejecuta las migraciones sin levantar el servidor.

Para arquitecturas multi-tenant, `ag-data` soporta nativamente schema-per-tenant en PostgreSQL: cada inquilino tiene su propio schema con las mismas tablas, y el router de conexión selecciona el schema en función del claim del JWT. También soporta Row-Level Security (RLS) para casos donde el aislamiento por schema es excesivo.

Las read replicas se configuran declarativamente; el módulo enruta queries de solo lectura al replica más cercano y queries de escritura al primario.

### 8.3 `ag-realtime` — Eventos y comunicación en tiempo real

`ag-realtime` ofrece tres modalidades de comunicación bidireccional: WebSocket binario, Server-Sent Events para flujos unidireccionales, y un bus de eventos pub/sub.

El bus de eventos usa NATS como broker. Para casos pequeños, NATS se ejecuta embebido en el mismo binario Anti-Gravital (modo edge). Para casos a escala, el binario se conecta a un clúster NATS externo. Esta dualidad permite arrancar simple y escalar sin reescribir.

Para WebSocket, el protocolo binario interno (basado en msgpack) reduce el overhead frente a JSON. Los handlers de WebSocket se declaran en el schema y reciben mensajes ya deserializados a structs Rust.

Para SSE, se usa como fallback automático en navegadores que no soportan WebSocket o están detrás de proxies que lo bloquean. La negociación es transparente.

La persistencia de eventos usa JetStream (componente de NATS) cuando está disponible, lo que permite replay de eventos para nuevos consumidores y durabilidad ante caídas del broker.

### 8.4 `ag-cache` — Caché multinivel

El módulo de caché ofrece dos niveles. El nivel L1, ya implementado, es caché en memoria con `moka`, una implementación concurrente sin locks contenciosos basada en TinyLFU, con invalidación por tags. El nivel L2 (caché distribuida entre instancias) aún no está implementado: `RFC-0005` propone un L2 nativo compatible con el protocolo RESP2, sin dependencia de Redis como servicio externo, y queda pendiente de aprobación e implementación.

La invalidación se hace por eventos. Cuando un endpoint emite un evento (`user.updated`), `ag-cache` invalida automáticamente las entradas relacionadas en ambos niveles. La política de invalidación se declara en el schema.

El caché de queries SQL es automático: las queries marcadas con `@cache(ttl: 5m)` en el schema se cachean transparentemente, y la invalidación se dispara cuando un evento toca alguna de las tablas involucradas.

### 8.5 `ag-storage` — Almacenamiento de objetos

`ag-storage` ofrece una abstracción sobre tres backends: S3 (AWS y compatibles), MinIO (self-hosted), y filesystem local (para desarrollo). El backend se selecciona por configuración; el código de aplicación no se entera.

Las URLs firmadas para descarga y subida directa se generan con un solo call: `storage.signed_url(key, Duration::from_mins(15), Permission::Write)`.

El procesamiento de imágenes (resize, compress, format conversion) se hace con el crate `image`, soportando JPEG, PNG, WebP y AVIF. Los thumbnails se generan automáticamente en upload si se declara la política en el schema.

### 8.6 `ag-observe` — Trazabilidad, métricas y logging

La observabilidad es una preocupación de primer nivel y no un módulo opcional para producción. Su stack es `tracing` para spans estructurados, `opentelemetry-rust` para exportación a backends compatibles (Jaeger, Tempo, Datadog, Honeycomb), `metrics` para métricas con backend Prometheus, y dashboards Grafana pre-configurados que se incluyen como JSON en el repositorio.

Cada request atraviesa todo el sistema con un correlation ID único que aparece en todos los logs estructurados, todos los spans de tracing, y todos los errores devueltos al cliente. Esto resuelve el problema del debugging en producción: dado un ticket de soporte con un correlation ID, el operador puede reconstruir el camino completo del request.

`tokio-console` se integra en modo desarrollo para inspección en vivo de las tareas Tokio.

### 8.7 `ag-ui` — Server-Side Rendering opcional

El módulo SSR existe para casos donde un frontend SPA es excesivo: dashboards internos, páginas de marketing, formularios simples, e interfaces administrativas. Está basado en `askama` (templating compilado en build time, con tipos verificados) e integración nativa con HTMX para interactividad sin frameworks JavaScript pesados.

Este módulo es explícitamente *no* un competidor de React, Vue, Svelte o Next.js. Para aplicaciones SPA o SSR ricas, el patrón recomendado es Anti-Gravital como backend con un frontend Next.js (u otro) que consume el cliente TypeScript generado.

### 8.8 `ag-mail` — Comunicación transaccional (estándar diferido)

Introducido por `ADR-0007` en la Fase 4.5. `ag-mail` es un módulo estándar
**diferido**: tiene la madurez y el alcance de un estándar, pero NO se instala
por defecto en los templates oficiales. Se incorpora cuando el proyecto
requiere correo transaccional outbound (verificación de cuentas, magic links,
recuperación de contraseña, alertas, notificaciones).

El alcance v1 es **exclusivamente outbound**. `ag-mail` NO es un MTA, NO
recibe correo (sin IMAP/POP), NO ofrece buzones persistentes, NO implementa
antispam, filtrado ni gestión de reputación de IP. Esta restricción es
deliberada y está fijada en el ADR: las capacidades inbound y de servidor
de correo completo son trabajo de un proyecto distinto, no de Anti-Gravital.

> **Actualización de alcance (`ADR-0010`, 2026-06-03).** La restricción "NO es
> un MTA / inbound nunca" de arriba queda **superseded**: `ag-mail` se expande
> a un MTA outbound nativo (resolución MX, entrega ESMTP+STARTTLS, firma DKIM,
> clasificación de bounces) para enviar correo autenticado sin terceros en la
> ruta de envío. La expansión es por fases y opt-in tras features de Cargo
> (`mta`, `api`, `queue-jetstream`); el baseline de relay outbound descrito
> aquí sigue siendo el modo por defecto y no cambia. Buzones, IMAP/POP/JMAP e
> inbound general siguen fuera de alcance; inbound se admite solo como parsing
> de DSN/ARF para procesar bounces. Plan técnico: `RFC-0009`. El MTA nativo es
> trabajo futuro y no se declara implementado aquí.

El stack técnico es `lettre` con transporte async Tokio y `rustls` para el
sender SMTP nativo (coherente con The Shield). Los adapters de proveedor se
se alcanzan apuntando el relay SMTP nativo a su endpoint; la via sin terceros es el MTA nativo (`mta`). El trait `MailSender` abstrae ambos. Cada
uno implementa el mismo trait `MailSender`:

```rust
#[async_trait::async_trait]
pub trait MailSender: Send + Sync {
    async fn send(&self, msg: &Email) -> Result<MessageId, AgMailError>;
    fn provider_name(&self) -> &'static str;
    fn dns_requirements(&self, domain: &str) -> Vec<DnsRecordSpec>;
}

pub enum AgMail {
    Native(SmtpSender),                // lettre + rustls
    Adapter(Box<dyn MailSender>),      // external providers via SMTP
}
```

El patrón Native | Adapter es idéntico al usado por `ag-storage`
(`Native | S3`) y al previsto para el L2 de `ag-cache` (L1 `moka` nativo
hoy; L2 RESP2 nativo propuesto en `RFC-0005`), reforzando la regla de
interoperabilidad del proyecto: integrar proveedores dominantes, no
reemplazarlos.

Los **templates** se modelan con el trait `MailTemplate` y una
implementación `StringTemplate` de sustitución `{{var}}`; cualquier motor
externo (askama, minijinja) puede conectarse implementando el trait. La
validación de variables se hace contra el `schema.ag`: el compilador del
DSL emite un aviso cuando el `from` de un bloque `mail` no referencia un
`domain` declarado, y verifica que las `vars` tipadas del template
coincidan con los marcadores usados. Un correo mal formado deja de ser un
bug de runtime y se acerca a un error detectable en build. Este es el
**diferenciador real** (correctitud en build-time), no la entregabilidad: la
entregabilidad es trabajo del proveedor; la corrección del contrato es
trabajo del framework.

La **cola asíncrona** acepta jobs con reintentos y backoff exponencial.
Backend por defecto en memoria (Tokio task + canal). Backend opcional
persistente vía `ag-data` (tabla de jobs) para sobrevivir reinicios.
Integración opcional con `ag-realtime` para fan-out de eventos. Cada job
emite métricas hacia `ag-observe`: `ag_mail_sent_total`,
`ag_mail_failed_total`, `ag_mail_retry_total`, histograma de latencia.

La **integración con `ag-auth`** es estrictamente unidireccional: `ag-auth`
consume `ag-mail` invocando un trait pequeño que `ag-auth` define. `ag-mail`
NO conoce a `ag-auth`. La sexta regla de la sección 5.3 documenta esta
direccionalidad.

Bloque `mail` del DSL v0.7 (ejemplo):

```ag
mail WelcomeEmail {
    from "hello@plenty.market"      # debe referenciar un bloque domain
    subject "Welcome to Plenty"
    template "emails/welcome.html"  # debe existir
    vars {
        name String
        activation_url String        # debe usarse en el HTML
    }
}
```

### 8.9 `ag-domains` — Gestión de dominios y TLS (opcional infra)

Introducido por `ADR-0007` en la Fase 4.5. `ag-domains` es un módulo
**opcional de infraestructura**: no todo backend administra DNS (muchos
despliegan tras un proxy o PaaS que ya lo resuelve), pero cuando un proyecto
quiere que `ag deploy` entregue una URL `https://miapi.example.com` con
certificado válido en un comando, `ag-domains` es el módulo responsable.

El módulo **NO es un registrador de dominios**: el dominio se compra
externamente (Namecheap, Cloudflare Registrar, etc.) y se delega vía
nameservers al proveedor configurado. `ag-domains` tampoco reemplaza
Terraform ni Pulumi: para infraestructura compleja multi-cloud o gestión
centralizada de zonas DNS arbitrarias, el proyecto debe usar las
herramientas dominantes. La frontera está fijada en el ADR.

El núcleo del módulo es el trait `DnsProvider`:

```rust
#[async_trait::async_trait]
pub trait DnsProvider: Send + Sync {
    async fn list_records(&self, zone: &str) -> Result<Vec<DnsRecord>, AgDomainsError>;
    async fn upsert_record(&self, zone: &str, record: &DnsRecord) -> Result<(), AgDomainsError>;
    async fn delete_record(&self, zone: &str, id: &str) -> Result<(), AgDomainsError>;
    fn provider_name(&self) -> &'static str;
}
```

Pequeño, versionado, con **tests de contrato** que todo adapter debe pasar.
El adapter inicial es Cloudflare (autenticación por API token). El trait
está diseñado para añadir Route53, Namecheap, DigitalOcean, etc. en
iteraciones posteriores sin tocar la superficie pública.

El **cliente ACME** (`instant-acme`) emite y renueva certificados de
Let's Encrypt. Soporta el challenge DNS-01 (preferido, usa el propio
`DnsProvider` para crear el TXT requerido) y HTTP-01 (alternativo). La
renovación corre como tarea Tokio en segundo plano, vigilando expiración
y renovando antes del umbral configurado. El almacenamiento de certificados
es filesystem por defecto, o `ag-storage` opcional.

La cooperación con `ag-mail` se materializa en `generate_mail_records`:
`ag-mail` declara sus requisitos vía `MailSender::dns_requirements` y
`ag-domains` los materializa como registros (SPF, DKIM, DMARC). Esta es una
relación de **cooperación**, no de control: `ag-mail` no depende de
`ag-domains`; un proyecto puede usar `ag-mail` con un adapter gestionado
(via SMTP) sin que `ag-domains` participe.

La **verificación de propagación** usa `hickory-resolver` para consultar
múltiples resolvers públicos y confirmar que los registros propagaron antes
de marcar una operación como exitosa. Esto bloquea `ag deploy` hasta que el
dominio responde, evitando entregar URLs que el operador prometió pero que
no resuelven todavía.

Bloque `domain` del DSL v0.7 (ejemplo):

```ag
domain plenty.market {
    provider "cloudflare"
    tls { mode auto  acme true }
    dns {
        CNAME "api"     -> "ag-cloud-target"
        TXT   "_dmarc"  -> "v=DMARC1; p=quarantine"
    }
    mail { spf auto  dkim auto  dmarc quarantine }
}
```


---

## 9. Sistema de plugins WASI (`ag-wasm-host`)

La extensibilidad de Anti-Gravital se construye sobre WebAssembly System Interface (WASI), no sobre el ecosistema de crates Rust nativos. Esta decisión tiene tres razones que la hacen no negociable.

La primera razón es seguridad. Los plugins son código de terceros que el operador del servidor ejecuta. Si fueran código nativo, un plugin malicioso o defectuoso podría corromper memoria del proceso, escapar a syscalls arbitrarias, o filtrar secretos. Los módulos WASI ejecutan en una sandbox con permisos explícitos declarados en el manifest del plugin; el plugin no puede acceder al filesystem, a la red, ni a syscalls que no estén declarados.

La segunda razón es multilenguaje. Un plugin WASI puede escribirse en Rust, Go (TinyGo), C, C++, AssemblyScript, Zig o cualquier lenguaje que compile a WebAssembly. Esto democratiza el ecosistema: un experto en seguridad que escribe en Go puede contribuir un exportador para Datadog sin tener que aprender Rust.

La tercera razón es estabilidad de ABI. La interfaz entre el host y el plugin se define con `wit-bindgen` y el Component Model, lo que permite que un plugin compilado para una versión de Anti-Gravital siga funcionando con versiones futuras sin recompilación, siempre que la ABI no cambie.

### 9.1 Runtime de plugins

El runtime es `wasmtime`, embebido como crate Rust. Cada plugin se carga en un store aislado con límites de memoria (256 MB por defecto, configurable), límites de fuel (consumo de instrucciones), y timeout de ejecución.

### 9.2 Ciclo de vida de un plugin

El ciclo de vida de un plugin tiene cinco estados. El primero es **descubierto**: el archivo `.wasm` está en el directorio de plugins del proyecto y aparece en el manifest. El segundo es **validado**: el host inspecciona el binario, verifica que el component model sea compatible, lee el manifest y confirma que los permisos solicitados están autorizados. El tercero es **cargado**: el módulo se compila ahead-of-time con Cranelift y se almacena en memoria. El cuarto es **activo**: el plugin recibe eventos y responde a invocaciones. El quinto es **descargado**: el plugin se libera, sea por shutdown del servidor o por reload dinámico.

### 9.3 Manifest del plugin

Cada plugin trae un archivo `plugin.toml` con sus metadatos y permisos solicitados:

```toml
[plugin]
name = "datadog-exporter"
version = "1.2.0"
author = "Gravital Labs"
license = "Apache-2.0"
description = "Exports metrics and traces to Datadog"

[abi]
anti_gravital_version = ">= 1.0.0, < 2.0.0"
component_model = "0.5"

[permissions]
network = ["api.datadoghq.com:443", "api.datadoghq.eu:443"]
env = ["DD_API_KEY", "DD_SITE"]
filesystem = []
clock = "read"

[capabilities]
exports = ["metrics_exporter", "trace_exporter"]
imports = ["host_logger", "host_clock"]

[limits]
max_memory = "64MB"
max_execution_time = "5s"
fuel = 100_000_000
```

### 9.4 API de host expuesta a plugins

El host expone un conjunto reducido de capacidades a los plugins, definidas en interfaces WIT (WebAssembly Interface Types). Las principales son: logger (escribir mensajes al sistema de tracing del host), clock (obtener tiempo actual y medir intervalos), metrics (registrar métricas adicionales), KV (almacenamiento clave-valor persistente por plugin), HTTP client (con allowlist de hosts del manifest), y events (subscripción al bus interno).

### 9.5 Puntos de extensión del framework

Los plugins pueden extender Anti-Gravital en cinco puntos: middleware adicional en la Shield (request hooks), handlers personalizados registrados en el router, exporters de observabilidad (métricas, traces, logs), processors de eventos (subscriptores al bus interno), y comandos personalizados de la CLI (`ag <plugin-cmd>`).

### 9.6 Plugins oficiales

El repositorio mantiene un conjunto de plugins oficiales bajo `plugins/`, cada uno con su propio crate y release cycle: `prometheus-exporter`, `datadog-exporter`, `sentry`, `honeycomb-exporter`, `slack-notifier`, `discord-webhook`. La existencia de plugins oficiales sirve como referencia técnica y como ejemplo de implementación para terceros.

### 9.7 Registro de plugins

A partir de la versión 1.0 del framework, se publica un registro oficial en `plugins.antigravital.dev`. El registro indexa plugins con metadatos verificados, escaneo de seguridad básico, y reviews de la comunidad. La instalación se hace con `ag plugin add <nombre>`. Los plugins se descargan, validan, y registran en el manifest del proyecto.

---

## 10. Subsistema de despliegue (`ag-cloud` + `ag-domains`)

Una de las correcciones estructurales más importantes derivadas del análisis crítico es que `ag-cloud` no es un competidor de Terraform ni de Kubernetes. Su rango objetivo es el mismo que cubren Railway, Fly.io, Render y Coolify: simplificar el despliegue de aplicaciones backend a entornos típicos sin obligar al equipo a operar infraestructura completa. Desde la Fase 4.5 (`ADR-0007`), `ag-cloud` coopera con `ag-domains` para resolver dominio, TLS y registros de correo dentro del propio flujo de `ag deploy`, sin reemplazar a los proveedores dominantes (Let's Encrypt, Cloudflare) y sin convertirse en un panel de hosting.

### 10.1 Filosofía de `ag-cloud`

El operador típico de un proyecto Anti-Gravital, especialmente en sus primeros años de vida, no necesita ni quiere operar un clúster Kubernetes. Necesita levantar su API en un VPS, conectarla a una base de datos, ponerla detrás de TLS, y olvidarse. `ag-cloud` resuelve este caso.

Para casos más complejos (despliegues multi-región, alta disponibilidad, gestión de secrets centralizada, políticas IAM, infraestructura compartida entre múltiples aplicaciones), `ag-cloud` no es la herramienta correcta y el proyecto debe declararlo abiertamente: usa Terraform, Pulumi o Helm.

### 10.2 El archivo `deploy.ag`

El subsistema de despliegue se controla con un archivo declarativo `deploy.ag` separado del schema del proyecto:

```yaml
app:
  name: payments-api
  domain: api.example.com

runtime:
  replicas: 3
  port: 8080
  health_check: /health
  resources:
    cpu: 1
    memory: 512MB

database:
  type: postgres
  version: "16"
  size: 20GB
  backup_schedule: "daily"

cache:
  type: redis
  version: "7"
  size: 1GB

storage:
  type: s3
  bucket: payments-api-uploads

secrets:
  source: vault
  path: secret/payments-api

observability:
  metrics: prometheus
  traces: tempo
  logs: loki

deployment:
  target: docker-compose      # opciones: docker-compose, fly, railway, k8s
  strategy: rolling
  max_surge: 1
  max_unavailable: 0
```

### 10.3 Targets de despliegue soportados

`ag-cloud` soporta cuatro targets de despliegue, cada uno con un nivel de abstracción distinto.

El target **docker-compose** genera un `docker-compose.yml` completo con servicios, redes, volúmenes, healthchecks, secrets cargados de archivos `.env` o de un secret manager, reverse proxy (Caddy por defecto) con TLS automático vía Let's Encrypt, y backup scripts para la base de datos. Es el target recomendado para self-hosting en un VPS único.

El target **fly** genera un `fly.toml` y ejecuta los comandos `flyctl` necesarios para desplegar a Fly.io. Es el target recomendado para edge computing global con bajo overhead operacional.

El target **railway** genera la configuración para Railway y triggerea el despliegue vía su API. Es el target recomendado para equipos que prefieren PaaS sin operación.

El target **k8s** genera manifests Kubernetes estándar (Deployment, Service, Ingress, ConfigMap, Secret, HorizontalPodAutoscaler) con valores razonables. Para configuraciones avanzadas, este target es un punto de partida que el equipo customiza, no una solución completa.

### 10.4 Pipeline de despliegue

El comando `ag deploy` ejecuta un pipeline estandarizado: validación del schema, compilación con `cargo build --release --target <target>`, construcción de la imagen Docker desde una base `scratch` o `distroless`, ejecución de tests de smoke, push de la imagen a un registro, aplicación de migraciones de base de datos en orden, despliegue rolling con healthchecks, y verificación post-despliegue.

### 10.5 Reverse proxy y TLS

Para despliegues docker-compose, `ag-cloud` configura Caddy como reverse proxy con TLS automático. Caddy obtiene y renueva certificados Let's Encrypt sin configuración explícita. Para entornos donde TLS lo gestiona un balanceador externo (Cloudflare, AWS ALB), Caddy se desactiva.

### 10.6 Integración con `ag-domains`

Introducida por `ADR-0007`. Cuando un proyecto declara dominios en su contrato
`.ag` (bloque `domain` del DSL v0.7), `ag deploy` resuelve un flujo de seis
pasos coordinado con `ag-domains`:

1. **Validar control del dominio.** Inserción de un registro TXT de verificación
   vía el `DnsProvider` configurado y confirmación de su presencia.
2. **Configurar DNS de aplicación.** `upsert_record` para apuntar el dominio al
   target del despliegue (CNAME al host de Fly/Railway, o registros A/AAAA en
   docker-compose).
3. **Emitir o renovar TLS.** Cliente ACME contra Let's Encrypt (DNS-01
   preferido). El certificado se almacena en filesystem o `ag-storage`.
4. **Asociar el dominio al target.** Configurar el reverse proxy (Caddy en
   docker-compose, fly cert en Fly, etc.) para servir el dominio con el
   certificado emitido.
5. **Materializar SPF/DKIM/DMARC** que `ag-mail` haya declarado en sus
   `MailSender::dns_requirements`.
6. **Verificar propagación** contra múltiples resolvers públicos antes de
   marcar el despliegue como exitoso.

`ag-cloud` **NO depende rígidamente** de `ag-domains` en todos los targets:
si el proyecto no declara dominios, el flujo se omite. Si el target es uno
donde el TLS lo gestiona un balanceador externo (Cloudflare en frente,
AWS ALB), `ag-cloud` puede saltarse el paso 3 sin afectar el resto del
pipeline. Esta flexibilidad es lo que mantiene a `ag-domains` como módulo
opcional, no como pieza obligatoria del runtime.

---

## 11. Integración con Inteligencia Artificial (`ag-ai`) y el Knowledge Graph

El módulo `ag-ai` es probablemente el diferenciador más significativo del proyecto en el contexto 2026, donde los agentes de IA son colaboradores cotidianos en el desarrollo de software. El módulo tiene dos componentes complementarios.

### 11.1 El Anti-DSL como contrato para agentes

El primer componente no es código: es la decisión arquitectónica de que el `schema.ag` sirva como contrato perfecto para agentes de IA. Un agente que recibe un endpoint declarado en `.ag` tiene exactamente lo que necesita para generar un handler correcto: tipos precisos, errores definidos, políticas de acceso, validaciones, eventos a emitir. Y, crítica diferencia con cualquier otro framework, el compilador Rust verifica después que el código generado por el agente sea type-safe antes de que llegue a producción.

Esto convierte el flujo de desarrollo en una colaboración estructurada. El ingeniero diseña el schema. El agente implementa los handlers. El compilador actúa como segundo revisor automático que rechaza cualquier desincronización. El operador supervisa y aprueba.

### 11.2 El Knowledge Graph

El segundo componente es el grafo de conocimiento del proyecto. `ag-ai` mantiene un grafo dirigido que indexa todas las entidades del proyecto y sus relaciones: modelos, endpoints, eventos, políticas, dependencias entre handlers, llamadas a bases de datos, llamadas a servicios externos, configuraciones, plugins instalados.

El grafo se reconstruye automáticamente en cada `ag generate` y se serializa a `.ag/knowledge-graph.json`. Desde este insumo se producen automáticamente: documentación arquitectónica en Markdown, diagramas C4 (Context, Container, Component) en formato Mermaid, registros de decisión arquitectónica (ADRs) sugeridos, listas de dependencias críticas, mapas de impacto para cambios propuestos, y un dashboard interactivo en el dev server.

### 11.3 Capacidades AI asistidas

El módulo expone tres capacidades adicionales accesibles desde la CLI:

`ag ai suggest-schema` analiza un dominio descrito en lenguaje natural y propone un primer borrador de `schema.ag`. El ingeniero refina desde allí.

`ag ai review-migration` analiza una migración SQL propuesta y reporta riesgos: locks, downtime, pérdida de datos, queries lentas durante la transición. Sugiere alternativas con migración en dos pasos cuando es necesario.

`ag ai analyze-architecture` produce un reporte sobre el grafo de conocimiento del proyecto: identificación de hotspots (modelos con demasiadas dependencias), endpoints sin tests, eventos emitidos pero sin consumidores, dead code, antipatrones.

### 11.4 Conexión con proveedores de modelos

El módulo no embebe ningún modelo de lenguaje. Se conecta a proveedores externos vía API: Anthropic Claude, OpenAI, modelos locales servidos por Ollama o vLLM. La conexión es configurable y el operador puede elegir o auto-hospedar. Para entornos sensibles, se soporta el modo offline donde las funciones AI están desactivadas pero el resto del framework funciona normalmente.

---

## 12. Framework de migración (`ag-migrate`): importadores

La adopción real de cualquier framework backend exitoso ha pasado siempre por la posibilidad de migrar desde el incumbente. La industria odia las reescrituras. El módulo `ag-migrate` no es un afterthought; es un ciudadano de primera clase del proyecto.

### 12.1 Importadores soportados

`ag-migrate` ofrece importadores oficiales para los frameworks más adoptados del mercado.

El importador **OpenAPI** consume cualquier spec OpenAPI 3.0 o 3.1 y produce un `schema.ag` con modelos, endpoints, errores y validaciones. Es el importador más genérico y sirve para migrar desde cualquier servicio que documente una OpenAPI, independiente del lenguaje en que esté escrito.

El importador **Prisma** consume un archivo `schema.prisma` y traduce modelos, relaciones y migraciones a Anti-Gravital. Cubre la migración desde aplicaciones TypeScript que usan Prisma como ORM.

El importador **Django** lee modelos Django (definidos como clases Python) y produce los modelos Anti-Gravital equivalentes. Incluye traducción de relaciones, managers, signals y migraciones.

El importador **FastAPI** consume aplicaciones FastAPI examinando los routers y los modelos Pydantic. Produce endpoints y modelos Anti-Gravital. Es probablemente el caso de migración más natural por la similitud filosófica entre FastAPI y Anti-Gravital.

El importador **Sequelize** lee modelos de aplicaciones Node.js que usan Sequelize ORM. Cubre el caso Express + Sequelize, muy común en el mercado.

El importador **GraphQL** consume un schema GraphQL SDL y produce su equivalente en Anti-Gravital.

### 12.2 Limitaciones honestas

Los importadores cubren la traducción del contrato (modelos, endpoints, validaciones), no la lógica de negocio. La lógica de los handlers debe escribirse manualmente o con asistencia de un agente de IA. Esto se documenta claramente para evitar expectativas erróneas.

### 12.3 Guías oficiales de migración

Para cada framework soportado se publica una guía oficial en la documentación: estrategia recomendada (big bang vs strangler fig), patrones para coexistencia durante la transición (proxy reverso que reparte tráfico entre el sistema legacy y el nuevo), testing comparativo, y casos de estudio reales cuando estén disponibles.

---

## 13. Puente de aplicaciones nativas (`ag-mobile`): Flutter y clientes generados

El reposicionamiento más importante derivado del análisis crítico es que Anti-Gravital no compite con Flutter. Se posiciona como **el backend nativo ideal para aplicaciones Flutter**. Esto multiplica el valor estratégico del proyecto: en lugar de competir con un framework de UI multiplataforma maduro y muy bien diseñado, Anti-Gravital se convierte en su compañero natural.

### 13.1 Generación de SDK Dart

`ag-mobile` genera un paquete Dart completo a partir del `schema.ag`. El paquete incluye tipos generados con freezed para inmutabilidad, cliente HTTP basado en dio con interceptores para autenticación, cliente WebSocket para realtime, soporte de Server-Sent Events, y mocks para tests.

### 13.2 Autenticación nativa Flutter

El módulo incluye widgets y servicios listos para los flujos de autenticación. La integración con WebAuthn aprovecha las plataformas nativas (Android Credential Manager API, iOS Passkeys vía AuthenticationServices). El flujo OAuth2 usa `flutter_appauth` con configuración auto-generada.

### 13.3 Offline-first y sincronización

`ag-mobile` ofrece un layer de sincronización offline opcional. Las operaciones se encolan localmente en una base SQLite, se replican al servidor cuando hay conectividad, y los conflictos se resuelven con políticas declarativas en el schema (last-write-wins, custom merge, server-wins). Es una funcionalidad ambiciosa que se implementa en una fase tardía del roadmap.

### 13.4 Otros clientes generados

Aunque Flutter es el target prioritario para móvil, el sistema de codegen es extensible. La versión 1.0 incluye generadores para Dart (Flutter), TypeScript (React, Vue, Svelte, Next.js), y Kotlin (Android nativo, opcionalmente Kotlin Multiplatform). Una versión posterior puede incluir Swift y Python.

---

## 14. Observabilidad (`ag-observe`)

Aunque ya cubierta brevemente como módulo estándar, la observabilidad merece una sección propia porque es probablemente la diferencia más visible entre un framework juguete y un framework de producción.

### 14.1 Tres pilares

`ag-observe` cubre los tres pilares clásicos: métricas, traces y logs. El stack es OpenTelemetry como capa de abstracción, con exporters configurables.

Las métricas se exponen en `/metrics` en formato Prometheus por defecto. Incluyen latencia por endpoint (p50, p95, p99, p999), throughput, tasa de errores por código HTTP, uso de pool de base de datos, uso de pool de Redis, conexiones WebSocket activas, y métricas custom registradas por la aplicación.

Los traces se exportan vía OTLP a cualquier backend compatible (Tempo, Jaeger, Datadog, Honeycomb, Lightstep). Cada request genera un trace con spans para la Shield, los handlers, las queries SQL, las llamadas externas, y la emisión de eventos.

Los logs son estructurados (JSON por defecto) e incluyen siempre el correlation ID. Se exportan a stdout (estándar para entornos cloud-native) y opcionalmente a backends como Loki o Datadog.

### 14.2 Dashboards Grafana incluidos

El repositorio incluye dashboards Grafana pre-configurados en JSON que el operador importa directamente. Cubren: overview del servicio, latencia y throughput por endpoint, errores y excepciones, salud de la base de datos, salud del caché, y métricas de runtime Rust (uso de memoria, número de tasks Tokio, GC pauses — que siempre serán cero, pero el dashboard lo confirma).

### 14.3 Inspección en vivo con tokio-console

En modo desarrollo, `tokio-console` se habilita automáticamente. Permite al desarrollador conectarse al proceso y ver en tiempo real qué tasks están ejecutándose, cuáles están bloqueadas, dónde se están consumiendo los recursos. Es una herramienta de debugging tremendamente útil que existe solo en Rust con Tokio.

---

## 15. Modelo de seguridad

La seguridad es una preocupación transversal, no un módulo. Esta sección documenta las garantías y las prácticas del proyecto.

### 15.1 Garantías por construcción

Rust elimina por construcción cuatro categorías de bugs que históricamente representan más del 70% de las vulnerabilidades críticas en software de sistemas: use-after-free, buffer overflows, data races, y null pointer dereferences. Estas garantías son a nivel de compilador, no de runtime; no requieren GC ni runtime checks.

Anti-Gravital prohíbe el uso de `unsafe` en todo el código del framework salvo en bloques explícitamente justificados, documentados, y revisados por al menos dos mantenedores. Cada bloque `unsafe` viene acompañado de un comentario que explica por qué es necesario y qué invariantes preserva.

### 15.2 Prácticas de criptografía

Las primitivas criptográficas se importan del crate `ring`, mantenido por miembros del equipo BoringSSL de Google. No se rueda criptografía propia. Los algoritmos por defecto son Ed25519 para firmas, ChaCha20-Poly1305 para AEAD, Argon2id para hashing de passwords, y TLS 1.3 para transporte. Algoritmos heredados (RSA, AES-CBC, SHA-1) están disponibles solo para interoperabilidad explícita.

### 15.3 Política de divulgación responsable

El repositorio mantiene un archivo `SECURITY.md` con direcciones de contacto (primario `anti@gravitalcloud.com`, respaldo `angelnereira@gravitalcloud.com`) y una política clara: las vulnerabilidades se reportan privadamente, el equipo confirma recepción en 48 horas, publica un parche en 30 días para vulnerabilidades críticas, y un CVE con crédito al reportero.

### 15.4 Auditorías

Antes de la versión 1.0 estable, el componente Shield del framework se somete a una auditoría externa por una empresa especializada en seguridad de sistemas Rust (Trail of Bits, NCC Group o equivalente). El reporte de auditoría se publica con el lanzamiento.

### 15.5 Fuzzing continuo

El parser del DSL y el parser HTTP se someten a fuzzing continuo con `cargo-fuzz`. La CI ejecuta corpus de fuzzing en cada PR; antes del 1.0, se completan al menos 72 horas de fuzzing sin crashes en cada parser.

---

## 16. Objetivos de rendimiento y metodología de validación

Esta sección sustituye los benchmarks absolutos del v3.0. Las cifras anteriores se presentaban como hechos cuando en realidad son extrapolaciones de componentes individuales. Esta versión las refrasea honestamente como **objetivos de diseño**, contra los cuales el proyecto se medirá públicamente.

### 16.1 Objetivos de diseño

| Métrica                                                    | Objetivo                  | Base de extrapolación                        |
|------------------------------------------------------------|---------------------------|----------------------------------------------|
| Throughput Hello World (plaintext)                         | ≥ 300 K req/s             | Axum + Tokio en TechEmpower                  |
| Throughput JSON simple                                     | ≥ 150 K req/s             | Axum + serde_json en benchmarks públicos     |
| Throughput CRUD con PostgreSQL                             | ≥ 40 K req/s              | sqlx + connection pool                       |
| Latencia p99 con DB query                                  | ≤ 5 ms                    | Mediciones de servicios Tokio en producción  |
| Memoria base (proceso idle, sin tráfico)                   | ≤ 15 MB                   | Tamaño de binarios Rust + Tokio              |
| Tiempo de arranque en frío                                 | ≤ 100 ms                  | Binarios Rust estáticos en Linux             |
| Tamaño del binario release con todos los módulos estándar  | ≤ 20 MB                   | Compilaciones de proyectos similares         |
| Conexiones WebSocket concurrentes en una instancia 2 vCPU  | ≥ 50 000                  | Tokio tasks stackless                        |

Estas cifras son objetivos técnicos. La especificación del proyecto exige que sean medidas con la suite `ag bench` en el repositorio, y que cada release publique los resultados reproducibles. Si una métrica no se alcanza, se publica como tal y se documenta el déficit. La credibilidad técnica del proyecto depende de no exagerar.

### 16.2 Metodología de medición

Toda comparación con frameworks competidores se hace bajo TechEmpower Framework Benchmarks, ejecutado por el equipo o por terceros independientes. Las comparaciones publicadas en la documentación incluyen: versión exacta del framework comparado, configuración usada, hardware del benchmark, número de runs y desviación estándar. Comparaciones que no cumplan estas reglas no se publican.

### 16.3 Hitos de validación para v1.0

La versión 1.0 estable se libera solo cuando se cumplen los siguientes hitos:

- Posición top-10 en TechEmpower Round (categorías Plaintext y JSON Serialization)
- Auditoría externa de seguridad sin findings críticos sin resolver
- 72 horas de fuzzing del parser DSL y el parser HTTP sin crashes
- Load test de 500 K req/s sostenidos por 30 minutos sin degradación >5%
- 24 horas de carga continua sin crecimiento de memoria detectable
- Binarios verificados en Linux x86-64, Linux ARM64, macOS ARM64, Windows x64
- Al menos un servicio en producción en Gravital Cloud por 30 días sin incidentes
- Al menos tres proyectos externos usando Anti-Gravital en producción

---

## 17. Modelo de gobernanza Open Source

### 17.1 Licencia y promesa

La licencia es Apache 2.0 para todo el ecosistema. No existe ni existirá una versión Enterprise cerrada con features reservadas para clientes pagos. El compromiso es explícito y se documenta en el README. Cualquier cambio de licencia futuro requeriría la aprobación de toda la comunidad de mantenedores, y el ecosistema sigue siendo forkable.

### 17.2 Modelo de mantenimiento

El proyecto adopta un modelo BDFL inicial con plan de transición a meritocracia explícita. En la fase inicial (versiones 0.x), Ángel Nereira es el mantenedor principal. A partir de la versión 1.0, se establece un comité técnico de cinco personas elegidas entre los contribuidores con mayor historial. El comité aprueba RFCs (Request For Comments) para cambios mayores.

### 17.3 RFCs

Cualquier cambio que afecte la API pública, el DSL, la arquitectura de plugins o el modelo de seguridad requiere un RFC. El proceso es: el proponente abre un issue en `anti-gravital-rfcs/`, la comunidad debate por al menos dos semanas, el comité técnico vota. Una vez aprobado, el RFC se mueve a estado "Accepted" y se implementa en una versión específica.

### 17.4 Compatibilidad

Después de la versión 1.0, el proyecto se compromete a semver estricto en la API pública. Breaking changes solo en mayores. Las versiones LTS se anuncian con un calendario público, con al menos 18 meses de soporte de seguridad.

### 17.5 Sostenibilidad económica

El proyecto se sostiene en tres patas. La primera es Gravital Labs (Nereira Technology and Business Solutions), que financia el desarrollo inicial como inversión estratégica. La segunda es servicios profesionales: consultoría de adopción, training y soporte premium para empresas que quieran SLA, sin que esto cierre features del producto. La tercera, a futuro, son sponsors corporativos (GitHub Sponsors, Open Collective) de empresas que dependen del proyecto.

---

## 18. Análisis de riesgos y mitigaciones

Esta sección documenta los riesgos reales del proyecto y las mitigaciones planeadas. Es deliberadamente honesta; un proyecto que no enumera sus riesgos no merece confianza.

### 18.1 Riesgo: complejidad del compilador del DSL

El compilador del DSL es un proyecto de varios años por sí solo. La mitigación es la implementación incremental por versiones del DSL descrita en la sección 7. La versión 0.1 cubre solo modelos básicos y es entregable en dos meses. Cada versión añade un subconjunto bien definido. La versión 1.0 estable del DSL es el hito de mayor riesgo del proyecto y se planifica para el final del cronograma.

### 18.2 Riesgo: curva de aprendizaje de Rust

Rust tiene una curva de aprendizaje real. La mitigación es triple. Primero, el DSL genera el 80% del scaffolding, de modo que los handlers que el desarrollador escribe son Rust simple: unos pocos `await`, acceso a estado compartido, retornar un `Result`. Segundo, la documentación incluye una guía "Rust para desarrolladores de Python/Node.js" con los conceptos mínimos necesarios. Tercero, el asistente AI integrado puede generar handlers que el desarrollador supervisa.

### 18.3 Riesgo: competencia con grandes players

Spring, .NET, Express y FastAPI tienen ecosistemas de décadas. Anti-Gravital no puede competir frontalmente con ellos en breadth. La mitigación es enfocarse en nichos donde los incumbentes tienen debilidades estructurales: aplicaciones de alta carga, servicios edge, backends para Flutter, backends para aplicaciones IA con streaming.

### 18.4 Riesgo: bus factor

El proyecto inicial tiene un bus factor preocupantemente bajo (un mantenedor). La mitigación es activa: documentación interna completa desde el día uno, incorporación de contribuidores externos desde la fase 1, y transición a comité técnico antes del 1.0.

### 18.5 Riesgo: cambios en el ecosistema Rust

El ecosistema Rust sigue evolucionando rápidamente. Axum, Tokio y sqlx pueden hacer cambios breaking en versiones futuras. La mitigación es pinneo conservador de versiones, tests de integración exhaustivos contra cada nueva versión de las dependencias core, y participación activa en sus comunidades para anticipar cambios.

### 18.6 Riesgo: fragmentación de la comunidad

Si la comunidad de Anti-Gravital fragmenta (por ejemplo, surgen forks competidores con features divergentes), el ecosistema se debilita. La mitigación es un proceso RFC abierto que da voz real a la comunidad, releases predecibles, y una hoja de ruta pública.

### 18.7 Riesgo: vulnerabilidades de seguridad post-lanzamiento

Aunque Rust elimina muchas categorías de vulnerabilidades, no elimina las lógicas (autorización rota, leaks de información, races a nivel de aplicación). La mitigación es la auditoría externa antes del 1.0, el programa de divulgación responsable, fuzzing continuo, y CI con análisis estático (clippy, cargo-audit, cargo-deny).

---

## 19. Glosario técnico

| Término                       | Definición                                                                                                                |
|-------------------------------|--------------------------------------------------------------------------------------------------------------------------|
| Anti-DSL (.ag)                | Lenguaje de definición de dominio del framework. Schema-first.                                                            |
| Axum                          | Framework HTTP de Rust construido sobre Tokio y Tower. Base del Core.                                                     |
| Backpressure                  | Mecanismo por el cual el sistema rechaza trabajo nuevo cuando está saturado. Implementado nativamente en Tower.            |
| Cargo                         | Sistema de build y gestor de paquetes de Rust.                                                                            |
| Cargo-fuzz                    | Herramienta de fuzzing integrada con Cargo.                                                                                |
| Core (capa B)                 | Capa de lógica de negocio del núcleo. Axum router, handlers, estado compartido.                                          |
| Correlation ID                | Identificador único por request que atraviesa todos los logs, traces y errores.                                          |
| Ed25519                       | Algoritmo de firma digital basado en la curva Edwards25519. Default para JWT en Anti-Gravital.                            |
| Flamegraph                    | Visualización de profiling de CPU. Con Rust puro cubre toda la aplicación sin gaps.                                       |
| Fuel (wasmtime)               | Cuota de instrucciones que un plugin WASM puede ejecutar antes de ser interrumpido.                                       |
| GIL                           | Global Interpreter Lock. Mecanismo de CPython que impide ejecución paralela real.                                         |
| Governor                      | Crate Rust para rate limiting basado en token bucket. Thread-safe sin locks contenciosos.                                 |
| HTMX                          | Librería JavaScript pequeña que permite interactividad sin frameworks SPA.                                                |
| JetStream                     | Sistema de persistencia de mensajes de NATS. Permite replay y durabilidad.                                                |
| Knowledge Graph               | Grafo dirigido del proyecto Anti-Gravital. Indexa modelos, endpoints, eventos, dependencias.                              |
| LSP                           | Language Server Protocol. El DSL `.ag` ofrece LSP para autocompletado en editores.                                        |
| Moka                          | Caché concurrente Rust con TinyLFU. Thread-safe sin locks contenciosos.                                                   |
| NATS                          | Sistema de mensajería pub/sub usado por `ag-realtime`.                                                                    |
| OpenAPI                       | Especificación estándar para describir APIs HTTP. Anti-Gravital la genera automáticamente.                                |
| Passkeys                      | Estándar FIDO2/WebAuthn para autenticación sin password.                                                                   |
| Ring                          | Crate Rust de criptografía de bajo nivel. Mantenido por miembros del equipo BoringSSL.                                    |
| Rustls                        | Implementación de TLS 1.3 en Rust puro, sin OpenSSL.                                                                       |
| Schema drift                  | Condición donde la definición de un schema queda desincronizada entre capas. Anti-Gravital la elimina por diseño.         |
| Schema-per-tenant             | Arquitectura multi-tenant donde cada cliente tiene su propio schema en PostgreSQL.                                       |
| Shield (capa A)               | Capa de confianza del núcleo. Pipeline de middleware Tower: TLS, auth, validation, rate limit, RBAC, CORS.                |
| sqlx                          | Crate Rust de acceso a bases de datos con verificación de queries en compile time.                                        |
| TechEmpower                   | Suite de benchmarks estándar de la industria para comparar frameworks web.                                                |
| Tokio                         | Runtime async de Rust. Provee concurrencia M:N mediante tasks livianas sin GC.                                            |
| tokio-console                 | Herramienta de diagnóstico en vivo para aplicaciones Tokio.                                                                |
| Tower                         | Crate Rust para servicios y middleware composables. Base arquitectónica del Shield.                                       |
| WASI                          | WebAssembly System Interface. Estándar para módulos WebAssembly con acceso controlado al sistema.                         |
| wasmtime                      | Runtime WebAssembly embebible en Rust. Host del sistema de plugins.                                                       |
| WebAuthn                      | Estándar W3C para autenticación con factores hardware (passkeys, security keys).                                          |
| Zero-copy                     | Transferencia de datos sin copiarlos en memoria. Reduce overhead de CPU.                                                  |
| Zero-overhead abstraction     | Principio de Rust: una abstracción no debe costar rendimiento frente al código manual equivalente.                       |
| ACME                          | Automatic Certificate Management Environment. Protocolo de emisión y renovación automática de certificados TLS (Let's Encrypt). Usado por `ag-domains` desde la Fase 4.5. |
| DKIM                          | DomainKeys Identified Mail. Mecanismo de autenticación de correo por firma criptográfica del dominio remitente. Generado por `ag-domains` para `ag-mail`. |
| SPF                           | Sender Policy Framework. Registro DNS que enumera servidores autorizados a enviar correo en nombre del dominio. Generado por `ag-domains` para `ag-mail`. |
| DMARC                         | Domain-based Message Authentication, Reporting and Conformance. Política que indica cómo tratar correo que falla SPF o DKIM. Generado por `ag-domains`. |
| MTA                           | Mail Transfer Agent. Servidor de correo completo (Postfix, Stalwart). `ag-mail` v1 **NO es un MTA**: solo envía outbound, no recibe inbound. |
| DnsProvider                   | Trait de `ag-domains` que abstrae proveedores DNS mediante adapters. Adapter inicial: Cloudflare. Diseñado para añadir Route53, Namecheap, etc. con tests de contrato. |
| Estándar diferido             | Clasificación de crate introducida por `ADR-0007`. Crate con madurez de estándar que NO se instala por defecto en templates oficiales. `ag-mail` es el primer caso. |

---

## 20. Apéndice: comparativa de mercado

Esta comparativa se ofrece como referencia técnica. Las cifras de los competidores se basan en benchmarks públicos verificables (TechEmpower, GitHub issues, documentación oficial). Las de Anti-Gravital son objetivos de diseño, no mediciones.

| Criterio                       | Spring Boot   | .NET Core     | FastAPI      | NestJS       | Anti-Gravital (objetivo)    |
|--------------------------------|---------------|---------------|--------------|--------------|-----------------------------|
| Runtime                        | JVM           | CLR           | CPython      | Node.js V8   | Ninguno (binario nativo)    |
| Memoria base                   | ~350 MB       | ~120 MB       | ~60 MB       | ~80 MB       | ≤ 15 MB                     |
| Tiempo de arranque             | ~6 s          | ~0.8 s        | ~0.8 s       | ~1.2 s       | ≤ 0.1 s                     |
| Throughput Hello World         | ~75 K req/s   | ~200 K req/s  | ~28 K req/s  | ~45 K req/s  | ≥ 300 K req/s               |
| Throughput CRUD + DB           | ~15 K req/s   | ~30 K req/s   | ~5 K req/s   | ~8 K req/s   | ≥ 40 K req/s                |
| Memory safety                  | Parcial       | Parcial       | Sí           | No           | Total (compilador Rust)     |
| Pausas de GC                   | Sí (JVM GC)   | Sí (CLR GC)   | No aplica    | Sí (V8 GC)   | No (sin GC)                 |
| Despliegue como binario único  | No            | Parcial       | No           | No           | Sí                          |
| Schema-first DX                | No            | No            | Parcial      | No           | Sí (Anti-DSL)               |
| Queries verificadas compile-time | No          | No            | No           | No           | Sí (sqlx)                   |
| DX nativa para agentes AI      | No            | No            | Parcial      | No           | Sí                          |
| Compilación cruzada nativa     | No            | No            | No           | No           | Sí                          |
| Licencia                       | Apache 2.0    | MIT           | MIT          | MIT          | Apache 2.0                  |

---

**Fin del documento de Arquitectura Técnica.**
Documento complementario: *Hoja de Ruta y Puertas de Verificación.*
Versión PDF unificada: *Anti-Gravital Blueprint v4.0 — Documento Maestro.*
