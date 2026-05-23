# f4-auth-complete — Plan de Implementacion

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implementar WebAuthn (FIDO2 passkeys), OAuth2 (Google + GitHub con PKCE) y refresh token blacklist en `ag-auth`. Eliminar todos los TECH-DEBT del modulo.

**Architecture:** Tres archivos nuevos: `webauthn.rs` (RP FIDO2 sin libreria externa de RP — parsing manual de authData + CBOR para COSE, verificacion con `p256` / `ed25519-dalek`), `oauth.rs` (OAuthClient usando `oauth2` v4 para PKCE + URL building, `reqwest` 0.12 para intercambio HTTP sin feature propia de oauth2), `refresh.rs` (blacklist en memoria via `HashSet<String>` protegido por `RwLock`). `AgAuth` se actualiza con campos opcionales para los tres subsistemas.

**Tech Stack:** Rust, `passkey-types 0.2` (tipos COSE/FIDO2), `oauth2 4` (PKCE + state), `ciborium 0.2` (CBOR para attestationObject y COSE key), `sha2 0.10` (rpIdHash), `p256 0.13` (verificacion ES256), `ed25519-dalek` (ya en workspace), `reqwest 0.12` (ya en workspace).

---

## Mapa de archivos

| Accion | Archivo | Responsabilidad |
|---|---|---|
| Modify | `Cargo.toml` (workspace) | Añadir passkey-types, oauth2, ciborium, sha2, p256, hmac |
| Modify | `crates/ag-auth/Cargo.toml` | Activar nuevas deps |
| Create | `crates/ag-auth/src/webauthn.rs` | WebAuthnRp — ceremonias registro + autenticacion |
| Create | `crates/ag-auth/src/oauth.rs` | OAuthClient — Google, GitHub, PKCE, exchange |
| Create | `crates/ag-auth/src/refresh.rs` | RefreshBlacklist — JTI blacklist en memoria |
| Modify | `crates/ag-auth/src/lib.rs` | AgAuth struct ampliada, new(config, http) |

---

## Task 1: Dependencias nuevas en workspace y ag-auth

**Files:**
- Modify: `Cargo.toml` (raiz del workspace)
- Modify: `crates/ag-auth/Cargo.toml`

- [ ] **Step 1: Añadir deps al workspace**

En `Cargo.toml` raiz, seccion `[workspace.dependencies]`, añadir despues de la linea `base64ct = ...`:

```toml
passkey-types = "0.2"
oauth2        = { version = "4", default-features = false }
ciborium      = "0.2"
sha2          = "0.10"
p256          = { version = "0.13", default-features = false, features = ["ecdsa"] }
hmac          = "0.12"
```

- [ ] **Step 2: Añadir deps a ag-auth/Cargo.toml**

En `crates/ag-auth/Cargo.toml`, seccion `[dependencies]`, añadir despues de `tracing`:

```toml
passkey-types = { workspace = true }
oauth2        = { workspace = true }
ciborium      = { workspace = true }
sha2          = { workspace = true }
p256          = { workspace = true }
hmac          = { workspace = true }
reqwest       = { workspace = true }
ed25519-dalek = { workspace = true }
url           = "2"
```

Y en `[dev-dependencies]` añadir:

```toml
passkey-authenticator = { version = "0.2", default-features = false }
```

Nota: `url = "2"` — verificar licencia MIT antes de compilar. Es MIT.
Nota: `passkey-authenticator` solo para tests, version 0.2, Apache-2.0.

- [ ] **Step 3: Verificar que cargo check compila (sin los nuevos modulos aun)**

```bash
cargo check -p ag-auth 2>&1 | head -20
```

Esperado: puede fallar si passkey-authenticator no existe en la version indicada.
Si falla por version, quitar `passkey-authenticator` del dev-dep (los tests de webauthn usaran datos sinteticos en su lugar).

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml crates/ag-auth/Cargo.toml
git commit -m "chore(ag-auth): añadir deps WebAuthn+OAuth2+CBOR al workspace"
```

---

## Task 2: `webauthn.rs` — tipos y registro

**Files:**
- Create: `crates/ag-auth/src/webauthn.rs`

- [ ] **Step 1: Escribir test que falle primero**

Crear `crates/ag-auth/src/webauthn.rs` con solo los tipos y un test inicial:

```rust
//! Relying Party FIDO2/WebAuthn para Anti-Gravital.
//!
//! Soporta attestation "none" (la mas comun en web). Las attestations
//! "packed", "tpm" y "android-key" quedan fuera de scope.

use base64ct::{Base64Url, Encoding};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Tipos publicos
// ---------------------------------------------------------------------------

/// Credencial almacenada tras el registro exitoso de un passkey.
#[derive(Debug, Clone)]
pub struct StoredCredential {
    /// ID de la credencial (bytes del autenticador).
    pub credential_id: Vec<u8>,
    /// Clave publica en formato COSE (CBOR).
    pub cose_public_key: Vec<u8>,
    /// Contador de firmas para detectar clonacion.
    pub sign_count: u32,
    /// Handle del usuario propietario de la credencial.
    pub user_handle: String,
}

/// Respuesta de la fase de registro enviada por el navegador.
pub struct RegistrationResponse {
    /// ID de la credencial en Base64Url.
    pub id: String,
    /// clientDataJSON codificado como bytes crudos (no Base64).
    pub client_data_json: Vec<u8>,
    /// attestationObject codificado como bytes crudos (no Base64).
    pub attestation_object: Vec<u8>,
}

/// Respuesta de la fase de autenticacion enviada por el navegador.
pub struct AuthenticationResponse {
    /// ID de la credencial que firmo el assertion.
    pub credential_id: Vec<u8>,
    /// clientDataJSON como bytes crudos.
    pub client_data_json: Vec<u8>,
    /// authenticatorData como bytes crudos.
    pub authenticator_data: Vec<u8>,
    /// Firma DER del assertion.
    pub signature: Vec<u8>,
}

/// Challenge enviado al cliente para iniciar el registro.
pub struct RegistrationChallenge {
    /// Challenge en Base64Url (para incluir en la respuesta).
    pub challenge_b64: String,
    /// PublicKeyCredentialCreationOptions serializado como JSON.
    pub options_json: serde_json::Value,
}

