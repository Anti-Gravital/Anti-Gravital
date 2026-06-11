# Anti-Gravital

Rust-native, modular backend framework for building secure, high-performance
backend services with a schema-first workflow. The repository is a Cargo
workspace of `ag-*` crates plus the `ag` developer CLI.

[English](#english) | [Espanol](#espanol) — English is the canonical project
language. A concise Spanish version follows.

> Current status (verified 2026-06-10): capabilities through Phase 4.5 plus the additive Phase 4.6 work (native outbound MTA in `ag-mail`; the `ag-workers` background execution engine, stages S1-S5) are available, but the pre-Phase 5 release gate is still OPEN. Do not interpret implemented modules as a production-readiness certification.

Anti-Gravital gives Rust backend teams a coherent framework experience without
hiding the underlying Rust ecosystem. It combines:

- A secure HTTP runtime and Shield pipeline built on Axum, Tower, Tokio and rustls.
- A schema-first Anti-DSL (`.ag`) for models, endpoints, validation, auth,
  events, mail and domain declarations.
- Code generation for Rust, SQL migrations, TypeScript, OpenAPI and AsyncAPI.
- A unified CLI for scaffolding, development, builds, schema workflows, mail
  checks and domain operations.
- Modular crates for auth, cache, realtime, storage, observability, mail,
  domains, background jobs and edge routing. UI integration, AI, mobile,
  migration tooling, cloud deploy and WASI plugins are reserved placeholder
  crates for later phases.

It does not replace PostgreSQL, Redis, NATS, object storage, Docker, Kubernetes,
Terraform, Flutter or frontend frameworks. `ag-mail` handles transactional
email, not IMAP/POP mailboxes. `ag-domains` manages DNS/domain/TLS workflows;
it is not a registrar. Generated Rust handlers are application-owned stubs.

### Architecture map

Tiers follow the canonical crate classification in `CLAUDE.md` §14 and
chapter 5 of the architecture master. `*` marks a placeholder crate whose
implementation begins in a later phase.

| Tier | Crates | Role |
| --- | --- | --- |
| Core | `ag-core`, `ag-dsl`, `ag-cli`, `ag-lsp`, `ag-wasm-host`* | Shield/runtime, DSL compiler, developer CLI, editor LSP, WASI plugin host |
| Standard | `ag-auth`, `ag-data`, `ag-realtime`, `ag-cache`, `ag-storage`, `ag-observe` | Auth, PostgreSQL data layer, events, cache, files/images and telemetry |
| Deferred standard | `ag-mail`, `ag-workers` | Transactional mail and native MTA; background-job execution engine |
| Optional | `ag-ui`*, `ag-cloud`*, `ag-ai`*, `ag-mobile`*, `ag-migrate`* | UI/SSR, cloud deploy, AI/knowledge-graph, mobile bridge and importers |
| Optional infra | `ag-domains`, `ag-edge` | DNS/domain/TLS control plane and the request-time edge data plane |

## English

### What is usable today

- HTTP security and serving through `ag-core` Shield.
- PostgreSQL pools and migrations through `ag-data`.
- DSL parsing, diagnostics, SQL/Rust/TypeScript/OpenAPI/AsyncAPI generation through `ag-dsl`.
- Project scaffolding and development commands through `ag-cli`.
- Auth, cache, realtime, storage and observability modules from Phase 4.
  (`ag-ui`, `ag-ai`, `ag-mobile`, `ag-cloud`, `ag-migrate` and `ag-wasm-host`
  are placeholder crates whose implementation starts in later phases.)
- Transactional mail and the implemented domain-management surface from Phase 4.5.
- The opt-in native outbound MTA and signed webhooks in `ag-mail` (Phase 4.6-A/B/C features `mta`/`api`).
- Background jobs through `ag-workers` (Phase 4.6-D): typed jobs, retries, DLQ, scheduling and worker pools on the in-memory backend by default, durable PostgreSQL backend opt-in. Live-database parity verification is tracked in GitHub Issues #108/#109/#103.

Every crate remains independently selectable. Later roadmap phases are additive and are not required to use the capabilities above.

### Install

Prerequisites: Git and Rust 1.95.0 or newer.

Linux or macOS, from a blank working directory:

```bash
git clone https://github.com/Anti-Gravital/Anti-Gravital.git && cd Anti-Gravital && bash install.sh
```

Windows PowerShell:

```powershell
git clone https://github.com/Anti-Gravital/Anti-Gravital.git; Set-Location Anti-Gravital; .\install.ps1
```

The installer verifies the Rust version, builds the workspace in release mode and installs `ag` into the Cargo bin directory. It performs no privileged system changes.
Before execution, verify the checkout using the canonical [installation integrity procedure](docs/security/INSTALLATION_INTEGRITY.md).

### Start a project

```bash
ag new my-api --template rest
cd my-api
ag dev
```

Available templates: `rest`, `realtime`, `fullstack`.

### CLI

| Command | Purpose |
| --- | --- |
| `ag new NAME --template TYPE` | Scaffold a project |
| `ag dev --bind 0.0.0.0:8080` | Run in development mode; uses cargo-watch when installed |
| `ag build [--target TRIPLE]` | Build a release binary |
| `ag generate --schema schema.ag --output generated` | Generate DSL artifacts |
| `ag schema lint` | Validate a DSL schema and report diagnostics |
| `ag schema diff REFERENCE` | Classify schema changes |
| `ag mail test --to ADDRESS` | Verify SMTP configuration |
| `ag domains check --domain HOST` | Check DNS propagation |
| `ag domains sync --zone-id ID` | Apply schema DNS records through the configured provider |
| `ag domains attach`|instructions|export-zone|status|list|verify|detach|diagnose` | Operate the implemented local domain attachment workflow |
| `ag workers list` | List the background workers declared in a schema (ag-workers, RFC-0012) |
| `ag workers run` | Run a standalone worker process against the configured backend |
| `ag workers enqueue KIND --payload FILE` | Enqueue a job onto the durable backend (needs `DATABASE_URL`) |
| `ag workers queues` | Show queue depths on the durable backend |
| `ag workers dlq list\|inspect\|retry\|purge` | Inspect and manage the dead-letter queue; `retry`/`purge` accept `--queue/--kind/--limit/--dry-run` for bounded bulk operations (RFC-0017) |
| `ag workers doctor` | Check workers config and durable-backend connectivity |

Run `ag COMMAND --help` for authoritative flags and environment variables.
`ag deploy`, `ag migrate`, and `ag plugin` are future commands and are not
available in the current binary.
See the [`ag-cli` command guide](crates/ag-cli/README.md) and the
[domain CLI reference](docs/ag-domains/reference/cli.md) for detailed workflows.

### DSL workflow

```bash
ag schema lint --schema schema.ag
ag generate --schema schema.ag --output generated
```

The generator writes a Rust module, SQL migration, TypeScript types/client, OpenAPI and optional AsyncAPI artifacts. Generated handler bodies are deliberate stubs and must be implemented by the application. Rust-side `@regex` request validation is executable and caches each compiled pattern. Generated projects using it must declare `regex = "1"`.

### Evidence-based roadmap

The roadmap has 10 main phases plus the additive Phase 4.5 (ADR-0007) and the
additive pre-Phase-5 extraction/hardening Phase 4.6 (ADR-0010 for the `ag-mail`
native MTA sub-phases A/B/C; RFC-0012/ADR-0013 for the `ag-workers` sub-phase D).
Later phases expand the ecosystem but are not required to use the implemented
capabilities. Durations are planning estimates, not release promises.

| Phase | Delivered repository capability | Current evidence state | Remaining gate work |
| --- | --- | --- | --- |
| 0 | Governance, Apache-2.0, monorepo, CI and technical constitution | Repository deliverables present | External branding, community and public calendar criteria remain |
| 1 | Shield HTTP/TLS/auth/rate-limit/validation pipeline | Implementation and tests available | Reference performance, coverage certification and external adoption criteria remain |
| 2 | Core extractors/responses, PostgreSQL data layer, scaffolds and CRUD example | Implementation available; measured benchmarks published | Published 40K req/s and p99 targets were not met on recorded hardware |
| 3 | DSL v0.1-v0.4, generators, LSP and VS Code extension | Broad parser/generator coverage; consolidation issue #70 open | 24-hour fuzz gate, direct generated-vs-manual benchmark and generator completeness |
| 4 | Standard auth/cache/realtime/storage/observe modules | Modules and tests available; realtime/cache hardening included in this audit. UI/AI/mobile/WASM crates remain placeholders for Phases 4+/6/8/9 | Manual scale/performance evidence and remaining documented debt |
| 4.5 | Transactional mail plus implemented DNS/TLS/domain management surface | Code and cross-module tests exist; `ag-domains` is under active development | Reconcile active domain work, release evidence and documentation before claiming completion |
| 4.6 | Additive pre-Phase-5 hardening: native outbound MTA + signed webhooks in `ag-mail` (A/B/C) and the `ag-workers` background engine (D) | S1-S5 of `ag-workers` implemented and CI-verified (DSL `worker`, CLI, 5 examples, benchmarks, fuzz target, coverage gate green); MTA implemented behind `mta`/`api` features | Live-PostgreSQL parity and integration runs (Issues #108/#109/#103), `ag-edge` producer wiring (#112), MTA durable spool and live-delivery evidence |
| 5 | `ag-cloud`: simplified build/deploy, secrets, logs, rollback, domains and TLS | Pending; not required for Phase 0-4.5 usage | Pre-Phase 5 gate; public beta v0.5 milestone |
| 6 | `ag-ai` and Knowledge Graph: providers/models, retrieval and graph-assisted backend workflows | Pending roadmap phase; existing crate work is not phase completion | Phase 5 completion and beta feedback |
| 7 | `ag-migrate`: importers and assisted migration from existing backend frameworks | Pending roadmap phase | Phase 6 completion and importer acceptance tests |
| 8 | `ag-mobile`: Flutter/Dart bridge, generated clients and offline/mobile contracts | Pending roadmap phase | Phase 7 completion and mobile compatibility gates |
| 9 | WASI plugins: sandboxed extensions, lifecycle hooks, permissions and registry | Pending roadmap phase | Phase 8 completion and security review |
| 10 | Hardening and 1.0: stable API/DSL, security audit, performance, docs, LTS and ecosystem readiness | Pending; stable 1.0 milestone | Phase 9 completion and 1.0 release gates |

The formal status is maintained in [docs/roadmap/STATUS.md](docs/roadmap/STATUS.md). The release decision is maintained in [docs/audits/PRE_FASE5_RELEASE_GATE.md](docs/audits/PRE_FASE5_RELEASE_GATE.md). Open technical debt is tracked as GitHub Issues (label `tech-debt`, CLAUDE.md rule 29); [docs/DEBT.md](docs/DEBT.md) is a frozen historical record.

Estimated public beta: end of Phase 5, around month 15 of the original plan.
Estimated stable 1.0: end of Phase 10, around month 30. See the
[roadmap index](docs/roadmap/README.md) and [calendar](docs/roadmap/calendar.md)
for the complete phase documents and exit criteria.

### Known release blockers

- The 24-hour fuzz gate is still pending.
- Stabilized performance evidence is pending.
- The release gate must be reevaluated on the final consolidation commit.
- Remaining open debt must be accepted, fixed or explicitly deferred before a production-ready claim.

### Contributing and security

Read [CONTRIBUTING.md](CONTRIBUTING.md), [CLAUDE.md](CLAUDE.md), [SECURITY.md](SECURITY.md) and [GOVERNANCE.md](GOVERNANCE.md) before opening a change.

## Espanol

Anti-Gravital es un framework backend modular y nativo en Rust para construir
servicios seguros y de alto rendimiento con un flujo schema-first. El
repositorio es un workspace Cargo compuesto por crates `ag-*` y la CLI de
desarrollo `ag`.

El objetivo es ofrecer una experiencia de framework completo sin ocultar el
ecosistema Rust. Combina un runtime HTTP seguro, el Anti-DSL (`.ag`),
generacion de Rust/SQL/TypeScript/OpenAPI/AsyncAPI, una CLI unificada y crates
modulares para auth, cache, realtime, storage, observabilidad, correo,
dominios, jobs en segundo plano y edge routing. UI, IA, mobile, migraciones,
cloud deploy y plugins WASI son crates placeholder reservados para fases
posteriores.

No reemplaza PostgreSQL, Redis, NATS, object storage, Docker, Kubernetes,
Terraform, Flutter ni frameworks frontend. `ag-mail` envia correo
transaccional, no aloja buzones IMAP/POP. `ag-domains` gestiona DNS, dominios y
TLS, pero no es un registrador. Los handlers generados son stubs que implementa
cada aplicacion.

### Mapa de arquitectura

Los niveles siguen la clasificacion canonica de crates de `CLAUDE.md` §14 y
el capitulo 5 del maestro de arquitectura. `*` marca un crate placeholder
cuya implementacion comienza en una fase posterior.

| Nivel | Crates | Rol |
| --- | --- | --- |
| Nucleo | `ag-core`, `ag-dsl`, `ag-cli`, `ag-lsp`, `ag-wasm-host`* | Shield/runtime, compilador DSL, CLI, LSP de editor y host de plugins WASI |
| Estandar | `ag-auth`, `ag-data`, `ag-realtime`, `ag-cache`, `ag-storage`, `ag-observe` | Auth, capa de datos PostgreSQL, eventos, cache, archivos/imagenes y telemetria |
| Estandar diferido | `ag-mail`, `ag-workers` | Correo transaccional y MTA nativo; motor de ejecucion de jobs en segundo plano |
| Opcional | `ag-ui`*, `ag-cloud`*, `ag-ai`*, `ag-mobile`*, `ag-migrate`* | UI/SSR, deploy cloud, IA/knowledge-graph, bridge mobile e importadores |
| Opcional infra | `ag-domains`, `ag-edge` | Plano de control DNS/dominios/TLS y plano de datos edge en tiempo de request |

> Estado actual (verificado 2026-06-10): las capacidades hasta la Fase 4.5 mas el trabajo aditivo de la Fase 4.6 (MTA outbound nativo en `ag-mail`; motor de ejecucion en segundo plano `ag-workers`, etapas S1-S5) estan disponibles, pero la puerta formal pre-Fase 5 sigue ABIERTA. Modulo implementado no significa certificacion de produccion.

### Disponible hoy

- Seguridad HTTP y serving con Shield en `ag-core`.
- Pools PostgreSQL y migraciones con `ag-data`.
- DSL, diagnostics y generacion SQL/Rust/TypeScript/OpenAPI/AsyncAPI con `ag-dsl`.
- Scaffolding, desarrollo y build con `ag-cli`.
- Modulos estandar de autenticacion, cache, realtime, storage y observabilidad.
- Correo transaccional y la superficie de dominios ya implementada en Fase 4.5.
- MTA outbound nativo y webhooks firmados de `ag-mail` (features `mta`/`api`, Fase 4.6-A/B/C).
- Jobs en segundo plano con `ag-workers` (Fase 4.6-D): jobs tipados, reintentos, DLQ, scheduling y pools sobre el backend en memoria por defecto; backend PostgreSQL durable opt-in. La verificacion contra base de datos viva se rastrea en los Issues #108/#109/#103.

Las fases posteriores son aditivas: no son requisito para usar lo anterior.

### Instalar

Requisitos: Git y Rust 1.95.0 o superior.

Linux o macOS:

```bash
git clone https://github.com/Anti-Gravital/Anti-Gravital.git && cd Anti-Gravital && bash install.sh
```

Windows PowerShell:

```powershell
git clone https://github.com/Anti-Gravital/Anti-Gravital.git; Set-Location Anti-Gravital; .\install.ps1
```

El instalador verifica Rust, compila el workspace en release e instala `ag` en el directorio binario de Cargo. No realiza cambios privilegiados del sistema.
Antes de ejecutarlo, verifica el checkout mediante el [procedimiento canonico de integridad](docs/security/INSTALLATION_INTEGRITY.md).

### Crear y operar un proyecto

```bash
ag new mi-api --template rest
cd mi-api
ag dev
```

Para trabajar con el DSL:

```bash
ag schema lint --schema schema.ag
ag generate --schema schema.ag --output generated
```

Los comandos locales de dominios disponibles son `attach`, `instructions`,
`export-zone`, `status`, `list`, `verify`, `detach` y `diagnose`. Los comandos
futuros `ag deploy`, `ag migrate` y `ag plugin` no estan disponibles.

### Roadmap y calendario completo

La hoja de ruta tiene 10 fases principales mas las fases aditivas 4.5 (ADR-0007)
y 4.6 (ADR-0010 para el MTA nativo de `ag-mail`, sub-fases A/B/C;
RFC-0012/ADR-0013 para `ag-workers`, sub-fase D). Las fases posteriores amplian
el ecosistema, pero no son requisito para usar lo ya implementado. Las
duraciones son estimaciones, no promesas de fecha.

| Fase | Objetivo | Duracion | Estado actual |
| --- | --- | --- | --- |
| 0 | Fundaciones, gobernanza, licencia, CI, RFC/ADR y comunidad | 2 meses | Base del repositorio presente; faltan criterios externos |
| 1 | Shield MVP: HTTP/TLS, JWT, limites, CORS, CSRF, validacion y logging | 2 meses | Implementacion disponible; faltan gates finales |
| 2 | Core MVP: HTTP tipado, PostgreSQL, migraciones, scaffolds y CRUD | 2 meses | Implementacion disponible; targets publicados siguen como gate |
| 3 | Anti-DSL v0.1-v0.4, generadores, LSP y VS Code | 3 meses | Implementacion amplia; faltan fuzzing/adopcion y gaps del generador |
| 4 | Modulos estandar auth, cache, realtime, storage y observabilidad | 3 meses | Modulos disponibles; UI/IA/mobile/WASM siguen como placeholders de fases posteriores; quedan escala y deuda documentada |
| 4.5 | Correo transaccional, DNS, dominios, TLS/ACME y attachments | Aditiva | Capacidades disponibles; `ag-domains` tiene trabajo activo |
| 4.6 | Endurecimiento aditivo pre-Fase 5: MTA nativo + webhooks en `ag-mail` (A/B/C) y motor `ag-workers` (D) | Aditiva | S1-S5 de `ag-workers` implementadas y verificadas en CI; paridad PostgreSQL viva en Issues #108/#109/#103 |
| 5 | `ag-cloud`: deploy, secretos, logs, rollback, dominios y TLS simplificados | 3 meses | Pendiente; hito beta publica v0.5 |
| 6 | `ag-ai` y Knowledge Graph: providers, retrieval y flujos asistidos | 3 meses | Pendiente; el crate existente no implica cierre de fase |
| 7 | `ag-migrate`: importadores y migracion asistida desde otros frameworks | 2 meses | Pendiente |
| 8 | `ag-mobile`: bridge Flutter/Dart, clientes y sincronizacion offline | 2 meses | Pendiente |
| 9 | Plugins WASI: aislamiento, hooks, permisos y registry | 3 meses | Pendiente |
| 10 | Endurecimiento y 1.0: API/DSL estable, auditoria, rendimiento, docs y LTS | 3 meses | Pendiente; hito estable 1.0 |

Beta publica estimada: final de Fase 5, alrededor del mes 15 del plan original.
Version 1.0 estable estimada: final de Fase 10, alrededor del mes 30. Consulta
`docs/roadmap/README.md`, `docs/roadmap/calendar.md`,
`docs/roadmap/STATUS.md` y `docs/audits/PRE_FASE5_RELEASE_GATE.md`. La deuda
tecnica abierta vive en GitHub Issues (etiqueta `tech-debt`); `docs/DEBT.md`
queda congelado como registro historico.

### Limitaciones que no se ocultan

- El fuzzing manual de 24 horas sigue pendiente.
- Falta evidencia final de rendimiento sobre codigo estabilizado.
- Los handlers Rust generados son stubs intencionales.
- La validacion Rust para `@regex` es ejecutable y cachea cada patron compilado; los proyectos generados que la usen deben declarar `regex = "1"`.
- `ag-domains` tiene trabajo activo y esta auditoria no modifica su implementacion.

## License / Licencia

Apache License 2.0. See [LICENSE](LICENSE).

Project initiated by Angel Nereira under Gravital Labs / Nereira Technology and Business Solutions.
