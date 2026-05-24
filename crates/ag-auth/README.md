# ag-auth

Autenticacion y autorizacion para Anti-Gravital: JWT Ed25519, API keys BLAKE3,
WebAuthn/Passkeys (CBOR+COSE), OAuth2 (Google, GitHub, PKCE) y refresh tokens
con blacklist en memoria.

> Estado: Fase 4 -- implementado.

## Uso minimo

```rust
use ag_auth::{AgAuth, AuthConfig, JwtClaims};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AuthConfig::from_env()?;
    let auth = AgAuth::new(config, Client::new());

    // JWT Ed25519
    let claims = JwtClaims {
        sub: "user-123".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as u64,
        ..Default::default()
    };
    let token = auth.jwt.sign(&claims)?;
    let verified = auth.jwt.verify::<JwtClaims>(&token)?;

    // API key
    let (key_id, api_key) = auth.create_api_key("sk")?;
    let is_valid = auth.verify_api_key(&key_id, &api_key)?;

    Ok(())
}
```

## Capacidades

### JWT Ed25519

Firma y verificacion con clave privada/publica Ed25519 en formato PEM.
Variables de entorno: `JWT_PRIVATE_KEY`, `JWT_PUBLIC_KEY`.

### API keys

Generacion con prefijo (`sk-...`, `pk-...`) y verificacion via hash BLAKE3.
No se almacena la clave en claro; solo el hash.

### WebAuthn/Passkeys

`WebAuthnRp` implementa las ceremonias de registro y autenticacion FIDO2:

- Codificacion CBOR con `ciborium`.
- Verificacion COSE (ES256 via `p256`, EdDSA via `ed25519-dalek`).
- Formato de desafio base64url.

### OAuth2

`OAuthClient` para Google y GitHub con flujo PKCE:

- `authorization_url(provider)` genera la URL de redireccion.
- `exchange_code(provider, code, verifier)` intercambia el codigo por tokens.
- Provider deshabilitado si las variables de entorno no estan definidas.

### Refresh tokens

`RefreshBlacklist` en memoria (`RwLock<HashSet<String>>`): revocacion
instantanea sin base de datos. Apropiado para instancia unica; para
clusters usar almacenamiento compartido.

## Variables de entorno

| Variable | Obligatorio | Descripcion |
|---|---|---|
| `JWT_PRIVATE_KEY` | si | Clave privada Ed25519 PEM |
| `JWT_PUBLIC_KEY` | si | Clave publica Ed25519 PEM |
| `WEBAUTHN_RP_ID` | no | RP ID WebAuthn (default: `localhost`) |
| `WEBAUTHN_ORIGIN` | no | Origen WebAuthn (default: `http://localhost:8080`) |
| `GOOGLE_CLIENT_ID` | no | OAuth2 Google -- deshabilita si ausente |
| `GOOGLE_CLIENT_SECRET` | no | OAuth2 Google secret |
| `GITHUB_CLIENT_ID` | no | OAuth2 GitHub -- deshabilita si ausente |
| `GITHUB_CLIENT_SECRET` | no | OAuth2 GitHub secret |

## Referencias

- Spec de diseno: `docs/superpowers/specs/2026-05-23-fase4-completion-design.md`
- Arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` seccion 8.1.
- Constitucion tecnica: `CLAUDE.md`.