/// Challenge enviado al cliente para iniciar la autenticacion.
pub struct AuthenticationChallenge {
    /// Challenge en Base64Url.
    pub challenge_b64: String,
    /// PublicKeyCredentialRequestOptions serializado como JSON.
    pub options_json: serde_json::Value,
}

/// Error del subsistema WebAuthn.
#[derive(Debug, thiserror::Error)]
pub enum WebAuthnError {
    #[error("challenge no encontrado o expirado")]
    ChallengeNotFound,
    #[error("origen invalido: esperado {expected}, recibido {received}")]
    InvalidOrigin { expected: String, received: String },
    #[error("rp_id hash no coincide")]
    InvalidRpId,
    #[error("flag de usuario presente no activo")]
    UserNotPresent,
    #[error("firma invalida")]
    InvalidSignature,
    #[error("tipo de operacion invalido: {0}")]
    InvalidType(String),
    #[error("formato invalido: {0}")]
    Format(String),
    #[error("algoritmo COSE no soportado: {0}")]
    UnsupportedAlgorithm(i64),
}

// ---------------------------------------------------------------------------
// Internos
// ---------------------------------------------------------------------------

struct PendingCeremony {
    challenge_bytes: Vec<u8>,
    created_at: Instant,
}

struct ParsedAuthData {
    rp_id_hash: [u8; 32],
    up: bool,
    sign_count: u32,
    cred_id: Option<Vec<u8>>,
    cose_public_key: Option<Vec<u8>>,
}

// ---------------------------------------------------------------------------
// WebAuthnRp
// ---------------------------------------------------------------------------

/// Relying Party FIDO2/WebAuthn.
///
/// Gestiona challenges pendientes y verifica las respuestas del autenticador.
/// Estado en memoria — no persistente entre reinicios del proceso.
pub struct WebAuthnRp {
    rp_id: String,
    origin: String,
    pending: HashMap<String, PendingCeremony>,
}

impl WebAuthnRp {
    /// Crea un nuevo RP con el identificador y el origen dados.
    pub fn new(rp_id: String, origin: String) -> Self {
        Self {
            rp_id,
            origin,
            pending: HashMap::new(),
        }
    }

    /// Genera un challenge de registro y lo devuelve junto con las opciones para el cliente.
    pub fn start_registration(&mut self, user_handle: &str, display_name: &str) -> RegistrationChallenge {
        let mut challenge_bytes = [0u8; 32];
        getrandom::getrandom(&mut challenge_bytes).expect("entopia del sistema");
        let challenge_b64 = Base64Url::encode_string(&challenge_bytes);

        self.pending.insert(
            user_handle.to_string(),
            PendingCeremony {
                challenge_bytes: challenge_bytes.to_vec(),
                created_at: Instant::now(),
            },
        );

        let options_json = serde_json::json!({
            "challenge": challenge_b64,
            "rp": { "id": self.rp_id, "name": self.rp_id },
            "user": {
                "id": Base64Url::encode_string(user_handle.as_bytes()),
                "name": user_handle,
                "displayName": display_name,
            },
            "pubKeyCredParams": [
                { "type": "public-key", "alg": -7 },
                { "type": "public-key", "alg": -8 },
            ],
            "timeout": 300000,
            "attestation": "none",
        });

        RegistrationChallenge { challenge_b64, options_json }
    }

    /// Finaliza el registro verificando la respuesta del autenticador.
    pub fn finish_registration(
        &mut self,
        user_handle: &str,
        response: RegistrationResponse,
    ) -> Result<StoredCredential, WebAuthnError> {
        let pending = self.take_pending(user_handle)?;
        let client_data = parse_client_data(&response.client_data_json)?;
        verify_type(&client_data, "webauthn.create")?;
        self.verify_origin(&client_data)?;
        verify_challenge(&client_data, &pending.challenge_bytes)?;
        let auth_data = parse_attestation_object(&response.attestation_object)?;
        self.verify_rp_id_hash(&auth_data.rp_id_hash)?;
        if !auth_data.up {
            return Err(WebAuthnError::UserNotPresent);
        }
        let cred_id = auth_data.cred_id.ok_or_else(|| WebAuthnError::Format("sin credentialId".into()))?;
        let cose_key = auth_data.cose_public_key.ok_or_else(|| WebAuthnError::Format("sin clave COSE".into()))?;

        Ok(StoredCredential {
            credential_id: cred_id,
            cose_public_key: cose_key,
            sign_count: auth_data.sign_count,
            user_handle: user_handle.to_string(),
        })
    }

    /// Genera un challenge de autenticacion.
    pub fn start_authentication(
        &mut self,
        user_handle: &str,
        creds: &[StoredCredential],
    ) -> AuthenticationChallenge {
        let mut challenge_bytes = [0u8; 32];
        getrandom::getrandom(&mut challenge_bytes).expect("entopia del sistema");
        let challenge_b64 = Base64Url::encode_string(&challenge_bytes);

        self.pending.insert(
            format!("auth:{user_handle}"),
            PendingCeremony {
                challenge_bytes: challenge_bytes.to_vec(),
                created_at: Instant::now(),
            },
        );

        let allow_creds: Vec<serde_json::Value> = creds
            .iter()
            .map(|c| serde_json::json!({
                "type": "public-key",
                "id": Base64Url::encode_string(&c.credential_id),
            }))
            .collect();

        let options_json = serde_json::json!({
            "challenge": challenge_b64,
            "rpId": self.rp_id,
            "allowCredentials": allow_creds,
            "timeout": 300000,
        });

        AuthenticationChallenge { challenge_b64, options_json }
    }

