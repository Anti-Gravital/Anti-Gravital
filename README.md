# Anti-Gravital

Rust-native, modular backend framework for building secure, high-performance
backend services with a schema-first workflow. The repository is a Cargo
workspace of `ag-*` crates plus the `ag` developer CLI.

English is the canonical project language. A concise Spanish version follows.

> Current status (verified 2026-06-08): capabilities through Phase 4.5 are available, but the pre-Phase 5 release gate is still OPEN. Do not interpret implemented modules as a production-readiness certification.

Anti-Gravital gives Rust backend teams a coherent framework experience without
hiding the underlying Rust ecosystem. It combines:

- A secure HTTP runtime and Shield pipeline built on Axum, Tower, Tokio and rustls.
- A schema-first Anti-DSL (`.ag`) for models, endpoints, validation, auth,
  events, mail and domain declarations.
- Code generation for Rust, SQL migrations, TypeScript, OpenAPI and AsyncAPI.
- A unified CLI for scaffolding, development, builds, schema workflows, mail
  checks and domain operations.
- Modular crates for auth, cache, realtime, storage, observability, mail,
  domains, UI integration, AI, mobile, migration tooling and WASI plugins.

It does not replace PostgreSQL, Redis, NATS, object storage, Docker, Kubernetes,
Terraform, Flutter or frontend frameworks. `ag-mail` handles transactional
email, not IMAP/POP mailboxes. `ag-domains` manages DNS/domain/TLS workflows;
it is not a registrar. Generated Rust handlers are application-owned stubs.

### Architecture map

| Layer | Main crates | Role |
| --- | --- | --- |
| Core | `ag-core`, `ag-data`, `ag-dsl`, `ag-cli` | Shield/runtime, PostgreSQL, DSL compiler and developer workflows |
| Standard | `ag-auth`, `ag-cache`, `ag-realtime`, `ag-storage`, `ag-observe` | Auth, cache, events, files/images and telemetry |
| Extended | `ag-ui`, `ag-ai`, `ag-mobile`, `ag-migrate`, `ag-wasm-host`, `ag-lsp` | UI, AI, mobile, importers, plugins and editor support |
| Phase 4.5 / edge | `ag-mail`, `ag-domains`, `ag-edge` | Mail, DNS/domain/TLS, host routing and TLS serving |

## English

### What is usable today

- HTTP security and serving through `ag-core` Shield.
- PostgreSQL pools and migrations through `ag-data`.
- DSL parsing, diagnostics, SQL/Rust/TypeScript/OpenAPI/AsyncAPI generation through `ag-dsl`.
- Project scaffolding and development commands through `ag-cli`.
- Auth, cache, realtime, storage, observability, UI, AI, mobile and WASM-host modules from Phase 4.
- Transactional mail and the implemented domain-management surface from Phase 4.5.

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
| `ag domains attach|instructions|export-zone|status|list|verify|detach|diagnose` | Operate the implemented local domain attachment workflow |
| `ag workers list` | List the background workers declared in a schema (ag-workers, RFC-0012) |
| `ag workers enqueue KIND --payload FILE` | Enqueue a job onto the durable backend (needs `DATABASE_URL`) |
| `ag workers queues` | Show queue depths on the durable backend |
| `ag workers dlq list\|inspect\|retry\|purge` | Inspect and manage the dead-letter queue |
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

The roadmap has 10 main phases plus the additive Phase 4.5 introduced by
ADR-0007. Later phases expand the ecosystem but are not required to use the
implemented Phase 0-4.5 capabilities. Durations are planning estimates, not
release promises.

| Phase | Delivered repository capability | Current evidence state | Remaining gate work |
| --- | --- | --- | --- |
| 0 | Governance, Apache-2.0, monorepo, CI and technical constitution | Repository deliverables present | External branding, community and public calendar criteria remain |
| 1 | Shield HTTP/TLS/auth/rate-limit/validation pipeline | Implementation and tests available | Reference performance, coverage certification and external adoption criteria remain |
| 2 | Core extractors/responses, PostgreSQL data layer, scaffolds and CRUD example | Implementation available; measured benchmarks published | Published 40K req/s and p99 targets were not met on recorded hardware |
| 3 | DSL v0.1-v0.4, generators, LSP and VS Code extension | Broad parser/generator coverage; consolidation issue #70 open | 24-hour fuzz gate, direct generated-vs-manual benchmark and generator completeness |
| 4 | Standard auth/cache/realtime/storage/observe/UI/AI/mobile/WASM modules | Modules and tests available; realtime/cache hardening included in this audit | Manual scale/performance evidence and remaining documented debt |
| 4.5 | Transactional mail plus implemented DNS/TLS/domain management surface | Code and cross-module tests exist; `ag-domains` is under active development | Reconcile active domain work, release evidence and documentation before claiming completion |
| 5 | `ag-cloud`: simplified build/deploy, secrets, logs, rollback, domains and TLS | Pending; not required for Phase 0-4.5 usage | Pre-Phase 5 gate; public beta v0.5 milestone |
| 6 | `ag-ai` and Knowledge Graph: providers/models, retrieval and graph-assisted backend workflows | Pending roadmap phase; existing crate work is not phase completion | Phase 5 completion and beta feedback |
| 7 | `ag-migrate`: importers and assisted migration from existing backend frameworks | Pending roadmap phase | Phase 6 completion and importer acceptance tests |
| 8 | `ag-mobile`: Flutter/Dart bridge, generated clients and offline/mobile contracts | Pending roadmap phase | Phase 7 completion and mobile compatibility gates |
| 9 | WASI plugins: sandboxed extensions, lifecycle hooks, permissions and registry | Pending roadmap phase | Phase 8 completion and security review |
| 10 | Hardening and 1.0: stable API/DSL, security audit, performance, docs, LTS and ecosystem readiness | Pending; stable 1.0 milestone | Phase 9 completion and 1.0 release gates |

