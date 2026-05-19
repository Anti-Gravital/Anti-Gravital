# ag-cache

> Capitulo de arquitectura: `docs/architecture/08-modulos-batteries-included.md`.
> README del crate: `crates/ag-cache/README.md`.
> Criticidad: Estandar.
> Fase de implementacion: Fase 4.

## Dominio

Cache de dos niveles: moka en memoria y Redis con fred. Invalidacion por evento.

## Dependencias internas permitidas

Depende de ag-core.

## Reglas aplicables

Vease las reglas 14 y 15 de `CLAUDE.md` para la separacion entre
nucleo, estandar y opcional, y para la politica de dependencias entre
crates Anti-Gravital.

## Estado

Fase 0: el crate `crates/ag-cache/` esta declarado en el workspace con
`src/lib.rs` vacio y `README.md` propio. No contiene codigo
funcional. La implementacion comienza en la fase indicada arriba.
