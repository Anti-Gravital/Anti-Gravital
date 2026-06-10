# ag-dsl

> Status: Phase 3 — implemented (alpha v0.1..v0.8, including the `worker`
> declaration of RFC-0012). Functional compiler: logos lexer, chumsky parser,
> type/semantic checks, readable diagnostics, and codegen to Rust, SQL, OpenAPI
> and TypeScript. Syntax may change between v0.x releases. Remaining gaps are
> tracked as GitHub Issues (label `tech-debt`).
> Criticality: Core.
> Capitulo de arquitectura: docs/architecture/07-anti-dsl.md

## Dominio

Compilador del Anti-DSL (archivos .ag). Implementa lexer con logos, parser con chumsky, sistema de tipos, diagnostics legibles, y generadores de codigo a Rust, SQL, OpenAPI, TypeScript y Dart. Entregado por subversiones v0.1..v1.0.

## Referencias

- Documento maestro de arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`.
- Capitulo navegable: `docs/architecture/07-anti-dsl.md`.
- Hoja de ruta del crate: `docs/modules/ag-dsl/README.md`.
- Constitucion tecnica: `CLAUDE.md`.

## Reglas aplicables

- Este crate cumple las reglas 14 y 15 de `CLAUDE.md` sobre crates y
  dependencias.
- Versionado semantico independiente del resto del workspace una vez
  publicado.
- Sin `unsafe` salvo justificacion via RFC bajo `docs/rfc/`.
