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

Fase 1 y Fase 2 completadas. El crate contiene ~2 600 lineas de codigo
funcional organizado en los modulos `shield` y `core`.

Modulo `shield` (Fase 1): HTTP/1.1 y HTTP/2 via Axum + Tokio, TLS 1.3
con rustls, autenticacion JWT Ed25519 (`shield::auth`), rate limiting
token-bucket por IP (`shield::rate_limit`), CORS (`shield::cors`), CSRF
double-submit cookie (`shield::csrf`), validacion de payload
(`shield::validation`) y logging estructurado (`shield::logging`).
Configuracion desde TOML. 84 tests (unit + E2E + doctest).

Modulo `core` (Fase 2): reexporta `State<T>`, `Path<T>`, `Query<T>`,
`ValidatedBody<T>`, `Claims<T>` y el modulo `response` con `Json`,
`PlainText` y `BodyStream`. Sistema de errores `AgError` con conversion
automatica a respuesta HTTP. `Shield::apply(router)` integra ambas capas.

Rendimiento medido (Ryzen 5 2500U, oha 1.14.0): stack HTTP sin DB 88 930
req/s. Ver `docs/benchmarks/` para metodologia completa.
