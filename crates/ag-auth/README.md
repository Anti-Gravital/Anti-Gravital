# ag-auth

> Estado: Fase 0 - Vacio. La implementacion comienza en Fase 4.
> Criticidad: Estandar.
> Capitulo de arquitectura: docs/architecture/08-modulos-batteries-included.md

## Dominio

Autenticacion y autorizacion: WebAuthn/Passkeys con FIDO2, JWT firmado con Ed25519, OAuth2 (Google, GitHub), API keys, refresh tokens con rotacion, rate limiting por usuario, RBAC declarativo con politicas evaluadas en el Shield.

## Referencias

- Documento maestro de arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`.
- Capitulo navegable: `docs/architecture/08-modulos-batteries-included.md`.
- Hoja de ruta del crate: `docs/modules/ag-auth/README.md`.
- Constitucion tecnica: `CLAUDE.md`.

## Reglas aplicables

- Este crate cumple las reglas 14 y 15 de `CLAUDE.md` sobre crates y
  dependencias.
- Versionado semantico independiente del resto del workspace una vez
  publicado.
- Sin `unsafe` salvo justificacion via RFC bajo `docs/rfc/`.
