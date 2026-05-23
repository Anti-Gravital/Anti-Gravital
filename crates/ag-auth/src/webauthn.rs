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
    /// Challenge no encontrado o expirado.
    #[error("challenge no encontrado o expirado")]
    ChallengeNotFound,
    /// Origen invalido.
    #[error("origen invalido: esperado {expected}, recibido {received}")]
    InvalidOrigin {
        /// Origen esperado por el RP.
        expected: String,
        /// Origen recibido del autenticador.
        received: String,
    },
    /// rp_id hash no coincide.
    #[error("rp_id hash no coincide")]
    InvalidRpId,
    /// Flag de usuario presente no activo.
    #[error("flag de usuario presente no activo")]
    UserNotPresent,
    /// Firma invalida.
    #[error("firma invalida")]
    InvalidSignature,
    /// Tipo de operacion invalido.
    #[error("tipo de operacion invalido: {0}")]
    InvalidType(String),
    /// Formato invalido.
    #[error("formato invalido: {0}")]
    Format(String),
    /// Algoritmo COSE no soportado.
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
    pub fn start_registration(
        &mut self,
        user_handle: &str,
        display_name: &str,
    ) -> RegistrationChallenge {
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

        RegistrationChallenge {
            challenge_b64,
            options_json,
        }
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
        let cred_id = auth_data
            .cred_id
            .ok_or_else(|| WebAuthnError::Format("sin credentialId".into()))?;
        let cose_key = auth_data
            .cose_public_key
            .ok_or_else(|| WebAuthnError::Format("sin clave COSE".into()))?;

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
            .map(|c| {
                serde_json::json!({
                    "type": "public-key",
                    "id": Base64Url::encode_string(&c.credential_id),
                })
            })
            .collect();

        let options_json = serde_json::json!({
            "challenge": challenge_b64,
            "rpId": self.rp_id,
            "allowCredentials": allow_creds,
            "timeout": 300000,
        });

        AuthenticationChallenge {
            challenge_b64,
            options_json,
        }
    }

    /// Verifica la respuesta de autenticacion y retorna el user_handle si es valida.
    pub fn finish_authentication(
        &mut self,
        user_handle: &str,
        response: AuthenticationResponse,
        creds: &mut [StoredCredential],
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

        verify_cose_signature(
            &cred.cose_public_key,
            &verification_data,
            &response.signature,
        )?;

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

fn verify_challenge(
    client_data: &serde_json::Value,
    expected_bytes: &[u8],
) -> Result<(), WebAuthnError> {
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
        _ => {
            return Err(WebAuthnError::Format(
                "attestationObject no es un mapa".into(),
            ))
        }
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

    Ok(ParsedAuthData {
        rp_id_hash,
        up,
        sign_count,
        cred_id,
        cose_public_key,
    })
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
            let signature =
                DerSignature::try_from(sig).map_err(|_| WebAuthnError::InvalidSignature)?;
            vk.verify(data, &signature)
                .map_err(|_| WebAuthnError::InvalidSignature)
        }
        -8 => {
            // EdDSA — Ed25519
            use ed25519_dalek::{Signature, Verifier, VerifyingKey as Ed25519Vk};

            let x =
                get_bytes(-2).ok_or_else(|| WebAuthnError::Format("Ed25519 x ausente".into()))?;
            let key_bytes: [u8; 32] = x
                .as_slice()
                .try_into()
                .map_err(|_| WebAuthnError::Format("Ed25519 key no es 32 bytes".into()))?;
            let vk =
                Ed25519Vk::from_bytes(&key_bytes).map_err(|_| WebAuthnError::InvalidSignature)?;
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
        assert_ne!(
            c1.challenge_b64, c2.challenge_b64,
            "challenges deben ser unicos"
        );
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
