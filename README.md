# Anti-Gravital

> Estado: Pre-lanzamiento. Fase 0 - Fundaciones y Gobernanza.
> Status: Pre-launch. Phase 0 - Foundations and Governance.

Anti-Gravital es un ecosistema de software libre para construir
aplicaciones backend de alto rendimiento en Rust puro, con tres
propiedades fundamentales: ausencia de runtime externo, enfoque
schema-first y arquitectura modular de crates independientes.

Anti-Gravital is an open source ecosystem for building high-performance
backend applications in pure Rust, with three core properties: no
external runtime, schema-first approach, and a modular architecture of
independent crates.

---

## En espanol

### Que es

Un runtime backend Rust de alto rendimiento, un lenguaje de definicion
de dominio llamado Anti-DSL (archivos `.ag`), una CLI unificada (`ag`),
un conjunto de modulos batteries-included publicados como crates
independientes, un sistema de plugins WASI, una capa de despliegue
simplificado, generadores de SDK tipados para TypeScript y Dart, e
importadores desde frameworks legacy.

### Que no es

No reemplaza Kubernetes. No reemplaza Flutter ni React Native. No
reemplaza Next.js. No reemplaza Docker. No reemplaza PostgreSQL,
Redis, MinIO ni NATS. No es un motor de juegos ni un framework de
computo cientifico. Vease el capitulo de alcance en
`docs/architecture/03-alcance-y-limites.md`.

### Estado del proyecto

El repositorio se encuentra en Fase 0 segun la
`docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`. La Fase 0 entrega
fundaciones y gobernanza: documentos maestros, licencia, gobernanza,
estructura del monorepo Cargo con 15 crates vacios, integracion
continua multiplataforma y plantillas de issue, pull request, RFC y
ADR. No hay codigo funcional todavia. El primer hito tecnico (Shield
MVP) llega en Fase 1.

### Fuente de verdad

Los tres documentos maestros viven en `docs/master/` y gobiernan toda
decision tecnica del proyecto:

- `ANTI-GRAVITAL-Blueprint-v4.0.pdf` - vision, posicionamiento y alcance.
- `ANTI-GRAVITAL-Arquitectura-Tecnica.md` - como se construye.
- `ANTI-GRAVITAL-Hoja-de-Ruta.md` - que se construye y cuando.

Esta documentacion se descompone en archivos navegables bajo
`docs/architecture/`, `docs/roadmap/`, `docs/modules/`, `docs/dsl/`,
`docs/security/`, `docs/governance/` y `docs/benchmarks/`. El contenido
de los derivados se copia verbatim del maestro correspondiente; si
existe divergencia, el maestro gana.

### Como contribuir

Vease `CONTRIBUTING.md` para la guia completa. Resumen rapido:

1. Lea los maestros bajo `docs/master/` y la fase actual en
   `docs/roadmap/`.
2. Para cambios arquitectonicos, abra una RFC en `docs/rfc/` antes de
   tocar codigo.
3. Mantenga sus pull requests cortas: titulo de hasta 256 caracteres y
   una unica unidad logica de cambio.
4. Pase `cargo fmt`, `cargo clippy -D warnings`, `cargo test`,
   `cargo audit` y `cargo deny check` antes de proponer cambios.

### Licencia

Apache 2.0. Vease `LICENSE`.

### Origen

Proyecto iniciado por Gravital Labs, division open source de Nereira
Technology and Business Solutions, Republica de Panama. Mantenedor
inicial: Angel Nereira.

---

## In English

### What it is

A high-performance Rust backend runtime, a domain definition language
called Anti-DSL (`.ag` files), a unified CLI (`ag`), a set of batteries
included modules published as independent crates, a WASI plugin
system, a simplified deployment layer, typed SDK generators for
TypeScript and Dart, and importers from legacy frameworks.

### What it is not

It does not replace Kubernetes, Flutter, React Native, Next.js, Docker,
PostgreSQL, Redis, MinIO or NATS. It is not a game engine or a
scientific computing framework. See the scope chapter at
`docs/architecture/03-alcance-y-limites.md`.

### Project status

The repository is in Phase 0 per
`docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`. Phase 0 delivers
foundations and governance: master documents, license, governance, a
Cargo workspace skeleton with 15 empty crates, multiplatform
continuous integration, and templates for issues, pull requests, RFCs
and ADRs. There is no functional code yet. The first technical
milestone (Shield MVP) ships in Phase 1.

### Source of truth

The three master documents live in `docs/master/` and govern every
technical decision:

- `ANTI-GRAVITAL-Blueprint-v4.0.pdf` - vision, positioning, scope.
- `ANTI-GRAVITAL-Arquitectura-Tecnica.md` - how the system is built.
- `ANTI-GRAVITAL-Hoja-de-Ruta.md` - what is built and when.

These documents are decomposed into navigable files under
`docs/architecture/`, `docs/roadmap/`, `docs/modules/`, `docs/dsl/`,
`docs/security/`, `docs/governance/` and `docs/benchmarks/`. The
content of the derivatives is copied verbatim from the corresponding
master; if a divergence appears, the master wins.

### How to contribute

See `CONTRIBUTING.md` for the full guide. Quick summary:

1. Read the masters in `docs/master/` and the current phase in
   `docs/roadmap/`.
2. For architectural changes, open an RFC in `docs/rfc/` before
   touching code.
3. Keep pull requests small: titles up to 256 characters and a single
   logical unit of change.
4. Run `cargo fmt`, `cargo clippy -D warnings`, `cargo test`,
   `cargo audit` and `cargo deny check` before submitting.

### License

Apache 2.0. See `LICENSE`.

### Origin

Project started by Gravital Labs, the open source division of Nereira
Technology and Business Solutions, Republic of Panama. Initial
maintainer: Angel Nereira.

---

## Calendario / Calendar

| Hito / Milestone                  | Fase / Phase | Estado / Status |
| --- | --- | --- |
| Fundaciones y gobernanza          | 0  | En curso / In progress |
| The Shield MVP                    | 1  | Pendiente / Pending |
| The Core MVP                      | 2  | Pendiente / Pending |
| Anti-DSL alpha                    | 3  | Pendiente / Pending |
| Modulos estandar                  | 4  | Pendiente / Pending |
| `ag-cloud` y version 0.5 beta     | 5  | Pendiente / Pending |
| `ag-ai` y Knowledge Graph         | 6  | Pendiente / Pending |
| `ag-migrate` importadores         | 7  | Pendiente / Pending |
| `ag-mobile` Flutter bridge        | 8  | Pendiente / Pending |
| Sistema de plugins WASI           | 9  | Pendiente / Pending |
| Endurecimiento y version 1.0      | 10 | Pendiente / Pending |

Detalle completo en `docs/roadmap/` y `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`.
