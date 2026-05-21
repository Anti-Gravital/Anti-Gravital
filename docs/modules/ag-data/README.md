# ag-data

> Capitulo de arquitectura: `docs/architecture/08-modulos-batteries-included.md`.
> README del crate: `crates/ag-data/README.md`.
> Criticidad: Estandar.
> Fase de implementacion: Fase 2 a Fase 4.

## Dominio

Capa de datos con sqlx, migraciones embebidas y ORM tipado generado por el DSL.

## Dependencias internas permitidas

Depende de ag-core. No depende de ag-auth.

## Reglas aplicables

Vease las reglas 14 y 15 de `CLAUDE.md` para la separacion entre
nucleo, estandar y opcional, y para la politica de dependencias entre
crates Anti-Gravital.

## Estado

Fase 2 MVP completada. El crate contiene la capa de datos minima
necesaria para el Core MVP.

Implementado: `DataConfig` con `url`, `max_connections` y defaults
configurables via `DATABASE_MAX_CONNECTIONS`. `DbPool` (alias de
`sqlx::PgPool`). `connect()` que establece el pool con los parametros
de `DataConfig`. `run_migrations()` que ejecuta un `sqlx::Migrator`
sobre el pool. `DataError` con conversion automatica a `AgError`.

Demostrado en `examples/todo-api/`: migraciones embebidas con
`sqlx::migrate!`, CRUD completo contra PostgreSQL real y despliegue
como binario estatico MUSL (5.3 MB) en imagen `FROM scratch` (2.49 MB).

El ORM generado por DSL y las funcionalidades avanzadas (relaciones,
paginacion, caches de queries) llegan en fases posteriores.
