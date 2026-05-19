# ag-cli

> Capitulo de arquitectura: `docs/architecture/05-ecosistema-modulos.md`.
> README del crate: `crates/ag-cli/README.md`.
> Criticidad: Nucleo.
> Fase de implementacion: Fase 2 a Fase 9.

## Dominio

Binario `ag`: orquesta creacion de proyectos, generacion DSL, hot reload, build, deploy, migracion y administracion de plugins.

## Dependencias internas permitidas

Depende de todos los crates a traves de Cargo features.

## Reglas aplicables

Vease las reglas 14 y 15 de `CLAUDE.md` para la separacion entre
nucleo, estandar y opcional, y para la politica de dependencias entre
crates Anti-Gravital.

## Estado

Fase 0: el crate `crates/ag-cli/` esta declarado en el workspace con
`src/lib.rs` vacio y `README.md` propio. No contiene codigo
funcional. La implementacion comienza en la fase indicada arriba.
