# ag-migrate

> Capitulo de arquitectura: `docs/architecture/12-migracion-ag-migrate.md`.
> README del crate: `crates/ag-migrate/README.md`.
> Criticidad: Opcional.
> Fase de implementacion: Fase 7.

## Dominio

Importadores: OpenAPI, Prisma, Django, FastAPI, Sequelize, GraphQL SDL. Genera schema.ag.

## Dependencias internas permitidas

Depende de ag-dsl.

## Reglas aplicables

Vease las reglas 14 y 15 de `CLAUDE.md` para la separacion entre
nucleo, estandar y opcional, y para la politica de dependencias entre
crates Anti-Gravital.

## Estado

Fase 0: el crate `crates/ag-migrate/` esta declarado en el workspace con
`src/lib.rs` vacio y `README.md` propio. No contiene codigo
funcional. La implementacion comienza en la fase indicada arriba.