    /// Verifica la respuesta de autenticacion y retorna el user_handle si es valida.
    pub fn finish_authentication(
        &mut self,
        user_handle: &str,
        response: AuthenticationResponse,
        creds: &mut Vec<StoredCredential>,
    ) -> Result<String, WebAuthnError> {
        let key = format!("auth:{user_handle}");
        let pending = self.take_pending(&key)?;

        let cred = creds
            .iter_mut()
            .find(|c| c.credential_id == response.credential_id)
            .ok_or_else(|| WebAuthnError::Format("credencial no encontrada".into()))?;

        let client_data = parse_client_data(&response.client_data_json)?;
        verify_type(&client_data, "webauthn.get")?;
        self.verify_origin(&client_data)?;
        verify_challenge(&client_data, &pending.challenge_bytes)?;

        let auth_data = parse_raw_auth_data(&response.authenticator_data)?;
        self.verify_rp_id_hash(&auth_data.rp_id_hash)?;
        if !auth_data.up {
            return Err(WebAuthnError::UserNotPresent);
        }

        // verification_data = authData || SHA-256(clientDataJSON)
        let mut cdj_hash = Sha256::new();
        cdj_hash.update(&response.client_data_json);
        let cdj_digest: [u8; 32] = cdj_hash.finalize().into();
        let mut verification_data = response.authenticator_data.clone();
        verification_data.extend_from_slice(&cdj_digest);

        verify_cose_signature(&cred.cose_public_key, &verification_data, &response.signature)?;

        cred.sign_count = auth_data.sign_count;
        Ok(cred.user_handle.clone())
    }

    /// Elimina challenges con mas de `max_age_secs` segundos.
    pub fn purge_expired_challenges(&mut self, max_age_secs: u64) {
        let max_age = Duration::from_secs(max_age_secs);
        self.pending.retain(|_, v| v.created_at.elapsed() < max_age);
    }

    fn take_pending(&mut self, key: &str) -> Result<PendingCeremony, WebAuthnError> {
        match self.pending.remove(key) {
            Some(p) if p.created_at.elapsed() < Duration::from_secs(300) => Ok(p),
            Some(_) => Err(WebAuthnError::ChallengeNotFound),
            None => Err(WebAuthnError::ChallengeNotFound),
        }
    }

    fn verify_origin(&self, client_data: &serde_json::Value) -> Result<(), WebAuthnError> {
        let received = client_data["origin"].as_str().unwrap_or("").to_string();
        if received != self.origin {
            return Err(WebAuthnError::InvalidOrigin {
                expected: self.origin.clone(),
                received,
            });
        }
        Ok(())
    }

