# ag-mobile

> Estado: Fase 0 - Vacio. La implementacion comienza en Fase 8.
> Criticidad: Opcional.
> Capitulo de arquitectura: docs/architecture/13-mobile-ag-mobile.md

## Dominio

Puente de aplicaciones nativas. Genera SDK Dart completo: tipos con freezed, cliente HTTP con dio mas interceptores, cliente WebSocket, cliente SSE, mocks para tests. Widgets de autenticacion con WebAuthn nativo (Android Credential Manager, iOS Passkeys) y OAuth2. Publicado como paquete `anti_gravital` en pub.dev.

## Referencias

- Documento maestro de arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`.
- Capitulo navegable: `docs/architecture/13-mobile-ag-mobile.md`.
- Hoja de ruta del crate: `docs/modules/ag-mobile/README.md`.
- Constitucion tecnica: `CLAUDE.md`.

## Reglas aplicables

- Este crate cumple las reglas 14 y 15 de `CLAUDE.md` sobre crates y
  dependencias.
- Versionado semantico independiente del resto del workspace una vez
  publicado.
- Sin `unsafe` salvo justificacion via RFC bajo `docs/rfc/`.
