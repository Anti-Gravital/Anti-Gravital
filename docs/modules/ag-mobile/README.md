# ag-mobile

> Capitulo de arquitectura: `docs/architecture/13-mobile-ag-mobile.md`.
> README del crate: `crates/ag-mobile/README.md`.
> Criticidad: Opcional.
> Fase de implementacion: Fase 8.

## Dominio

Generador Dart para Flutter: tipos freezed, cliente dio, WebSocket, SSE, mocks, widgets de auth nativa.

## Dependencias internas permitidas

Depende de ag-dsl. Puede depender de ag-ai para generacion asistida.

## Reglas aplicables

Vease las reglas 14 y 15 de `CLAUDE.md` para la separacion entre
nucleo, estandar y opcional, y para la politica de dependencias entre
crates Anti-Gravital.

## Estado

Fase 0: el crate `crates/ag-mobile/` esta declarado en el workspace con
`src/lib.rs` vacio y `README.md` propio. No contiene codigo
funcional. La implementacion comienza en la fase indicada arriba.
