# Fase 5 - ag-cloud despliegue simplificado

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-04-8-fundaciones-del-proyecto.md](./fase-04-8-fundaciones-del-proyecto.md)
> Siguiente: [fase-05-5-funciones-serverless.md](./fase-05-5-funciones-serverless.md)

## Phase 5 — `ag-cloud` simplified deployment

**Objective.** Build the deployment subsystem in the style of Railway/Fly.io. Support for the four targets: docker-compose, fly, railway, k8s. This is the **public beta version (0.5)** milestone.

### 5.1 Entry criteria

- [ ] Phases 4, 4.5 and 4.6 implemented and the pre-Phase-5 release gate
  (`docs/audits/PRE_FASE5_RELEASE_GATE.md`) closed: every blocking row `pass`,
  including the 24-hour fuzz run (issue #156) and the final benchmark evidence
  (issue #157).
- [ ] RFC decision on the deployment targets supported in 1.0.
- [ ] Phase 4.8 stages A and B complete (project layout and declared
  environment), because deployment artifacts are generated from them.

RFC prerequisites (rule 28), each approved before its stage starts:

- [ ] RFC for the `deploy.ag` specification (new DSL surface).
- [ ] RFC for the `ag deploy` and `ag rollback` commands, including the
  secrets model and what is stored locally.

### 5.2 Deliverables

- [ ] `ag-cloud` crate with modules for each target.
- [ ] Specification of the `deploy.ag` file.
- [ ] Multi-stage Dockerfile generator optimized for minimal image.
- [ ] docker-compose target: complete stack generation with Caddy as reverse proxy and automatic TLS.
- [ ] fly target: integration with flyctl.
- [ ] railway target: integration with its API.
- [ ] k8s target: generation of standard manifests.
- [ ] Terraform integration: generated variable declarations and outputs for
  the resources a project needs; the user's Terraform owns the resources
  (`RFC-0019` section 4.6).
- [ ] `ag deploy` command.
- [ ] `ag rollback` command.
- [ ] Database migrations pipeline integrated into the deployment.
- [ ] Interactive deploy flow per `RFC-0018`: target selection, review screen
  listing every resource that will be created or changed, explicit
  confirmation; `ag rollback` shows what it will revert before acting. Flag
  parity is mandatory.
- [ ] Documentation: "From zero to production in 15 minutes" with each target.

Stages:

1. **5-A, specification.** `deploy.ag` grammar and validation; `ag-cloud`
   crate layout with one decoupled module per target behind its own Cargo
   feature; contract-test harness shared by all targets.
2. **5-B, image.** Multi-stage Dockerfile generator; reproducible image build;
   image size recorded.
3. **5-C, self-hosted target first.** docker-compose target with reverse proxy
   and automatic TLS; migrations pipeline; `ag deploy` and `ag rollback` for
   this target; optional `ag-domains` integration. This is the native,
   self-hostable path required by `ADR-0009` rule 5 and is delivered before
   any hosted target.
4. **5-D, k8s target.** Standard manifests generated from the same
   specification.
5. **5-E, hosted targets.** fly and railway, each as an adapter module.
6. **5-F, interactive flow and documentation.** Deploy wizard, per-target
   guides, release of 0.5.

### 5.3 Exit criteria (gate before Phase 6 and version 0.5)

- [ ] The `todo-api` example deploys successfully to Fly.io with `ag deploy`.
- [ ] The `ecommerce-api` example deploys successfully with docker-compose to a VPS and is accessed via domain with TLS.
- [ ] The `realtime-chat` example deploys successfully to Railway.
- [ ] Phase 4.7 stages A and B complete (interactive project creation and
  configuration).
- [ ] A failed deployment leaves the previous version serving, and
  `ag rollback` restores it; demonstrated on the docker-compose target.
- [ ] Version 0.5 (public beta) released on GitHub Releases.
- [ ] Public announcement on Hacker News, Reddit `/r/rust`, Twitter/X, Bluesky, LinkedIn.
- [ ] At least ten external projects report that they have deployed Anti-Gravital in production or staging.
- [ ] At least 1 500 stars on the repository.

### 5.4 Phase risks

The main risk is the dependency on external APIs (Fly, Railway) that can change. The mitigation is to structure each target as a decoupled module with contract tests.

The second risk is that deployment is the first place where the tool handles
secrets and performs irreversible actions on infrastructure the user pays for.
The mitigation is the order of the stages (self-hosted first), the mandatory
review screen before any change, and a secrets model fixed by RFC before code.

---

