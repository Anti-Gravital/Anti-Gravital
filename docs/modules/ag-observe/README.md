# ag-observe

> Capitulo de arquitectura: `docs/architecture/14-observabilidad-ag-observe.md`.
> README del crate: `crates/ag-observe/README.md`.
> Criticidad: Estandar.
> Fase de implementacion: Fase 4.

## Dominio

Tracing estructurado, OpenTelemetry, Prometheus, tokio-console, dashboards Grafana.

## Dependencias internas permitidas

Depende de ag-core.

## Reglas aplicables

Vease las reglas 14 y 15 de `CLAUDE.md` para la separacion entre
nucleo, estandar y opcional, y para la politica de dependencias entre
crates Anti-Gravital.

## Estado

Fase 0: el crate `crates/ag-observe/` esta declarado en el workspace con
`src/lib.rs` vacio y `README.md` propio. No contiene codigo
funcional. La implementacion comienza en la fase indicada arriba.
