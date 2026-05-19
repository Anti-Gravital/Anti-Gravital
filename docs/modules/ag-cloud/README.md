# ag-cloud

> Capitulo de arquitectura: `docs/architecture/10-despliegue-ag-cloud.md`.
> README del crate: `crates/ag-cloud/README.md`.
> Criticidad: Opcional.
> Fase de implementacion: Fase 5.

## Dominio

Despliegue simplificado: docker-compose, Fly.io, Railway, Kubernetes. Genera Dockerfile multi-stage.

## Dependencias internas permitidas

Depende de ag-cli.

## Reglas aplicables

Vease las reglas 14 y 15 de `CLAUDE.md` para la separacion entre
nucleo, estandar y opcional, y para la politica de dependencias entre
crates Anti-Gravital.

## Estado

Fase 0: el crate `crates/ag-cloud/` esta declarado en el workspace con
`src/lib.rs` vacio y `README.md` propio. No contiene codigo
funcional. La implementacion comienza en la fase indicada arriba.
