# ag-storage

> Capitulo de arquitectura: `docs/architecture/08-modulos-batteries-included.md`.
> README del crate: `crates/ag-storage/README.md`.
> Criticidad: Estandar.
> Fase de implementacion: Fase 4.

## Dominio

Adaptadores S3, MinIO, filesystem. URLs firmadas. Procesamiento basico de imagenes.

## Dependencias internas permitidas

Depende de ag-core. Puede depender de ag-auth para URLs firmadas autenticadas.

## Reglas aplicables

Vease las reglas 14 y 15 de `CLAUDE.md` para la separacion entre
nucleo, estandar y opcional, y para la politica de dependencias entre
crates Anti-Gravital.

## Estado

Fase 0: el crate `crates/ag-storage/` esta declarado en el workspace con
`src/lib.rs` vacio y `README.md` propio. No contiene codigo
funcional. La implementacion comienza en la fase indicada arriba.
