# ag-core

> Estado: Fase 1 en curso. Bootstrap del Shield publicado.
> Criticidad: Nucleo.
> Capitulo de arquitectura: docs/architecture/06-nucleo-shield-y-core.md
> RFC vigente: docs/rfc/RFC-0002-diseno-shield-mvp.md

## Dominio

Runtime HTTP de alto rendimiento. Combina la capa Shield (pipeline
Tower de middleware) con la capa Core (router Axum con extractores
tipados, sistema de errores y estado compartido). Es la unica
dependencia obligatoria del ecosistema Anti-Gravital.

## Estado del bootstrap

Lo que ya esta:

- Crate compilando con dependencias declaradas (axum, tokio, tower,
  tower-http, tracing, serde, thiserror).
- Modulos `error`, `config`, `runtime`, `shield`, `core` con esqueleto
  estable.
- Capa Shield con logging estructurado (primera capa de la pipeline).
- Soporte HTTP/1.1 y HTTP/2 via Axum + Tokio (sin TLS).
- Tipos `AgError` y `AgResult` con mapeo automatico a respuestas HTTP.
- `ShieldConfig` deserializable desde TOML.
- Tests unitarios por modulo y tests E2E con servidor real.

Lo que falta de Fase 1 (en PRs posteriores):

- Capa de validacion de payload.
- Capa CORS.
- Capa CSRF.
- Capa de rate limiting con governor.
- Capa de autenticacion JWT Ed25519.
- Capa TLS 1.3 con rustls.
- Configuracion TOML completa.
- Benchmark Hello World con criterion.
- Documentacion publicada en docs.rs.

Lo que llega en Fase 2:

- Router Core con extractores tipados.
- Sistema de respuestas JSON, plaintext, streams.
- Integracion con `ag-data`.

## Referencias

- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` seccion 6.
- `docs/architecture/06-nucleo-shield-y-core.md`.
- `docs/roadmap/fase-01-shield-mvp.md`.
- `docs/rfc/RFC-0002-diseno-shield-mvp.md`.

## Reglas aplicables

- Cumple reglas 14 y 15 de `CLAUDE.md` (crates y dependencias).
- `ag-core` no depende de ningun crate Anti-Gravital.
- Versionado semantico independiente del resto del workspace.
- Sin `unsafe`. Vease `unsafe_code = "deny"` en `Cargo.toml` raiz.