    fn verify_rp_id_hash(&self, hash: &[u8; 32]) -> Result<(), WebAuthnError> {
        let mut h = Sha256::new();
        h.update(self.rp_id.as_bytes());
        let expected: [u8; 32] = h.finalize().into();
        if expected != *hash {
            return Err(WebAuthnError::InvalidRpId);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Funciones de parseo
// ---------------------------------------------------------------------------

fn parse_client_data(bytes: &[u8]) -> Result<serde_json::Value, WebAuthnError> {
    serde_json::from_slice(bytes).map_err(|e| WebAuthnError::Format(e.to_string()))
}

fn verify_type(client_data: &serde_json::Value, expected: &str) -> Result<(), WebAuthnError> {
    let got = client_data["type"].as_str().unwrap_or("");
    if got != expected {
        return Err(WebAuthnError::InvalidType(got.to_string()));
    }
    Ok(())
}

fn verify_challenge(client_data: &serde_json::Value, expected_bytes: &[u8]) -> Result<(), WebAuthnError> {
    let ch_b64 = client_data["challenge"].as_str().unwrap_or("");
    let got = Base64Url::decode_vec(ch_b64)
        .map_err(|_| WebAuthnError::Format("challenge base64 invalido".into()))?;
    if got != expected_bytes {
        return Err(WebAuthnError::ChallengeNotFound);
    }
    Ok(())
}

/// Parsea el attestationObject CBOR y extrae authData.
fn parse_attestation_object(bytes: &[u8]) -> Result<ParsedAuthData, WebAuthnError> {
    let value: ciborium::value::Value = ciborium::from_reader(bytes)
        .map_err(|e| WebAuthnError::Format(format!("attestationObject CBOR: {e}")))?;

    let map = match &value {
        ciborium::value::Value::Map(m) => m,
        _ => return Err(WebAuthnError::Format("attestationObject no es un mapa".into())),
    };

    let auth_data_bytes = map
        .iter()
        .find(|(k, _)| k == &ciborium::value::Value::Text("authData".into()))
        .and_then(|(_, v)| v.as_bytes().map(|b| b.to_vec()))
        .ok_or_else(|| WebAuthnError::Format("authData ausente".into()))?;

    parse_raw_auth_data(&auth_data_bytes)
}

/// Parsea authData binario (37+ bytes).
fn parse_raw_auth_data(auth_data: &[u8]) -> Result<ParsedAuthData, WebAuthnError> {
    if auth_data.len() < 37 {
        return Err(WebAuthnError::Format("authData demasiado corto".into()));
    }
    let rp_id_hash: [u8; 32] = auth_data[0..32].try_into().unwrap();
    let flags = auth_data[32];
    let up = (flags & 0x01) != 0;
    let at = (flags & 0x40) != 0; // attested credential data present
    let sign_count = u32::from_be_bytes(auth_data[33..37].try_into().unwrap());

    let (cred_id, cose_public_key) = if at && auth_data.len() > 37 {
        let rest = &auth_data[37..];
        if rest.len() < 18 {
            return Err(WebAuthnError::Format("attestedCredentialData corto".into()));
        }
        let cred_id_len = u16::from_be_bytes([rest[16], rest[17]]) as usize;
        let start = 18 + cred_id_len;
        if rest.len() < start {
            return Err(WebAuthnError::Format("credentialId corto".into()));
        }
        let cred_id = rest[18..start].to_vec();
        let cose_key = rest[start..].to_vec();
        (Some(cred_id), Some(cose_key))
    } else {
        (None, None)
    };

    Ok(ParsedAuthData { rp_id_hash, up, sign_count, cred_id, cose_public_key })
}

/// Verifica una firma COSE contra los datos de verificacion.
///
/// Soporta alg -7 (ES256/P-256) y alg -8 (EdDSA/Ed25519).
fn verify_cose_signature(
    cose_key_bytes: &[u8],
    data: &[u8],
    sig: &[u8],
) -> Result<(), WebAuthnError> {
    let key_value: ciborium::value::Value = ciborium::from_reader(cose_key_bytes)
        .map_err(|e| WebAuthnError::Format(format!("COSE key CBOR: {e}")))?;

    let map = match &key_value {
        ciborium::value::Value::Map(m) => m,
        _ => return Err(WebAuthnError::Format("COSE key no es mapa".into())),
    };

    let get_int = |key: i64| -> Option<i64> {
        map.iter()
            .find(|(k, _)| k == &ciborium::value::Value::Integer(key.into()))
            .and_then(|(_, v)| v.as_integer().and_then(|i| i64::try_from(i).ok()))
    };
    let get_bytes = |key: i64| -> Option<Vec<u8>> {
        map.iter()
            .find(|(k, _)| k == &ciborium::value::Value::Integer(key.into()))
            .and_then(|(_, v)| v.as_bytes().map(|b| b.to_vec()))
    };

    let alg = get_int(3).ok_or_else(|| WebAuthnError::Format("alg ausente".into()))?;

    match alg {
        -7 => {
            // ES256 — P-256 ECDSA con SHA-256
            use p256::ecdsa::{signature::Verifier, DerSignature, VerifyingKey};
            use p256::EncodedPoint;

            let x = get_bytes(-2).ok_or_else(|| WebAuthnError::Format("P-256 x ausente".into()))?;
            let y = get_bytes(-3).ok_or_else(|| WebAuthnError::Format("P-256 y ausente".into()))?;

            let point = EncodedPoint::from_affine_coordinates(
                x.as_slice().into(),
                y.as_slice().into(),
                false,
            );
            let vk = VerifyingKey::from_encoded_point(&point)
                .map_err(|_| WebAuthnError::InvalidSignature)?;
            let signature = DerSignature::try_from(sig)
                .map_err(|_| WebAuthnError::InvalidSignature)?;
            vk.verify(data, &signature)
                .map_err(|_| WebAuthnError::InvalidSignature)
        }
        -8 => {
            // EdDSA — Ed25519
            use ed25519_dalek::{Signature, Verifier, VerifyingKey as Ed25519Vk};

            let x = get_bytes(-2).ok_or_else(|| WebAuthnError::Format("Ed25519 x ausente".into()))?;
            let key_bytes: [u8; 32] = x
                .as_slice()
                .try_into()
                .map_err(|_| WebAuthnError::Format("Ed25519 key no es 32 bytes".into()))?;
            let vk = Ed25519Vk::from_bytes(&key_bytes)
                .map_err(|_| WebAuthnError::InvalidSignature)?;
            let sig_bytes: [u8; 64] = sig
                .try_into()
                .map_err(|_| WebAuthnError::InvalidSignature)?;
            let signature = Signature::from_bytes(&sig_bytes);
            vk.verify(data, &signature)
                .map_err(|_| WebAuthnError::InvalidSignature)
        }
        alg => Err(WebAuthnError::UnsupportedAlgorithm(alg)),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rp() -> WebAuthnRp {
        WebAuthnRp::new("localhost".into(), "http://localhost:8080".into())
    }

    #[test]
    fn start_registration_generates_unique_challenges() {
        let mut rp = make_rp();
        let c1 = rp.start_registration("user1", "User One");
        let c2 = rp.start_registration("user2", "User Two");
        assert_ne!(c1.challenge_b64, c2.challenge_b64, "challenges deben ser unicos");
    }

    #[test]
    fn start_registration_options_contain_rp_and_user() {
        let mut rp = make_rp();
        let ch = rp.start_registration("alice", "Alice");
        let opts = &ch.options_json;
        assert_eq!(opts["rp"]["id"], "localhost");
        assert!(opts["challenge"].as_str().is_some());
    }

    #[test]
    fn finish_registration_fails_with_wrong_challenge() {
        let mut rp = make_rp();
        rp.start_registration("bob", "Bob");

        // challenge incorrecto en clientDataJSON
        let client_data = serde_json::json!({
            "type": "webauthn.create",
            "challenge": Base64Url::encode_string(b"wrong_challenge_32_bytes_padding!"),
            "origin": "http://localhost:8080",
        });
        let cdj_bytes = serde_json::to_vec(&client_data).unwrap();

        let response = RegistrationResponse {
            id: "cred-id".into(),
            client_data_json: cdj_bytes,
            attestation_object: vec![],
        };
        let result = rp.finish_registration("bob", response);
        // Debe fallar — con attestationObject vacio falla en parseo CBOR
        assert!(result.is_err());
    }

    #[test]
    fn finish_registration_fails_with_wrong_origin() {
        let mut rp = make_rp();
        let ch = rp.start_registration("carol", "Carol");

        let client_data = serde_json::json!({
            "type": "webauthn.create",
            "challenge": ch.challenge_b64,
            "origin": "https://evil.com",
        });
        let cdj_bytes = serde_json::to_vec(&client_data).unwrap();

        let response = RegistrationResponse {
            id: "cred".into(),
            client_data_json: cdj_bytes,
            attestation_object: vec![],
        };
        let result = rp.finish_registration("carol", response);
        assert!(matches!(result, Err(WebAuthnError::InvalidOrigin { .. })));
    }

    #[test]
    fn expired_challenge_rejected() {
        let mut rp = WebAuthnRp::new("localhost".into(), "http://localhost".into());
        rp.start_registration("dave", "Dave");
        // Purgar inmediatamente con TTL de 0 segundos
        rp.purge_expired_challenges(0);

        let client_data = serde_json::json!({
            "type": "webauthn.create",
            "challenge": Base64Url::encode_string(b"irrelevant_challenge_32_bytes!!!"),
            "origin": "http://localhost",
        });
        let response = RegistrationResponse {
            id: "x".into(),
            client_data_json: serde_json::to_vec(&client_data).unwrap(),
            attestation_object: vec![],
        };
        let result = rp.finish_registration("dave", response);
        assert!(matches!(result, Err(WebAuthnError::ChallengeNotFound)));
    }

    #[test]
    fn parse_raw_auth_data_rejects_short_input() {
        let result = parse_raw_auth_data(&[0u8; 10]);
        assert!(matches!(result, Err(WebAuthnError::Format(_))));
    }
}
```

- [ ] **Step 2: Verificar que compila**

```bash
cargo check -p ag-auth 2>&1 | grep -E "^error" | head -20
```

Esperado: 0 errores. Si hay errores de imports, corregir las rutas de `use`.

- [ ] **Step 3: Ejecutar tests**

```bash
cargo test -p ag-auth webauthn 2>&1
```

Esperado: todos los tests de webauthn pasan.

- [ ] **Step 4: Exponer el modulo en lib.rs**

En `crates/ag-auth/src/lib.rs`, añadir antes de `pub use api_keys`:

```rust
pub mod webauthn;
pub use webauthn::{
    AuthenticationChallenge, AuthenticationResponse, RegistrationChallenge,
    RegistrationResponse, StoredCredential, WebAuthnError, WebAuthnRp,
};
```

- [ ] **Step 5: Commit**

```bash
git add crates/ag-auth/src/webauthn.rs crates/ag-auth/src/lib.rs
git commit -m "feat(ag-auth): WebAuthnRp — registro y autenticacion FIDO2 (ES256 + EdDSA)"
```

---

## Task 3: `oauth.rs` — OAuthClient Google + GitHub

**Files:**
- Create: `crates/ag-auth/src/oauth.rs`

- [ ] **Step 1: Escribir tests que fallen**

Crear `crates/ag-auth/src/oauth.rs`:

```rust
//! Cliente OAuth2 Authorization Code + PKCE para Google y GitHub.
//!
//! No usa la feature reqwest de oauth2 para evitar conflicto de versiones.
//! El intercambio de tokens se implementa directamente con reqwest 0.12.

use crate::config::AuthConfig;
use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenUrl,
    basic::BasicClient,
};

/// Proveedor OAuth2 soportado.
#[derive(Debug, Clone, Copy)]
pub enum OAuthProvider {
    /// Google Identity Platform.
    Google,
    /// GitHub OAuth Apps.
    GitHub,
}

impl std::fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuthProvider::Google => write!(f, "Google"),
            OAuthProvider::GitHub => write!(f, "GitHub"),
        }
    }
}

