# ag-data

> Status: Phase 2 — implemented (base layer). PostgreSQL connection pool via sqlx
> (`DataConfig`, pool, URL sanitization) and embedded migrations (`sqlx::migrate!`).
> Pending: DSL-generated typed ORM (Phase 3), row-level security and multi-tenancy
> (later phases). See `docs/DEBT.md`.
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
