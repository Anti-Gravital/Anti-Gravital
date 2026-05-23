# Fase 4 Completion — Spec de Diseno

**Fecha:** 2026-05-23
**Rama objetivo:** `f4-auth-webauthn-oauth2`, `f4-realtime-nats-ext`, `f4-e2e-tests`
**Crates afectados:** `ag-auth`, `ag-realtime`, `tests/integration` (nuevo)
**Documentos relacionados:** `ANTI-GRAVITAL-Arquitectura-Tecnica.md` secciones 4-5, RFC-0003

---

## 1. Contexto

La Fase 4 tiene tres TECH-DEBTs documentados que bloquean el cierre de la rama:

1. `ag-auth` — WebAuthn y OAuth2 tienen configuracion pero sin handlers reales.
2. `ag-realtime` — el modo `External` (NATS real) usa igualmente el bus in-process.
3. Tests E2E cross-module — no existe ningun test que use los cinco crates juntos.

Este spec cubre los tres sistemas. Un RFC separado (RFC-0005) documenta la propuesta de L2 cache nativo con protocolo RESP como trabajo futuro.

---

## 2. ag-auth: WebAuthn + OAuth2

### 2.1 Dependencias nuevas

| Crate | Version | Licencia | Justificacion |
|---|---|---|---|
| `passkey-types` | `0.2` | Apache-2.0 | Tipos FIDO2/COSE/CBOR para WebAuthn RP |
| `oauth2` | `5` | MIT/Apache-2.0 | Flujo Authorization Code para Google y GitHub |
| `ring` | ya en workspace via deps transitivas | ISC | Verificacion de firmas ECDSA P-256 / Ed25519 |
| `base64ct` | ya en workspace | MIT/Apache-2.0 | Encode/decode de challenges y credential IDs |

`webauthn-rs` queda excluido por licencia MPL-2.0 (no en `deny.toml` allowlist).

### 2.2 WebAuthn — nuevos archivos

**`crates/ag-auth/src/webauthn.rs`**

Implementa el rol de Relying Party (RP) FIDO2. Dos ceremonias:

**Registro:**
1. `start_registration(user_handle, display_name)` — genera challenge aleatorio 32 bytes, devuelve `RegistrationChallenge` con `PublicKeyCredentialCreationOptions` JSON. Guarda challenge en `HashMap<String, PendingCeremony>` con timestamp.
2. `finish_registration(response: RegistrationResponse)` — verifica:
   - `clientDataJSON.type == "webauthn.create"`
   - `clientDataJSON.origin` coincide con `self.origin`
   - `clientDataJSON.challenge` coincide con el challenge pendiente
   - `rpIdHash` del authenticatorData coincide con SHA-256 del `rp_id`
   - Flag UP (user present) activo
   - Parsea `credentialPublicKey` COSE con `passkey-types`
   - Devuelve `StoredCredential`

**Autenticacion:**
1. `start_authentication(creds)` — genera challenge, devuelve `AuthenticationChallenge` con los credential IDs permitidos.
2. `finish_authentication(response, creds)` — verifica:
   - `clientDataJSON.type == "webauthn.get"`
   - origin y challenge
   - rpIdHash
   - Firma sobre `authData || SHA-256(clientDataJSON)` con la clave COSE almacenada usando `ring::signature`
   - Incrementa `sign_count` en el credential
   - Devuelve `user_handle`

```rust
pub struct WebAuthnRp {
    rp_id: String,
    origin: String,
    pending: HashMap<String, PendingCeremony>,
}

pub struct StoredCredential {
    pub credential_id: Vec<u8>,
    pub cose_public_key: Vec<u8>,
    pub sign_count: u32,
    pub user_handle: String,
}

pub struct RegistrationChallenge {
    pub challenge_b64: String,
    pub options_json: serde_json::Value,
}

pub struct AuthenticationChallenge {
    pub challenge_b64: String,
    pub options_json: serde_json::Value,
}

#[derive(Debug, thiserror::Error)]
pub enum WebAuthnError {
    #[error("challenge no encontrado o expirado")]
    ChallengeNotFound,
    #[error("origen invalido: esperado {expected}, recibido {received}")]
    InvalidOrigin { expected: String, received: String },
    #[error("rp_id hash no coincide")]
    InvalidRpId,
    #[error("firma invalida")]
    InvalidSignature,
    #[error("formato invalido: {0}")]
    Format(String),
}

impl WebAuthnRp {
    pub fn new(rp_id: String, origin: String) -> Self;
    pub fn start_registration(&mut self, user_handle: &str, display_name: &str) -> RegistrationChallenge;
    pub fn finish_registration(&mut self, response: RegistrationResponse) -> Result<StoredCredential, WebAuthnError>;
    pub fn start_authentication(&mut self, creds: &[StoredCredential]) -> AuthenticationChallenge;
    pub fn finish_authentication(&mut self, response: AuthenticationResponse, creds: &mut Vec<StoredCredential>) -> Result<String, WebAuthnError>;
    pub fn purge_expired_challenges(&mut self, max_age_secs: u64);
}
```

