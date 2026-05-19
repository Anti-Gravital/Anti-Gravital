# ag-cache

> Estado: Fase 0 - Vacio. La implementacion comienza en Fase 4.
> Criticidad: Estandar.
> Capitulo de arquitectura: docs/architecture/08-modulos-batteries-included.md

## Dominio

Cache de dos niveles: L1 con moka en memoria del proceso, L2 opcional con Redis via fred. Invalidacion por evento, estampillas de version, y soporte para patrones cache-aside y read-through declarados desde el DSL.

## Referencias

- Documento maestro de arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`.
- Capitulo navegable: `docs/architecture/08-modulos-batteries-included.md`.
- Hoja de ruta del crate: `docs/modules/ag-cache/README.md`.
- Constitucion tecnica: `CLAUDE.md`.

## Reglas aplicables

- Este crate cumple las reglas 14 y 15 de `CLAUDE.md` sobre crates y
  dependencias.
- Versionado semantico independiente del resto del workspace una vez
  publicado.
- Sin `unsafe` salvo justificacion via RFC bajo `docs/rfc/`.
