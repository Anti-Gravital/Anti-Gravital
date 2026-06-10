# Anti-Gravital — Blueprint v4.1

**[English](#in-english) | [Espanol](#en-espanol)**

- Version: 4.1 (markdown source)
- Organization: Gravital Labs — Nereira Technology and Business Solutions
- Origin: Republic of Panama
- License: Apache 2.0
- Status: living document; markdown source of record for the Blueprint.

> This markdown file is the versionable source of the Anti-Gravital
> Blueprint. The legacy `ANTI-GRAVITAL-Blueprint-v4.0.pdf` is the
> presentation artifact and is registered as explicit debt pending
> re-export (see `VERSION.md`). When the markdown and the PDF diverge,
> the markdown governs.

---

## In English

### 1. Vision

Anti-Gravital is an open source ecosystem for building high-performance
backend applications in pure Rust. It rests on three non-negotiable
properties:

1. **No external runtime.** Applications compile to a single static
   binary. No interpreter, no language VM, no mandatory sidecar.
2. **Schema-first.** A domain definition language (Anti-DSL, `.ag` files)
   is the source of truth. Rust, SQL, OpenAPI, TypeScript and Dart
   artifacts are generated from it.
3. **Modular crates.** Capabilities ship as independent, separately
   versioned crates so a project only depends on what it uses.

The guiding sentence of the project: build Anti-Gravital as real,
modular, verifiable and sustainable infrastructure; never as a demo
inflated by technical hype.

### 2. Positioning

Anti-Gravital is infrastructure, not a framework that tries to solve
everything. Where a dominant tool already exists (Kubernetes, Docker,
PostgreSQL, Redis, NATS, MinIO, Terraform, React, Next.js, Flutter),
Anti-Gravital integrates with it rather than reinventing it. The strategy
is intelligent integration through small traits with swappable adapters.

The project is born in Panama and targets Latin America as its first
adoption focus, while keeping English as the canonical language of code
and technical documentation to remain open to global contributors
(see `ADR-0008`).

### 3. Scope

In scope:

- A high-performance Rust backend runtime (the Shield + Core of `ag-core`).
- The Anti-DSL and its compiler (`ag-dsl`) with multi-target codegen.
- A unified CLI (`ag`).
- Batteries-included standard modules: `ag-auth`, `ag-data`,
  `ag-realtime`, `ag-cache`, `ag-storage`, `ag-observe`.
- Deferred-standard modules: `ag-mail` (transactional outbound email) and
  `ag-workers` (background job execution engine).
- Optional infrastructure modules: `ag-domains` (DNS, ACME, SPF/DKIM/DMARC)
  and `ag-edge` (request-time edge data plane).
- Optional modules planned for later phases: `ag-ui`, `ag-cloud`, `ag-ai`,
  `ag-mobile`, `ag-migrate`.
- A WASI plugin system.

Explicitly out of scope:

- `ag-mail` is not a full MTA: no inbound, no IMAP/POP, no mailboxes, no
  antispam, no IP reputation.
- `ag-domains` is not a registrar and does not replace Terraform.
- Anti-Gravital does not reinvent dominant infrastructure tools.

### 4. Ecosystem at a glance

The ecosystem started at 17 crates in four tiers and has grown additively to
20 with `ag-lsp`, `ag-edge` and `ag-workers`:

- **Core:** `ag-core`, `ag-dsl`, `ag-cli`, `ag-lsp`, `ag-wasm-host`.
- **Standard:** `ag-auth`, `ag-data`, `ag-realtime`, `ag-cache`,
  `ag-storage`, `ag-observe`.
- **Deferred standard:** `ag-mail` and `ag-workers` (standard maturity, not
  installed by default in official templates; added when a project needs
  them). `ag-mail` does not depend on `ag-auth`; `ag-workers` is consumed by
  `ag-mail`/`ag-cli` behind features, never the reverse.
- **Optional:** `ag-ui`, `ag-cloud`, `ag-ai`, `ag-mobile`, `ag-migrate`,
  and the optional-infrastructure crates `ag-domains` (consumed optionally
  by `ag-cloud` during `ag deploy`) and `ag-edge` (request-time data plane).

Dependency rules: `ag-core` depends on no Anti-Gravital crate; no circular
dependencies; `ag-auth -> ag-mail` (not the reverse); `ag-cloud ->
ag-domains` (non-rigid).

### 5. Delivery state (close of Phase 4.5)

Phases 1 through 4.5 have their technical implementation complete and
merged to `main`:

- Phase 1 — Shield MVP.
- Phase 2 — Core MVP + full request roundtrip + PostgreSQL CRUD.
- Phase 3 — Anti-DSL alpha (v0.1-v0.4), `ag-lsp`, VS Code plugin, fuzzing.
- Phase 4 — standard modules (`ag-auth`, `ag-cache`, `ag-realtime`,
  `ag-storage`, `ag-observe`) plus DSL v0.5-v0.6.
- Phase 4.5 — `ag-mail` and `ag-domains`, DSL v0.7 (mail/domain/template),
  `ag-auth -> ag-mail` integration, CLI subcommands, cross-module E2E tests.

Practical implementation decisions worth recording at blueprint level:

- WebAuthn in `ag-auth` is implemented with `ciborium` + `p256` +
  `ed25519-dalek` instead of `webauthn-rs`, for license compatibility
  (Apache-2.0). See `ADR-0006`.
- Mail templating ships as a `MailTemplate` trait plus a `StringTemplate`
  implementation, allowing any engine (askama, minijinja) to be plugged in.
- `ag-cache` ships L1 (moka); the native L2 (RESP2) is proposed in
  `RFC-0005` and not yet implemented.

Phase 5 (`ag-cloud`) is next. The public beta (v0.5) milestone remains at
the end of Phase 5.

### 6. Governance and source of truth

Documentation is the contract. The order of authority is: the Hoja de
Ruta (roadmap) defines what may be built and when; the Arquitectura
Tecnica defines how; this Blueprint defines vision, positioning and scope.
If code contradicts the documentation, the code is wrong. Large decisions
require an RFC; architectural decisions are recorded as ADRs.

---

## En espanol

### 1. Vision

Anti-Gravital es un ecosistema de software libre para construir
aplicaciones backend de alto rendimiento en Rust puro. Se apoya en tres
propiedades no negociables:

1. **Sin runtime externo.** Las aplicaciones compilan a un unico binario
   estatico. Sin interprete, sin VM de lenguaje, sin sidecar obligatorio.
2. **Schema-first.** Un lenguaje de definicion de dominio (Anti-DSL,
   archivos `.ag`) es la fuente de verdad. De el se generan los
   artefactos Rust, SQL, OpenAPI, TypeScript y Dart.
3. **Crates modulares.** Las capacidades se publican como crates
   independientes y versionados por separado, de modo que un proyecto
   solo depende de lo que usa.

La frase rectora del proyecto: construir Anti-Gravital como infraestructura
real, modular, verificable y sostenible; nunca como una demo inflada por
hype tecnico.

### 2. Posicionamiento

Anti-Gravital es infraestructura, no un framework que intenta resolverlo
todo. Donde ya existe una herramienta dominante (Kubernetes, Docker,
PostgreSQL, Redis, NATS, MinIO, Terraform, React, Next.js, Flutter),
Anti-Gravital se integra en lugar de reinventarla. La estrategia es
integracion inteligente mediante traits pequenos con adapters
intercambiables.

El proyecto nace en Panama y tiene a Latinoamerica como primer foco de
adopcion, manteniendo el ingles como idioma canonico del codigo y de la
documentacion tecnica para permanecer abierto a contribuidores globales
(vease `ADR-0008`).

### 3. Alcance

Dentro del alcance:

- Un runtime backend Rust de alto rendimiento (Shield + Core de `ag-core`).
- El Anti-DSL y su compilador (`ag-dsl`) con codegen multi-objetivo.
- Una CLI unificada (`ag`).
- Modulos estandar batteries-included: `ag-auth`, `ag-data`,
  `ag-realtime`, `ag-cache`, `ag-storage`, `ag-observe`.
- Modulos estandar diferidos: `ag-mail` (correo transaccional outbound) y
  `ag-workers` (motor de ejecucion de jobs en segundo plano).
- Modulos opcionales de infraestructura: `ag-domains` (DNS, ACME,
  SPF/DKIM/DMARC) y `ag-edge` (plano de datos edge en tiempo de request).
- Modulos opcionales planeados para fases posteriores: `ag-ui`,
  `ag-cloud`, `ag-ai`, `ag-mobile`, `ag-migrate`.
- Un sistema de plugins WASI.

Explicitamente fuera del alcance:

- `ag-mail` no es un MTA completo: sin inbound, sin IMAP/POP, sin buzones,
  sin antispam, sin reputacion de IP.
- `ag-domains` no es un registrador y no reemplaza Terraform.
- Anti-Gravital no reinventa herramientas de infraestructura dominantes.

### 4. El ecosistema de un vistazo

El ecosistema arranco en 17 crates en cuatro niveles y ha crecido de forma
aditiva a 20 con `ag-lsp`, `ag-edge` y `ag-workers`:

- **Nucleo:** `ag-core`, `ag-dsl`, `ag-cli`, `ag-lsp`, `ag-wasm-host`.
- **Estandar:** `ag-auth`, `ag-data`, `ag-realtime`, `ag-cache`,
  `ag-storage`, `ag-observe`.
- **Estandar diferido:** `ag-mail` y `ag-workers` (madurez de estandar, no
  instalados por defecto en los templates oficiales; se incorporan cuando el
  proyecto los necesita). `ag-mail` no depende de `ag-auth`; `ag-workers` lo
  consumen `ag-mail`/`ag-cli` detras de features, nunca al reves.
- **Opcionales:** `ag-ui`, `ag-cloud`, `ag-ai`, `ag-mobile`, `ag-migrate`,
  y los crates opcionales de infraestructura `ag-domains` (consumido
  opcionalmente por `ag-cloud` durante `ag deploy`) y `ag-edge` (plano de
  datos en tiempo de request).

Reglas de dependencia: `ag-core` no depende de ningun crate Anti-Gravital;
sin dependencias circulares; `ag-auth -> ag-mail` (no al reves);
`ag-cloud -> ag-domains` (no rigida).

### 5. Estado de entrega (cierre de la Fase 4.5)

Las fases 1 a 4.5 tienen su implementacion tecnica completa y mergeada a
`main`:

- Fase 1 — Shield MVP.
- Fase 2 — Core MVP + roundtrip completo + CRUD PostgreSQL.
- Fase 3 — Anti-DSL alpha (v0.1-v0.4), `ag-lsp`, plugin VS Code, fuzzing.
- Fase 4 — modulos estandar (`ag-auth`, `ag-cache`, `ag-realtime`,
  `ag-storage`, `ag-observe`) mas DSL v0.5-v0.6.
- Fase 4.5 — `ag-mail` y `ag-domains`, DSL v0.7 (mail/domain/template),
  integracion `ag-auth -> ag-mail`, subcomandos de CLI, tests E2E
  cross-module.

Decisiones practicas de implementacion dignas de registrar a nivel de
blueprint:

- WebAuthn en `ag-auth` se implementa con `ciborium` + `p256` +
  `ed25519-dalek` en lugar de `webauthn-rs`, por compatibilidad de
  licencia (Apache-2.0). Vease `ADR-0006`.
- El templating de correo se ofrece como trait `MailTemplate` mas una
  implementacion `StringTemplate`, permitiendo conectar cualquier motor
  (askama, minijinja).
- `ag-cache` entrega L1 (moka); el L2 nativo (RESP2) esta propuesto en
  `RFC-0005` y aun no implementado.

La Fase 5 (`ag-cloud`) es la proxima. El hito de beta publica (v0.5)
permanece al final de la Fase 5.

### 6. Gobernanza y fuente de verdad

La documentacion es el contrato. El orden de autoridad es: la Hoja de Ruta
define que se puede construir y cuando; la Arquitectura Tecnica define
como; este Blueprint define vision, posicionamiento y alcance. Si el codigo
contradice la documentacion, el codigo esta mal. Toda decision grande
requiere un RFC; las decisiones arquitectonicas se registran como ADRs.
