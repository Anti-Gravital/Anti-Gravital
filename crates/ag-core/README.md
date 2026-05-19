# ag-core

> Estado: Fase 0 - Vacio. La implementacion comienza en Fase 1 (Shield MVP) y Fase 2 (Core MVP).
> Criticidad: Nucleo.
> Capitulo de arquitectura: docs/architecture/06-nucleo-shield-y-core.md

## Dominio

Runtime HTTP de alto rendimiento. Combina la capa Shield (Tower middleware con TLS 1.3, autenticacion JWT Ed25519, rate limiting, validacion, CORS, CSRF, RBAC) y la capa Core (router Axum con extractores tipados, sistema de errores y estado compartido). Es la unica dependencia obligatoria del ecosistema.

## Referencias

- Documento maestro de arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`.
- Capitulo navegable: `docs/architecture/06-nucleo-shield-y-core.md`.
- Hoja de ruta del crate: `docs/modules/ag-core/README.md`.
- Constitucion tecnica: `CLAUDE.md`.

## Reglas aplicables

- Este crate cumple las reglas 14 y 15 de `CLAUDE.md` sobre crates y
  dependencias.
- Versionado semantico independiente del resto del workspace una vez
  publicado.
- Sin `unsafe` salvo justificacion via RFC bajo `docs/rfc/`.