The formal status is maintained in [docs/roadmap/STATUS.md](docs/roadmap/STATUS.md). The release decision is maintained in [docs/audits/PRE_FASE5_RELEASE_GATE.md](docs/audits/PRE_FASE5_RELEASE_GATE.md). Open technical debt is maintained in [docs/DEBT.md](docs/DEBT.md).

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
dominios, UI, IA, mobile, migraciones y plugins WASI.

No reemplaza PostgreSQL, Redis, NATS, object storage, Docker, Kubernetes,
Terraform, Flutter ni frameworks frontend. `ag-mail` envia correo
transaccional, no aloja buzones IMAP/POP. `ag-domains` gestiona DNS, dominios y
TLS, pero no es un registrador. Los handlers generados son stubs que implementa
cada aplicacion.

### Mapa de arquitectura

| Capa | Crates principales | Rol |
| --- | --- | --- |
| Nucleo | `ag-core`, `ag-data`, `ag-dsl`, `ag-cli` | Shield/runtime, PostgreSQL, compilador DSL y workflows |
| Estandar | `ag-auth`, `ag-cache`, `ag-realtime`, `ag-storage`, `ag-observe` | Auth, cache, eventos, archivos/imagenes y telemetria |
| Extendida | `ag-ui`, `ag-ai`, `ag-mobile`, `ag-migrate`, `ag-wasm-host`, `ag-lsp` | UI, IA, mobile, importadores, plugins y editor |
| Fase 4.5 / edge | `ag-mail`, `ag-domains`, `ag-edge` | Correo, DNS/dominios/TLS, routing y serving TLS |

> Estado actual (verificado 2026-06-08): las capacidades hasta la Fase 4.5 estan disponibles, pero la puerta formal pre-Fase 5 sigue ABIERTA. Modulo implementado no significa certificacion de produccion.

### Disponible hoy

- Seguridad HTTP y serving con Shield en `ag-core`.
- Pools PostgreSQL y migraciones con `ag-data`.
- DSL, diagnostics y generacion SQL/Rust/TypeScript/OpenAPI/AsyncAPI con `ag-dsl`.
- Scaffolding, desarrollo y build con `ag-cli`.
- Modulos estandar de autenticacion, cache, realtime, storage y observabilidad.
- Correo transaccional y la superficie de dominios ya implementada en Fase 4.5.

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

La hoja de ruta tiene 10 fases principales mas la Fase 4.5 aditiva. Las fases
posteriores amplian el ecosistema, pero no son requisito para usar lo ya
implementado. Las duraciones son estimaciones, no promesas de fecha.

| Fase | Objetivo | Duracion | Estado actual |
| --- | --- | --- | --- |
| 0 | Fundaciones, gobernanza, licencia, CI, RFC/ADR y comunidad | 2 meses | Base del repositorio presente; faltan criterios externos |
| 1 | Shield MVP: HTTP/TLS, JWT, limites, CORS, CSRF, validacion y logging | 2 meses | Implementacion disponible; faltan gates finales |
| 2 | Core MVP: HTTP tipado, PostgreSQL, migraciones, scaffolds y CRUD | 2 meses | Implementacion disponible; targets publicados siguen como gate |
| 3 | Anti-DSL v0.1-v0.4, generadores, LSP y VS Code | 3 meses | Implementacion amplia; faltan fuzzing/adopcion y gaps del generador |
| 4 | Modulos auth, cache, realtime, storage, observabilidad, UI, IA, mobile y WASM | 3 meses | Modulos disponibles; quedan escala y deuda documentada |
| 4.5 | Correo transaccional, DNS, dominios, TLS/ACME y attachments | Aditiva | Capacidades disponibles; `ag-domains` tiene trabajo activo |
| 5 | `ag-cloud`: deploy, secretos, logs, rollback, dominios y TLS simplificados | 3 meses | Pendiente; hito beta publica v0.5 |
| 6 | `ag-ai` y Knowledge Graph: providers, retrieval y flujos asistidos | 3 meses | Pendiente; el crate existente no implica cierre de fase |
| 7 | `ag-migrate`: importadores y migracion asistida desde otros frameworks | 2 meses | Pendiente |
| 8 | `ag-mobile`: bridge Flutter/Dart, clientes y sincronizacion offline | 2 meses | Pendiente |
| 9 | Plugins WASI: aislamiento, hooks, permisos y registry | 3 meses | Pendiente |
| 10 | Endurecimiento y 1.0: API/DSL estable, auditoria, rendimiento, docs y LTS | 3 meses | Pendiente; hito estable 1.0 |

Beta publica estimada: final de Fase 5, alrededor del mes 15 del plan original.
Version 1.0 estable estimada: final de Fase 10, alrededor del mes 30. Consulta
`docs/roadmap/README.md`, `docs/roadmap/calendar.md`,
`docs/roadmap/STATUS.md`, `docs/audits/PRE_FASE5_RELEASE_GATE.md` y
`docs/DEBT.md`.

### Limitaciones que no se ocultan

- El fuzzing manual de 24 horas sigue pendiente.
- Falta evidencia final de rendimiento sobre codigo estabilizado.
- Los handlers Rust generados son stubs intencionales.
- La validacion Rust para `@regex` es ejecutable y cachea cada patron compilado; los proyectos generados que la usen deben declarar `regex = "1"`.
- `ag-domains` tiene trabajo activo y esta auditoria no modifica su implementacion.

## License / Licencia

Apache License 2.0. See [LICENSE](LICENSE).

Project initiated by Angel Nereira under Gravital Labs / Nereira Technology and Business Solutions.