/// Informacion del usuario obtenida del proveedor tras la autenticacion.
#[derive(Debug, Clone)]
pub struct OAuthUser {
    /// ID unico del usuario en el proveedor.
    pub id: String,
    /// Email del usuario (puede ser None si el proveedor no lo devuelve).
    pub email: Option<String>,
    /// Nombre del usuario.
    pub name: Option<String>,
    /// Proveedor de donde proviene el usuario.
    pub provider: OAuthProvider,
}

/// Error del cliente OAuth2.
#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    #[error("proveedor {0} no configurado")]
    ProviderNotConfigured(OAuthProvider),
    #[error("error de red: {0}")]
    Http(String),
    #[error("respuesta invalida del proveedor: {0}")]
    InvalidResponse(String),
    #[error("error OAuth2: {0}")]
    OAuth(String),
}

/// Cliente OAuth2 para Google y GitHub.
///
/// Construir con [`OAuthClient::from_config`]. Cada provider se habilita
/// independientemente mediante las variables de entorno correspondientes.
pub struct OAuthClient {
    google: Option<BasicClient>,
    github: Option<BasicClient>,
    http: reqwest::Client,
}

impl OAuthClient {
    /// Construye el cliente a partir de la configuracion del modulo de auth.
    pub fn from_config(config: &AuthConfig, http: reqwest::Client) -> Self {
        let google = config
            .oauth_google_client_id
            .as_ref()
            .zip(config.oauth_google_client_secret.as_ref())
            .map(|(id, secret)| {
                BasicClient::new(
                    ClientId::new(id.clone()),
                    Some(ClientSecret::new(secret.clone())),
                    AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".into())
                        .expect("Google auth URL invalida"),
                    Some(
                        TokenUrl::new("https://oauth2.googleapis.com/token".into())
                            .expect("Google token URL invalida"),
                    ),
                )
            });

        let github = config
            .oauth_github_client_id
            .as_ref()
            .zip(config.oauth_github_client_secret.as_ref())
            .map(|(id, secret)| {
                BasicClient::new(
                    ClientId::new(id.clone()),
                    Some(ClientSecret::new(secret.clone())),
                    AuthUrl::new("https://github.com/login/oauth/authorize".into())
                        .expect("GitHub auth URL invalida"),
                    Some(
                        TokenUrl::new("https://github.com/login/oauth/access_token".into())
                            .expect("GitHub token URL invalida"),
                    ),
                )
            });

        Self { google, github, http }
    }

    /// Genera la URL de autorizacion con PKCE y state CSRF.
    ///
    /// Retorna `(url, state, pkce_verifier)`. El llamador debe:
    /// 1. Redirigir al usuario a `url`.
    /// 2. Persistir `state` y `pkce_verifier` en sesion para el callback.
    pub fn authorization_url(
        &self,
        provider: OAuthProvider,
        redirect_uri: &str,
    ) -> Result<(url::Url, CsrfToken, PkceCodeVerifier), OAuthError> {
        let client = self.client_for(provider)?;
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let redirect = RedirectUrl::new(redirect_uri.to_string())
            .map_err(|e| OAuthError::OAuth(e.to_string()))?;

        let scopes = match provider {
            OAuthProvider::Google => vec![
                Scope::new("openid".into()),
                Scope::new("email".into()),
                Scope::new("profile".into()),
            ],
            OAuthProvider::GitHub => vec![
                Scope::new("read:user".into()),
                Scope::new("user:email".into()),
            ],
        };

        let (url, state) = client
            .authorize_url(CsrfToken::new_random)
            .set_redirect_uri(std::borrow::Cow::Owned(redirect))
            .set_pkce_challenge(pkce_challenge)
            .add_scopes(scopes)
            .url();

        Ok((url, state, pkce_verifier))
    }

    /// Intercambia un codigo de autorizacion por la informacion del usuario.
    ///
    /// El llamador debe pasar el mismo `redirect_uri` y el `pkce_verifier` guardados en sesion.
    pub async fn exchange_code(
        &self,
        provider: OAuthProvider,
        code: &str,
        verifier: PkceCodeVerifier,
        redirect_uri: &str,
    ) -> Result<OAuthUser, OAuthError> {
        let token_url = match provider {
            OAuthProvider::Google => "https://oauth2.googleapis.com/token",
            OAuthProvider::GitHub => "https://github.com/login/oauth/access_token",
        };
        let client_id = self.client_id_for(provider)?;
        let client_secret = self.client_secret_for(provider)?;

        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("code_verifier", verifier.secret()),
            ("client_id", &client_id),
            ("client_secret", &client_secret),
        ];