TTL de challenges: 5 minutos por defecto. `purge_expired_challenges` se llama manualmente (el consumidor decide cuando limpiar).

**Tipos de respuesta del cliente:**

```rust
pub struct RegistrationResponse {
    pub id: String,
    pub raw_id: Vec<u8>,
    pub client_data_json: Vec<u8>,
    pub attestation_object: Vec<u8>,
}

pub struct AuthenticationResponse {
    pub id: String,
    pub raw_id: Vec<u8>,
    pub client_data_json: Vec<u8>,
    pub authenticator_data: Vec<u8>,
    pub signature: Vec<u8>,
    pub user_handle: Option<Vec<u8>>,
}
```

**Attestation:** Solo se verifican attestations de tipo `none` (attestation anonima — la mas comun en implementaciones web). Attestations `packed` / `tpm` / `android-key` quedan fuera de scope de esta iteracion.

**Tests minimos:**

- `registration_and_authentication_roundtrip` — usa `passkey-authenticator` como cliente de prueba para simular el ciclo completo sin navegador real.
- `invalid_origin_rejected`
- `expired_challenge_rejected`
- `invalid_signature_rejected`

### 2.3 OAuth2 — nuevos archivos

**`crates/ag-auth/src/oauth.rs`**

Flujo Authorization Code estandar. El crate `oauth2 v5` maneja la generacion de URLs, el intercambio de codigo y el PKCE. El consumidor es responsable de:
- Redirigir al usuario a la URL generada
- Recibir el callback con `code` y `state`
- Llamar a `exchange_code` con esos valores

```rust
#[derive(Debug, Clone, Copy)]
pub enum OAuthProvider {
    Google,
    GitHub,
}

pub struct OAuthClient {
    google: Option<oauth2::basic::BasicClient>,
    github: Option<oauth2::basic::BasicClient>,
    http: reqwest::Client,
}

pub struct OAuthUser {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub provider: OAuthProvider,
}

#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    #[error("proveedor {0:?} no configurado")]
    ProviderNotConfigured(OAuthProvider),
    #[error("error de red: {0}")]
    Http(String),
    #[error("respuesta invalida del proveedor: {0}")]
    InvalidResponse(String),
}

impl OAuthClient {
    pub fn from_config(config: &AuthConfig, http: reqwest::Client) -> Self;
    pub fn authorization_url(&self, provider: OAuthProvider, redirect_uri: &str) -> Result<(url::Url, oauth2::CsrfToken, oauth2::PkceCodeVerifier), OAuthError>;
    pub async fn exchange_code(&self, provider: OAuthProvider, code: &str, verifier: oauth2::PkceCodeVerifier, redirect_uri: &str) -> Result<OAuthUser, OAuthError>;
}
```

**User info endpoints:**

- Google: `https://www.googleapis.com/oauth2/v2/userinfo` con `Authorization: Bearer <token>`
- GitHub: `https://api.github.com/user` con `Authorization: Bearer <token>` y `User-Agent: anti-gravital`

**PKCE:** Obligatorio. Mitiga ataques de intercepcion de codigo. `oauth2 v5` lo genera automaticamente.

**Tests minimos:**

- `authorization_url_contains_provider_domain` — verifica que la URL generada apunta a Google/GitHub
- `unconfigured_provider_returns_error`
- `exchange_code_without_network` — mock de respuesta HTTP con `wiremock` o patron similar

