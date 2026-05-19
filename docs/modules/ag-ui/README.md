# ag-ui

> Capitulo de arquitectura: `docs/architecture/05-ecosistema-modulos.md`.
> README del crate: `crates/ag-ui/README.md`.
> Criticidad: Opcional.
> Fase de implementacion: Fase 4 o posterior.

## Dominio

SSR con askama, hidratacion selectiva, integracion HTMX. No compite con frameworks SPA.

## Dependencias internas permitidas

Depende de ag-core.

## Reglas aplicables

Vease las reglas 14 y 15 de `CLAUDE.md` para la separacion entre
nucleo, estandar y opcional, y para la politica de dependencias entre
crates Anti-Gravital.

## Estado

Fase 0: el crate `crates/ag-ui/` esta declarado en el workspace con
`src/lib.rs` vacio y `README.md` propio. No contiene codigo
funcional. La implementacion comienza en la fase indicada arriba.
