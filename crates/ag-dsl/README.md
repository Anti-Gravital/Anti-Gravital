# ag-dsl

> Estado: Fase 0 - Vacio. La implementacion comienza en Fase 3 (alpha v0.1..v0.4) a Fase 10 (v1.0).
> Criticidad: Nucleo.
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