### 2.4 Cambios en AgAuth

```rust
pub struct AgAuth {
    pub jwt: JwtSigner,
    pub webauthn: Option<WebAuthnRp>,  // None si rp_id esta vacio
    pub oauth: Option<OAuthClient>,    // None si ningun proveedor configurado
}
```

`AgAuth::new(config, http_client)` construye los tres subsistemas segun la configuracion.

---

## 3. ag-realtime: NATS externo + TLS + JetStream

### 3.1 Dependencias nuevas

| Crate | Version | Justificacion |
|---|---|---|
| `async-nats` | `0.48` (ya en workspace) | Cliente oficial NATS con TLS y JetStream |

No se añaden dependencias nuevas — `async-nats` ya esta declarado en el workspace.

### 3.2 Cambios de arquitectura

**`crates/ag-realtime/src/external.rs`** (nuevo)

Encapsula toda la logica del cliente NATS externo. Aislamiento: `lib.rs` no importa `async-nats` directamente.

```rust
pub struct NatsExternalClient {
    client: async_nats::Client,
    js: async_nats::jetstream::Context,
    stream_name: String,
    jetstream_enabled: bool,
}

impl NatsExternalClient {
    pub async fn connect(config: &RealtimeConfig) -> Result<Self, RealtimeError>;
    pub async fn publish(&self, subject: &str, payload: Vec<u8>) -> Result<(), RealtimeError>;
    pub async fn subscribe(&self, subject: &str) -> Result<BoxStream<'static, Event>, RealtimeError>;
}
```

**Enum interno en `lib.rs`:**

```rust
enum RealtimeBus {
    InProcess(Arc<EventBus>),
    External(Arc<NatsExternalClient>),
}
```

**`AgRealtime::new()` ahora es async:**

```rust
pub async fn new(config: RealtimeConfig) -> Result<Self, RealtimeError>
```

En `InProcess` no hay `await` real — la funcion resuelve instantaneamente.

### 3.3 Configuracion TLS extendida

Campos nuevos en `RealtimeConfig`:

```rust
pub struct RealtimeConfig {
    // Existentes
    pub nats_mode: NatsMode,
    pub nats_url: String,
    pub broadcast_capacity: usize,
    // TLS
    pub nats_tls: bool,
    pub nats_tls_ca_path: Option<String>,
    pub nats_tls_cert_path: Option<String>,
    pub nats_tls_key_path: Option<String>,
    // JetStream
    pub jetstream_enabled: bool,
    pub jetstream_stream_name: String,
    pub jetstream_max_msgs: i64,
    pub jetstream_max_bytes: i64,
}
```

Variables de entorno nuevas:

| Variable | Default | Descripcion |
|---|---|---|
| `NATS_TLS` | `false` | Activa TLS con CA del sistema |
| `NATS_TLS_CA` | — | Ruta a CA personalizada |
| `NATS_TLS_CERT` | — | Ruta a cert de cliente (mTLS) |
| `NATS_TLS_KEY` | — | Ruta a clave de cliente (mTLS) |
| `NATS_JETSTREAM` | `false` | Activa publicacion/consumo via JetStream |
| `NATS_JS_STREAM` | `AG_EVENTS` | Nombre del stream JetStream |
| `NATS_JS_MAX_MSGS` | `1000000` | Limite de mensajes del stream |
| `NATS_JS_MAX_BYTES` | `1073741824` | Limite de bytes del stream (1 GiB) |

### 3.4 JetStream

El stream `AG_EVENTS` se crea si no existe, con politica `Limits`. Los subjects son wildcards `>` (acepta cualquier subject).

**Publicacion:** Si `jetstream_enabled`, se publica con `js.publish(subject, payload).await` y se espera el ACK. Si JetStream no esta habilitado, se publica con `client.publish(subject, payload).await` (best effort).

**Suscripcion:** Se crea un push consumer efimero (sin nombre, sin durabilidad) para tests y consumidores ad-hoc. Para consumidores duraderos en produccion, el consumidor del crate crea el consumer explicitamente.

### 3.5 API publica actualizada

