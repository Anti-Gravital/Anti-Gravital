# ag-ai

> Capitulo de arquitectura: `docs/architecture/11-ai-knowledge-graph.md`.
> README del crate: `crates/ag-ai/README.md`.
> Criticidad: Opcional.
> Fase de implementacion: Fase 6.

## Dominio

Knowledge graph desde el AST del DSL. Documentacion arquitectonica. Sugerencias asistidas con proveedores configurables.

## Dependencias internas permitidas

Depende de ag-dsl.

## Reglas aplicables

Vease las reglas 14 y 15 de `CLAUDE.md` para la separacion entre
nucleo, estandar y opcional, y para la politica de dependencias entre
crates Anti-Gravital.

## Estado

Fase 0: el crate `crates/ag-ai/` esta declarado en el workspace con
`src/lib.rs` vacio y `README.md` propio. No contiene codigo
funcional. La implementacion comienza en la fase indicada arriba.
