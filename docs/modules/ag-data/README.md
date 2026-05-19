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

Fase 0: el crate `crates/ag-data/` esta declarado en el workspace con
`src/lib.rs` vacio y `README.md` propio. No contiene codigo
funcional. La implementacion comienza en la fase indicada arriba.