```rust
impl AgRealtime {
    pub async fn new(config: RealtimeConfig) -> Result<Self, RealtimeError>;

    // Sync cuando InProcess, async internamente para NATS (pero expuesto como sync via spawn)
    pub fn broadcast(&self, subject: impl Into<String>, payload: Vec<u8>) -> Result<(), RealtimeError>;
    pub fn broadcast_json<T: Serialize>(&self, subject: impl Into<String>, value: &T) -> Result<(), RealtimeError>;

    // Nueva — siempre async
    pub async fn subscribe(&self, subject: &str) -> Result<BoxStream<'static, Event>, RealtimeError>;

    pub fn bus(&self) -> Option<Arc<EventBus>>;  // None en modo External
}
```

`broadcast` en modo External lanza un `tokio::spawn` para el publish async y no bloquea al llamador. Esta es la misma semantica que el bus InProcess (fire-and-forget con capacidad de buffer). Los errores de publish NATS se registran via `tracing::error!` y se descartan — el contrato es best-effort igual que InProcess. Quien necesite garantias de entrega debe usar `subscribe` + ACK de JetStream directamente.

### 3.6 Tests

- Existentes (InProcess): sin cambios.
- Nuevos con NATS real: solo se ejecutan si `NATS_URL` esta definida en el entorno. El skip se implementa con un check en runtime al inicio del test: `if std::env::var("NATS_URL").is_err() { return; }`. En CI sin NATS estos tests pasan silenciosamente (exit 0).
- Tests TLS: usan certificados auto-firmados generados con `rcgen` (ya en workspace).
- Tests de `exchange_code` sin red: se marcan `#[ignore]` en CI. Se verifican con credenciales reales en entorno de desarrollo o con un listener TCP minimo que simula las respuestas del proveedor.

---

## 4. tests/integration — Tests E2E cross-module

### 4.1 Nuevo workspace member

```
tests/integration/
  Cargo.toml
  tests/
    fase4_e2e.rs
```

`Cargo.toml`:

```toml
[package]
name = "ag-integration-tests"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
publish = false
description = "Tests de integracion cross-module de Fase 4"

[lints]
workspace = true

[dev-dependencies]
ag-auth    = { path = "../../crates/ag-auth",    version = "0.0.0" }
ag-cache   = { path = "../../crates/ag-cache",   version = "0.0.0" }
ag-observe = { path = "../../crates/ag-observe", version = "0.0.0" }
ag-realtime = { path = "../../crates/ag-realtime", version = "0.0.0" }
ag-storage = { path = "../../crates/ag-storage", version = "0.0.0" }
tokio      = { workspace = true }
serde_json = { workspace = true }
tempfile   = { workspace = true }
rcgen      = { workspace = true }
ed25519-dalek = { workspace = true }
rand_core  = { workspace = true }
```

### 4.2 Tests en `tests/fase4_e2e.rs`

**Tests unitarios por modulo (verificacion de integracion minima):**

```rust
// ag-auth
#[tokio::test]
async fn jwt_sign_verify_roundtrip() { ... }

#[tokio::test]
async fn api_key_create_and_verify() { ... }

// ag-cache
#[tokio::test]
async fn cache_l1_set_get_tag_invalidation() { ... }

// ag-realtime
#[tokio::test]
async fn realtime_inprocess_broadcast_subscribe() { ... }

// ag-storage
#[tokio::test]
async fn storage_put_get_delete_roundtrip() { ... }

// ag-observe
#[test]
fn observe_init_does_not_panic() { ... }
```

**Test cross-module principal:**

```rust
#[tokio::test]
async fn e2e_authenticated_cached_event() {
    // 1. ag-observe: init tracing en modo test (RUST_LOG=off para no contaminar salida)
    // 2. ag-auth: generar par de claves Ed25519 con rcgen, construir AgAuth
    // 3. ag-auth: firmar JWT con claims {sub: "user-42", role: "admin"}
    // 4. ag-auth: verificar JWT, extraer claims
    // 5. ag-cache: construir AgCache L1
    // 6. ag-cache: set("user:42:profile", json_payload, tags=["user:42"])
    // 7. ag-realtime: construir AgRealtime InProcess
    // 8. ag-realtime: subscribe("user.profile.updated")
    // 9. ag-realtime: broadcast_json("user.profile.updated", payload)
    // 10. ag-realtime: verificar que el suscriptor recibe el evento
    // 11. ag-cache: get("user:42:profile") -> hit (mismo proceso, L1 calido)
    // 12. ag-cache: invalidate_tag("user:42") -> elimina la entrada
    // 13. ag-cache: get("user:42:profile") -> None (invalido correctamente)
    // 14. ag-storage: tempdir, put("avatars/user-42.png", bytes)
    // 15. ag-storage: get("avatars/user-42.png") -> bytes identicos
    // assert todos los pasos exitosos
}
```

