# ag-realtime

> Capitulo de arquitectura: `docs/architecture/08-modulos-batteries-included.md`.
> README del crate: `crates/ag-realtime/README.md`.
> Criticidad: Estandar.
> Fase de implementacion: Fase 4.

## Dominio

WebSocket binario, SSE fallback, NATS pub/sub. Presence, replay, broadcasting selectivo.

## Dependencias internas permitidas

Depende de ag-core. Puede depender de ag-auth para suscripciones autenticadas.

## Reglas aplicables

Vease las reglas 14 y 15 de `CLAUDE.md` para la separacion entre
nucleo, estandar y opcional, y para la politica de dependencias entre
crates Anti-Gravital.

## Estado

Fase 0: el crate `crates/ag-realtime/` esta declarado en el workspace con
`src/lib.rs` vacio y `README.md` propio. No contiene codigo
funcional. La implementacion comienza en la fase indicada arriba.