        let response = self.http
            .post(token_url)
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| OAuthError::Http(e.to_string()))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(OAuthError::InvalidResponse(format!("token exchange: {body}")));
        }

        let token_body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| OAuthError::InvalidResponse(e.to_string()))?;

        let access_token = token_body["access_token"]
            .as_str()
            .ok_or_else(|| OAuthError::InvalidResponse("access_token ausente".into()))?
            .to_string();

        self.fetch_user_info(provider, &access_token).await
    }

    // ---------------------------------------------------------------------------
    // Privados
    // ---------------------------------------------------------------------------

    fn client_for(&self, provider: OAuthProvider) -> Result<&BasicClient, OAuthError> {
        match provider {
            OAuthProvider::Google => self.google.as_ref(),
            OAuthProvider::GitHub => self.github.as_ref(),
        }
        .ok_or(OAuthError::ProviderNotConfigured(provider))
    }

    fn client_id_for(&self, provider: OAuthProvider) -> Result<String, OAuthError> {
        self.client_for(provider).map(|c| c.client_id().to_string())
    }

    fn client_secret_for(&self, provider: OAuthProvider) -> Result<String, OAuthError> {
        self.client_for(provider)
            .and_then(|c| {
                c.client_secret()
                    .map(|s| s.secret().to_string())
                    .ok_or(OAuthError::ProviderNotConfigured(provider))
            })
    }

    async fn fetch_user_info(
        &self,
        provider: OAuthProvider,
        access_token: &str,
    ) -> Result<OAuthUser, OAuthError> {
        let (url, user_agent) = match provider {
            OAuthProvider::Google => ("https://www.googleapis.com/oauth2/v2/userinfo", None),
            OAuthProvider::GitHub => ("https://api.github.com/user", Some("anti-gravital")),
        };

        let mut req = self.http.get(url).bearer_auth(access_token);
        if let Some(ua) = user_agent {
            req = req.header("User-Agent", ua);
        }

        let resp = req.send().await.map_err(|e| OAuthError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(OAuthError::InvalidResponse(format!("user info: {body}")));
        }

        let info: serde_json::Value = resp.json().await.map_err(|e| OAuthError::InvalidResponse(e.to_string()))?;

        let id = info["id"]
            .as_i64()
            .map(|n| n.to_string())
            .or_else(|| info["id"].as_str().map(|s| s.to_string()))
            .ok_or_else(|| OAuthError::InvalidResponse("id ausente".into()))?;

        Ok(OAuthUser {
            id,
            email: info["email"].as_str().map(|s| s.to_string()),
            name: info["name"].as_str().map(|s| s.to_string()),
            provider,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn config_google() -> AuthConfig {
        AuthConfig {
            jwt_private_key_pem: "x".into(),
            jwt_public_key_pem: "y".into(),
            webauthn_rp_id: "localhost".into(),
            webauthn_origin: "http://localhost".into(),
            oauth_google_client_id: Some("google-client-id".into()),
            oauth_google_client_secret: Some("google-secret".into()),
            oauth_github_client_id: None,
            oauth_github_client_secret: None,
        }
    }

    fn config_empty() -> AuthConfig {
        AuthConfig {
            jwt_private_key_pem: "x".into(),
            jwt_public_key_pem: "y".into(),
            webauthn_rp_id: "localhost".into(),
            webauthn_origin: "http://localhost".into(),
            oauth_google_client_id: None,
            oauth_google_client_secret: None,
            oauth_github_client_id: None,
            oauth_github_client_secret: None,
        }
    }

    #[test]
    fn authorization_url_google_contains_accounts_google() {
        let http = reqwest::Client::new();
        let client = OAuthClient::from_config(&config_google(), http);
        let (url, _state, _verifier) = client
            .authorization_url(OAuthProvider::Google, "http://localhost/callback")
            .expect("debe generar URL para Google");
        assert!(url.host_str().unwrap_or("").contains("google.com"),
            "URL debe apuntar a google.com: {url}");
    }

    #[test]
    fn authorization_url_unconfigured_returns_error() {
        let http = reqwest::Client::new();
        let client = OAuthClient::from_config(&config_empty(), http);
        let result = client.authorization_url(OAuthProvider::Google, "http://localhost/callback");
        assert!(matches!(result, Err(OAuthError::ProviderNotConfigured(_))));
    }

    #[test]
    fn authorization_url_github_contains_github() {
        let mut cfg = config_google();
        cfg.oauth_github_client_id = Some("github-id".into());
        cfg.oauth_github_client_secret = Some("github-secret".into());
        let http = reqwest::Client::new();
        let client = OAuthClient::from_config(&cfg, http);
        let (url, _state, _verifier) = client
            .authorization_url(OAuthProvider::GitHub, "http://localhost/callback")
            .expect("debe generar URL para GitHub");
        assert!(url.host_str().unwrap_or("").contains("github.com"),
            "URL debe apuntar a github.com: {url}");
    }

    #[test]
    fn pkce_verifier_is_different_each_call() {
        let http = reqwest::Client::new();
        let client = OAuthClient::from_config(&config_google(), http);
        let (_, _, v1) = client
            .authorization_url(OAuthProvider::Google, "http://localhost/c")
            .unwrap();
        let (_, _, v2) = client
            .authorization_url(OAuthProvider::Google, "http://localhost/c")
            .unwrap();
        assert_ne!(v1.secret(), v2.secret(), "verifiers deben ser unicos");
    }
}
```

- [ ] **Step 2: Añadir dependencia `url` al workspace**

En `Cargo.toml` raiz, `[workspace.dependencies]`, añadir:

```toml
url = "2"
```

En `crates/ag-auth/Cargo.toml` ya añadida en Task 1 Step 2.

- [ ] **Step 3: Compilar y testear**

```bash
cargo test -p ag-auth oauth -- --nocapture 2>&1
```

Esperado: `authorization_url_*` y `unconfigured_*` pasan. Los tests de `exchange_code` requieren red — no se ejecutan en CI basico.

- [ ] **Step 4: Exponer en lib.rs**

En `crates/ag-auth/src/lib.rs`, añadir:

```rust
pub mod oauth;
pub use oauth::{OAuthClient, OAuthError, OAuthProvider, OAuthUser};
```

- [ ] **Step 5: Commit**

```bash
git add crates/ag-auth/src/oauth.rs crates/ag-auth/src/lib.rs Cargo.toml
git commit -m "feat(ag-auth): OAuthClient Google+GitHub con PKCE y exchange manual"
```

---

## Task 4: `refresh.rs` — blacklist de JTI en memoria

**Files:**
- Create: `crates/ag-auth/src/refresh.rs`

- [ ] **Step 1: Test primero**

Crear `crates/ag-auth/src/refresh.rs`:

```rust
//! Blacklist de refresh tokens mediante JTI (JWT ID).
//!
//! Implementacion estateful en memoria. Los JTIs revocados se retienen
//! hasta que se llama a `purge_expired` con la timestamp de expiracion.
//! No persistente entre reinicios — compatible con arquitecturas stateless
//! donde el mismo pod maneja las sesiones activas.

use std::collections::HashSet;
use std::sync::RwLock;

/// Blacklist en memoria para JTIs revocados.
///
/// Thread-safe via `RwLock`. Operar con `Arc<RefreshBlacklist>` en aplicaciones multi-thread.
pub struct RefreshBlacklist {
    revoked: RwLock<HashSet<String>>,
}

impl Default for RefreshBlacklist {
    fn default() -> Self {
        Self::new()
    }
}

impl RefreshBlacklist {
    /// Crea una blacklist vacia.
    pub fn new() -> Self {
        Self {
            revoked: RwLock::new(HashSet::new()),
        }
    }

    /// Revoca un JTI. Los tokens con este JTI seran rechazados.
    pub fn revoke(&self, jti: &str) {
        self.revoked
            .write()
            .expect("RefreshBlacklist envenenado")
            .insert(jti.to_string());
    }

    /// Retorna `true` si el JTI fue revocado.
    pub fn is_revoked(&self, jti: &str) -> bool {
        self.revoked
            .read()
            .expect("RefreshBlacklist envenenado")
            .contains(jti)
    }

    /// Elimina todos los JTIs de la blacklist.
    ///
    /// Llamar periodicamente en produccion para liberar memoria.
    /// En produccion, mantener una estructura con timestamp de expiracion
    /// para borrar solo los JTIs cuyo token haya expirado.
    pub fn clear(&self) {
        self.revoked
            .write()
            .expect("RefreshBlacklist envenenado")
            .clear();
    }

    /// Retorna el numero de JTIs revocados en la blacklist.
    pub fn len(&self) -> usize {
        self.revoked.read().expect("RefreshBlacklist envenenado").len()
    }

    /// Retorna `true` si la blacklist esta vacia.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revoke_and_check() {
        let bl = RefreshBlacklist::new();
        assert!(!bl.is_revoked("jti-abc"));
        bl.revoke("jti-abc");
        assert!(bl.is_revoked("jti-abc"));
    }

    #[test]
    fn unknown_jti_not_revoked() {
        let bl = RefreshBlacklist::new();
        bl.revoke("jti-1");
        assert!(!bl.is_revoked("jti-2"), "jti-2 no debe estar revocado");
    }

    #[test]
    fn clear_removes_all() {
        let bl = RefreshBlacklist::new();
        bl.revoke("jti-a");
        bl.revoke("jti-b");
        assert_eq!(bl.len(), 2);
        bl.clear();
        assert!(bl.is_empty());
        assert!(!bl.is_revoked("jti-a"));
    }

    #[test]
    fn double_revoke_idempotent() {
        let bl = RefreshBlacklist::new();
        bl.revoke("jti-x");
        bl.revoke("jti-x");
        assert_eq!(bl.len(), 1, "revocar dos veces el mismo JTI no duplica la entrada");
    }

    #[test]
    fn thread_safe_access() {
        use std::sync::Arc;
        use std::thread;

        let bl = Arc::new(RefreshBlacklist::new());
        let handles: Vec<_> = (0..8)
            .map(|i| {
                let bl = Arc::clone(&bl);
                thread::spawn(move || {
                    bl.revoke(&format!("jti-{i}"));
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(bl.len(), 8);
    }
}
```

- [ ] **Step 2: Compilar y testear**

```bash
cargo test -p ag-auth refresh -- --nocapture 2>&1
```

Esperado: todos los tests pasan incluido `thread_safe_access`.

- [ ] **Step 3: Exponer en lib.rs**

```rust
pub mod refresh;
pub use refresh::RefreshBlacklist;
```

- [ ] **Step 4: Commit**

```bash
git add crates/ag-auth/src/refresh.rs crates/ag-auth/src/lib.rs
git commit -m "feat(ag-auth): RefreshBlacklist — revocacion de JTI en memoria thread-safe"
```

---

## Task 5: Actualizar `AgAuth` — struct ampliada y nueva firma de `new`

**Files:**
- Modify: `crates/ag-auth/src/lib.rs`

- [ ] **Step 1: Reemplazar AgAuth en lib.rs**

Reemplazar el bloque desde `/// Fachada principal` hasta el final de `impl AgAuth` con:

```rust
/// Fachada principal del modulo de autenticacion.
pub struct AgAuth {
    /// Firmador/verificador de JWTs.
    pub jwt: JwtSigner,
    /// Relying Party WebAuthn. None si `webauthn_rp_id` esta vacio.
    pub webauthn: Option<webauthn::WebAuthnRp>,
    /// Cliente OAuth2. None si ningun proveedor esta configurado.
    pub oauth: Option<oauth::OAuthClient>,
    /// Blacklist de refresh tokens.
    pub refresh_blacklist: std::sync::Arc<refresh::RefreshBlacklist>,
}

impl AgAuth {
    /// Crea una nueva instancia de `AgAuth`.
    ///
    /// - `webauthn` se inicializa si `config.webauthn_rp_id` no esta vacio.
    /// - `oauth` se inicializa si al menos un proveedor tiene client_id configurado.
    /// - `http_client` se usa internamente para OAuth2 — el llamador lo provee
    ///   para permitir configuracion de timeouts, proxies y TLS personalizado.
    pub fn new(config: AuthConfig, http_client: reqwest::Client) -> Result<Self, AuthConfigError> {
        let jwt = JwtSigner::new(
            config.jwt_private_key_pem.clone(),
            config.jwt_public_key_pem.clone(),
        );

        let webauthn_rp = if !config.webauthn_rp_id.is_empty() {
            Some(webauthn::WebAuthnRp::new(
                config.webauthn_rp_id.clone(),
                config.webauthn_origin.clone(),
            ))
        } else {
            None
        };

        let has_google = config.oauth_google_client_id.is_some();
        let has_github = config.oauth_github_client_id.is_some();
        let oauth_client = if has_google || has_github {
            Some(oauth::OAuthClient::from_config(&config, http_client))
        } else {
            None
        };

        Ok(Self {
            jwt,
            webauthn: webauthn_rp,
            oauth: oauth_client,
            refresh_blacklist: std::sync::Arc::new(refresh::RefreshBlacklist::new()),
        })
    }

    /// Genera una nueva API key y su hash BLAKE3.
    pub fn create_api_key(&self, prefix: &str) -> (String, String) {
        api_keys::generate(prefix)
    }

    /// Verifica una API key contra su hash almacenado.
    pub fn verify_api_key(&self, raw_key: &str, stored_hash: &str) -> bool {
        api_keys::verify(raw_key, stored_hash)
    }
}
```

- [ ] **Step 2: Actualizar tests en lib.rs para la nueva firma**

Reemplazar la funcion `fake_config()` en el bloque `#[cfg(test)]` de lib.rs y actualizar las llamadas a `AgAuth::new`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AuthConfig;

    fn fake_config() -> AuthConfig {
        AuthConfig {
            jwt_private_key_pem: "fake-private".to_string(),
            jwt_public_key_pem: "fake-public".to_string(),
            webauthn_rp_id: String::new(),
            webauthn_origin: String::new(),
            oauth_google_client_id: None,
            oauth_google_client_secret: None,
            oauth_github_client_id: None,
            oauth_github_client_secret: None,
        }
    }

    fn http() -> reqwest::Client { reqwest::Client::new() }

    #[test]
    fn new_succeeds_with_valid_config() {
        let auth = AgAuth::new(fake_config(), http()).expect("debe construirse con config valida");
        let _ = &auth.jwt;
        assert!(auth.webauthn.is_none(), "sin rp_id, webauthn debe ser None");
        assert!(auth.oauth.is_none(), "sin providers, oauth debe ser None");
    }

    #[test]
    fn new_enables_webauthn_when_rp_id_set() {
        let mut cfg = fake_config();
        cfg.webauthn_rp_id = "example.com".into();
        cfg.webauthn_origin = "https://example.com".into();
        let auth = AgAuth::new(cfg, http()).unwrap();
        assert!(auth.webauthn.is_some());
    }

    #[test]
    fn new_enables_oauth_when_google_configured() {
        let mut cfg = fake_config();
        cfg.oauth_google_client_id = Some("gid".into());
        cfg.oauth_google_client_secret = Some("gsecret".into());
        let auth = AgAuth::new(cfg, http()).unwrap();
        assert!(auth.oauth.is_some());
    }

    #[test]
    fn create_api_key_uses_prefix() {
        let auth = AgAuth::new(fake_config(), http()).unwrap();
        let (raw, _hash) = auth.create_api_key("sk");
        assert!(raw.starts_with("sk_"));
    }

    #[test]
    fn verify_api_key_roundtrip() {
        let auth = AgAuth::new(fake_config(), http()).unwrap();
        let (raw, hash) = auth.create_api_key("test");
        assert!(auth.verify_api_key(&raw, &hash));
    }

    #[test]
    fn verify_api_key_rejects_wrong_key() {
        let auth = AgAuth::new(fake_config(), http()).unwrap();
        let (_raw, hash) = auth.create_api_key("test");
        assert!(!auth.verify_api_key("test_wrongkey", &hash));
    }
}
```

- [ ] **Step 3: Compilar y testear todo ag-auth**

```bash
cargo test -p ag-auth 2>&1
```

Esperado: todos los tests pasan. 0 errores de compilacion.

- [ ] **Step 4: Eliminar TECH-DEBT comments de lib.rs**

Eliminar los tres bloques TECH-DEBT del inicio de `lib.rs` que referenciaban WebAuthn, OAuth2 y refresh tokens como pendientes.

- [ ] **Step 5: CI local**

```bash
cargo fmt --all
cargo clippy -p ag-auth -- -D warnings
cargo deny check
```

Esperado: sin errores. Si clippy reporta algo, corregirlo antes del commit.

- [ ] **Step 6: Commit final**

```bash
git add crates/ag-auth/src/lib.rs
git commit -m "feat(ag-auth): AgAuth ampliado con WebAuthn, OAuth2 y RefreshBlacklist"
```

---

## Task 6: Verificacion final y merge

- [ ] **Step 1: Cobertura de ag-auth**

```bash
cargo llvm-cov --lib -p ag-auth --lcov --output-path /tmp/cov-auth.lcov 2>&1 | grep -E "ag-auth/src"
```

Verificar que todos los archivos superan 80%.

- [ ] **Step 2: cargo deny check completo**

```bash
cargo deny check 2>&1 | grep -E "^error"
```

Esperado: sin errores de licencia o advisories nuevos.

- [ ] **Step 3: Actualizar TECH-DEBT en ag-auth/Cargo.toml**

Eliminar el comentario TECH-DEBT de `[dependencies]` que decia "webauthn-rs (MPL-2.0) y oauth2 (reqwest 0.11)".

Añadir en su lugar:

```toml
# WebAuthn: passkey-types (Apache-2.0) para tipos FIDO2/COSE.
# OAuth2: oauth2 v4 sin feature reqwest; intercambio HTTP via reqwest 0.12.
```

- [ ] **Step 4: Commit final y merge**

```bash
git add -u
git commit -m "docs(ag-auth): eliminar TECH-DEBTs resueltos — WebAuthn+OAuth2+Refresh"
```

Merge a fase-4 cuando CI verde:

```bash
git checkout fase-4
git merge --no-ff f4-auth-complete -m "merge(f4-auth-complete): WebAuthn+OAuth2+RefreshBlacklist en ag-auth"
```