El test no requiere ningun servicio externo. Verifica que los contratos entre crates son coherentes.

### 4.3 Adicion al workspace

En `Cargo.toml` raiz, agregar `"tests/integration"` a `[workspace.members]`.

---

## 5. RFC-0005: ag-cache L2 nativo con protocolo RESP

**Ubicacion:** `docs/rfc/RFC-0005-ag-cache-native-l2.md`

Este RFC documenta la propuesta de reemplazar la dependencia de Redis con un servidor de cache nativo de Anti-Gravital que:

1. Implementa el protocolo RESP2 (REdis Serialization Protocol) para ser compatible con cualquier cliente Redis estandar (`redis-cli`, librerías Redis en cualquier lenguaje).
2. Vive en `ag-cache` como feature optional `native-server`.
3. Usa el L1 existente (moka) como almacenamiento subyacente — el servidor RESP es solo un frontend de protocolo sobre el mismo store en memoria.
4. Soporta los comandos minimos para el caso de uso de cache distribuida: `GET`, `SET`, `DEL`, `EXPIRE`, `TTL`, `EXISTS`, `MGET`, `MSET`, `KEYS` (con patron simple).

**Motivacion:** Eliminar la dependencia operacional de Redis como servicio externo gestionado por terceros, manteniendo la compatibilidad con herramientas existentes que hablan RESP.

**Fuera de scope del RFC:** persistencia, clustering, replicacion, comandos de listas/sets/hashes avanzados — Anti-Gravital es cache, no reemplazo de Redis para casos de uso generales.

**Estado:** Propuesto. Requiere aprobacion antes de implementacion.

---

## 6. Orden de implementacion

Las tres ramas son independientes y pueden desarrollarse en paralelo:

```
fase-4
  ├── f4-auth-webauthn-oauth2     (ag-auth: webauthn.rs, oauth.rs)
  ├── f4-realtime-nats-ext        (ag-realtime: external.rs, config extendida)
  └── f4-e2e-tests                (tests/integration/)
```

El RFC-0005 se commite en `fase-4` directamente (es solo documentacion).

Cada rama se mergea a `fase-4` con `--no-ff` al completarse.

---

## 7. Checklist de salida por rama

### f4-auth-webauthn-oauth2
- [ ] `passkey-types` y `oauth2 v5` en workspace Cargo.toml
- [ ] `webauthn.rs`: WebAuthnRp, StoredCredential, ceremonias completas
- [ ] `oauth.rs`: OAuthClient, Google + GitHub, PKCE
- [ ] `AgAuth::new()` acepta `http_client: reqwest::Client`
- [ ] Tests: roundtrip registro+auth, OAuth URL, errores
- [ ] Tests existentes de `config.rs` actualizados para la nueva firma de `AgAuth::new(config, http_client)`
- [ ] `cargo fmt`, `cargo clippy -D warnings`, `cargo deny check`

### f4-realtime-nats-ext
- [ ] `external.rs`: NatsExternalClient, connect, publish, subscribe
- [ ] `config.rs`: campos TLS y JetStream con from_env
- [ ] `AgRealtime::new()` es async, devuelve Result
- [ ] Tests InProcess existentes siguen pasando
- [ ] Tests External con NATS real (skip si NATS_URL no definida)
- [ ] `cargo fmt`, `cargo clippy -D warnings`, `cargo deny check`

### f4-e2e-tests
- [ ] `tests/integration/Cargo.toml` en workspace
- [ ] 6 tests unitarios por modulo
- [ ] `e2e_authenticated_cached_event` cross-module completo
- [ ] `cargo test -p ag-integration-tests` verde sin servicios externos
- [ ] `cargo deny check` verde

### RFC-0005
- [ ] `docs/rfc/RFC-0005-ag-cache-native-l2.md` escrito y commiteado
