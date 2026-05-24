# ag-auth

> Capitulo de arquitectura: `docs/architecture/08-modulos-batteries-included.md`.
> README del crate: `crates/ag-auth/README.md`.
> ADR de decision de libreria WebAuthn: `docs/adr/0006-ag-auth-webauthn-sin-webauthn-rs.md`.
> Criticidad: Estandar.
> Fase de implementacion: Fase 4. Estado: implementado.

## Dominio

Autenticacion y autorizacion: WebAuthn/FIDO2, JWT Ed25519, OAuth2 PKCE, API keys BLAKE3,
refresh tokens con blacklist en memoria.

## Stack implementado

| Componente | Libreria | Version |
|---|---|---|
| JWT firma/verificacion | `jsonwebtoken` | 10.x |
| Ed25519 par de claves | `ed25519-dalek` | 2.x |
| WebAuthn CBOR | `ciborium` | 0.2 |
| WebAuthn COSE ES256 | `p256` | 0.13 |
| WebAuthn COSE EdDSA | `ed25519-dalek` | 2.x |
| OAuth2 PKCE | `oauth2` | 5.x |
| HTTP OAuth | `reqwest` | 0.12 |
| API keys hash | `blake3` | 1.x |
| API keys encoding | `base64ct` | 1.x |

**Nota:** El master `ANTI-GRAVITAL-Arquitectura-Tecnica.md` especificaba `webauthn-rs`
como libreria WebAuthn. Esta libreria usa licencia MPL-2.0 incompatible con Apache-2.0.
La decision de usar `ciborium` + `p256` + `ed25519-dalek` directamente esta documentada
en `docs/adr/0006-ag-auth-webauthn-sin-webauthn-rs.md`.

## Capacidades implementadas (Fase 4)

- `AgAuth::new(config: AuthConfig, http_client: reqwest::Client)` — punto de entrada.
- `JwtSigner::sign(claims: &Claims) -> Result<String>` — JWT Ed25519.
- `JwtVerifier::verify(token: &str) -> Result<Claims>` — verificacion Ed25519.
- `AgAuth::create_api_key(prefix: &str) -> (String, String)` — (clave_raw, hash_blake3).
- `AgAuth::verify_api_key(raw: &str, hash: &str) -> bool`.
- `WebAuthnRp::new(rp_id, origin)` — registro y autenticacion FIDO2.
- `OAuthClient::authorization_url(provider)` — URL con PKCE.
- `OAuthClient::exchange_code(provider, code, verifier)` — intercambio de codigo.
- `RefreshBlacklist` — revocacion de refresh tokens en `RwLock<HashSet<String>>`.

## Dependencias internas permitidas

Depende de `ag-core`. Puede depender de `ag-data` para persistencia de sesiones.

## Tests

32 tests pasan. Cobertura >= 80%. Race condition en tests de env vars resuelta
con `static ENV_LOCK: Mutex<()>` en `config.rs`.

## Pendiente (criterios externos)

- Publicacion en crates.io con version 0.1.0.
- Documentacion de API expandida (guia de integracion con ag-core Shield).
- RBAC declarativo desde DSL (planificado para Fase 5).
