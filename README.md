# Anti-Gravital

Rust-native, modular backend framework. The repository is a Cargo workspace of `ag-*` crates plus the `ag` developer CLI.

English is the canonical project language. A concise Spanish version follows.

> Current status (verified 2026-06-08): capabilities through Phase 4.5 are available, but the pre-Phase 5 release gate is still OPEN. Do not interpret implemented modules as a production-readiness certification.

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

Prerequisites: Git and Rust 1.79.0 or newer.

Linux or macOS, from a blank working directory:

```bash
git clone https://github.com/Anti-Gravital/Anti-Gravital.git && cd Anti-Gravital && bash install.sh
```

Windows PowerShell:

```powershell
git clone https://github.com/Anti-Gravital/Anti-Gravital.git; Set-Location Anti-Gravital; .\install.ps1
```

The installer verifies the Rust version, builds the workspace in release mode and installs `ag` into the Cargo bin directory. It performs no privileged system changes.

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
| `ag domains attach|instructions|export-zone|status|list|verify|detach` | Operate the implemented local domain attachment workflow |

Run `ag COMMAND --help` for authoritative flags and environment variables.

### DSL workflow

```bash
ag schema lint --schema schema.ag
ag generate --schema schema.ag --output generated
```

The generator writes a Rust module, SQL migration, TypeScript types/client, OpenAPI and optional AsyncAPI artifacts. Generated handler bodies are deliberate stubs and must be implemented by the application. Rust-side `@regex` request validation is not yet emitted as executable code; track this under issue #70.

### Evidence-based roadmap

| Phase | Delivered repository capability | Current evidence state | Remaining gate work |
| --- | --- | --- | --- |
| 0 | Governance, Apache-2.0, monorepo, CI and technical constitution | Repository deliverables present | External branding, community and public calendar criteria remain |
| 1 | Shield HTTP/TLS/auth/rate-limit/validation pipeline | Implementation and tests available | Reference performance, coverage certification and external adoption criteria remain |
| 2 | Core extractors/responses, PostgreSQL data layer, scaffolds and CRUD example | Implementation available; measured benchmarks published | Published 40K req/s and p99 targets were not met on recorded hardware |
| 3 | DSL v0.1-v0.4, generators, LSP and VS Code extension | Broad parser/generator coverage; consolidation issue #70 open | 24-hour fuzz gate, direct generated-vs-manual benchmark and generator completeness |
| 4 | Standard auth/cache/realtime/storage/observe/UI/AI/mobile/WASM modules | Modules and tests available; realtime/cache hardening included in this audit | Manual scale/performance evidence and remaining documented debt |
| 4.5 | Transactional mail plus implemented DNS/TLS/domain management surface | Code and cross-module tests exist; `ag-domains` is under active development | Reconcile active domain work, release evidence and documentation before claiming completion |
| 5+ | Cloud and later additive capabilities | Not required for Phases 0-4.5 usage | Blocked from release advancement while the formal gate remains open |

The formal status is maintained in [docs/roadmap/STATUS.md](docs/roadmap/STATUS.md). The release decision is maintained in [docs/audits/PRE_FASE5_RELEASE_GATE.md](docs/audits/PRE_FASE5_RELEASE_GATE.md). Open technical debt is maintained in [docs/DEBT.md](docs/DEBT.md).

### Known release blockers

- The 24-hour fuzz gate is still pending.
- Stabilized performance evidence is pending.
- The release gate must be reevaluated on the final consolidation commit.
- Remaining open debt must be accepted, fixed or explicitly deferred before a production-ready claim.

### Contributing and security

Read [CONTRIBUTING.md](CONTRIBUTING.md), [CLAUDE.md](CLAUDE.md), [SECURITY.md](SECURITY.md) and [GOVERNANCE.md](GOVERNANCE.md) before opening a change.

## Espanol

Anti-Gravital es un framework backend modular y nativo en Rust. El repositorio es un workspace Cargo compuesto por crates `ag-*` y la CLI de desarrollo `ag`.

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

Requisitos: Git y Rust 1.79.0 o superior.

Linux o macOS:

```bash
git clone https://github.com/Anti-Gravital/Anti-Gravital.git && cd Anti-Gravital && bash install.sh
```

Windows PowerShell:

```powershell
git clone https://github.com/Anti-Gravital/Anti-Gravital.git; Set-Location Anti-Gravital; .\install.ps1
```

El instalador verifica Rust, compila el workspace en release e instala `ag` en el directorio binario de Cargo. No realiza cambios privilegiados del sistema.

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

La tabla de roadmap en ingles es bilingue por contenido y representa el estado verificable. Las fuentes normativas son `docs/roadmap/STATUS.md`, `docs/audits/PRE_FASE5_RELEASE_GATE.md` y `docs/DEBT.md`.

### Limitaciones que no se ocultan

- El fuzzing manual de 24 horas sigue pendiente.
- Falta evidencia final de rendimiento sobre codigo estabilizado.
- Los handlers Rust generados son stubs intencionales.
- La validacion Rust ejecutable para `@regex` sigue pendiente en el issue #70.
- `ag-domains` tiene trabajo activo y esta auditoria no modifica su implementacion.

## License / Licencia

Apache License 2.0. See [LICENSE](LICENSE).

Project initiated by Angel Nereira under Gravital Labs / Nereira Technology and Business Solutions.
