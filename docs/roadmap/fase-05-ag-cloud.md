# Fase 5 - ag-cloud despliegue simplificado

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-04-5-ag-mail-y-ag-domains.md](./fase-04-5-ag-mail-y-ag-domains.md)
> Siguiente: [fase-06-ag-ai-knowledge-graph.md](./fase-06-ag-ai-knowledge-graph.md)

## Phase 5 — `ag-cloud` simplified deployment

**Objective.** Build the deployment subsystem in the style of Railway/Fly.io. Support for the four targets: docker-compose, fly, railway, k8s. This is the **public beta version (0.5)** milestone.

### 5.1 Entry criteria

- [ ] Phase 4 completed.
- [ ] RFC decision on the deployment targets supported in 1.0.

### 5.2 Deliverables

- [ ] `ag-cloud` crate with modules for each target.
- [ ] Specification of the `deploy.ag` file.
- [ ] Multi-stage Dockerfile generator optimized for minimal image.
- [ ] docker-compose target: complete stack generation with Caddy as reverse proxy and automatic TLS.
- [ ] fly target: integration with flyctl.
- [ ] railway target: integration with its API.
- [ ] k8s target: generation of standard manifests.
- [ ] `ag deploy` command.
- [ ] `ag rollback` command.
- [ ] Database migrations pipeline integrated into the deployment.
- [ ] Documentation: "From zero to production in 15 minutes" with each target.

### 5.3 Exit criteria (gate before Phase 6 and version 0.5)

- [ ] The `todo-api` example deploys successfully to Fly.io with `ag deploy`.
- [ ] The `ecommerce-api` example deploys successfully with docker-compose to a VPS and is accessed via domain with TLS.
- [ ] The `realtime-chat` example deploys successfully to Railway.
- [ ] Version 0.5 (public beta) released on GitHub Releases.
- [ ] Public announcement on Hacker News, Reddit `/r/rust`, Twitter/X, Bluesky, LinkedIn.
- [ ] At least ten external projects report that they have deployed Anti-Gravital in production or staging.
- [ ] At least 1 500 stars on the repository.

### 5.4 Phase risks

The main risk is the dependency on external APIs (Fly, Railway) that can change. The mitigation is to structure each target as a decoupled module with contract tests.

---

