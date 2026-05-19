# ag-data

> Estado: Fase 0 - Vacio. La implementacion comienza en Fase 2 (CRUD basico) a Fase 4 (avanzado).
> Criticidad: Estandar.
> Capitulo de arquitectura: docs/architecture/08-modulos-batteries-included.md

## Dominio

Capa de datos: pool de conexiones PostgreSQL con sqlx (queries verificadas en tiempo de compilacion), migraciones embebidas con sqlx::migrate!, ORM tipado generado por el DSL, transacciones declarativas, soporte para row-level security y multi-tenancy.

## Referencias

- Documento maestro de arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`.
- Capitulo navegable: `docs/architecture/08-modulos-batteries-included.md`.
- Hoja de ruta del crate: `docs/modules/ag-data/README.md`.
- Constitucion tecnica: `CLAUDE.md`.

## Reglas aplicables

- Este crate cumple las reglas 14 y 15 de `CLAUDE.md` sobre crates y
  dependencias.
- Versionado semantico independiente del resto del workspace una vez
  publicado.
- Sin `unsafe` salvo justificacion via RFC bajo `docs/rfc/`.
