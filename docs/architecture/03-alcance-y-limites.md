# Capitulo 3. Que es Anti-Gravital y que no es (alcance y limites)

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 3
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [02-manifiesto-y-posicionamiento.md](./02-manifiesto-y-posicionamiento.md)
> Siguiente: [04-estado-del-arte.md](./04-estado-del-arte.md)

## 3. What Anti-Gravital is and is not (scope and limits)

The clear definition of scope is probably the most important architectural decision of this project. A framework that tries to be everything ends up being nothing. This section establishes the explicit limits of the project.

### 3.1 What Anti-Gravital is

Anti-Gravital is:

- A high-performance **Rust backend runtime** for HTTP, WebSocket, and SSE services.
- A **domain definition language** (Anti-DSL, `.ag` files) and its compiler.
- A **unified CLI** (`ag`) for creation, generation, development, build, deployment, and administration.
- A **set of optional modules** published as independent Rust crates (auth, data, realtime, cache, storage, observe; mail and workers —deferred standard—).
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

