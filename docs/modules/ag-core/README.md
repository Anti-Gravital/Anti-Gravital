# ag-core

> Capitulo de arquitectura: `docs/architecture/06-nucleo-shield-y-core.md`.
> README del crate: `crates/ag-core/README.md`.
> Criticidad: Nucleo.
> Fase de implementacion: Fase 1 (Shield) y Fase 2 (Core).

## Dominio

Runtime HTTP de alto rendimiento. Shield (Tower middleware) mas Core (Axum router) en un mismo proceso. Extractores tipados, sistema de errores tipado, runtime Tokio multi-thread.

## Dependencias internas permitidas

No depende de ningun crate Anti-Gravital. Es la base sobre la que todo lo demas se construye.

## Reglas aplicables

Vease las reglas 14 y 15 de `CLAUDE.md` para la separacion entre
nucleo, estandar y opcional, y para la politica de dependencias entre
crates Anti-Gravital.

## Estado

Fase 0: el crate `crates/ag-core/` esta declarado en el workspace con
`src/lib.rs` vacio y `README.md` propio. No contiene codigo
funcional. La implementacion comienza en la fase indicada arriba.
