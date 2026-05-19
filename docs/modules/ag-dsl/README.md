# ag-dsl

> Capitulo de arquitectura: `docs/architecture/07-anti-dsl.md`.
> README del crate: `crates/ag-dsl/README.md`.
> Criticidad: Nucleo.
> Fase de implementacion: Fase 3 a Fase 10.

## Dominio

Lexer, parser, AST, analisis semantico y codegen del Anti-DSL. Implementacion incremental v0.1..v1.0.

## Dependencias internas permitidas

No depende de ningun crate Anti-Gravital.

## Reglas aplicables

Vease las reglas 14 y 15 de `CLAUDE.md` para la separacion entre
nucleo, estandar y opcional, y para la politica de dependencias entre
crates Anti-Gravital.

## Estado

Fase 0: el crate `crates/ag-dsl/` esta declarado en el workspace con
`src/lib.rs` vacio y `README.md` propio. No contiene codigo
funcional. La implementacion comienza en la fase indicada arriba.
