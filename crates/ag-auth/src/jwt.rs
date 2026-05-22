//! Firma y verificacion de JSON Web Tokens con el algoritmo Ed25519 (EdDSA).
//!
//! Usa [`jsonwebtoken`] con claves en formato PEM PKCS#8. Las claves deben
//! generarse fuera del crate (openssl, age, etc.) y pasarse via `AuthConfig`.

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// Claims estandar de un JWT emitido por Anti-Gravital.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Claims {
    /// Sujeto del token (normalmente el UUID del usuario).
    pub sub: String,
    /// Timestamp de expiracion (segundos desde Unix epoch).
    pub exp: u64,
    /// Timestamp de emision (segundos desde Unix epoch).
    pub iat: u64,
    /// Identificador unico del token. Permite revocacion por JTI.
    pub jti: String,
    /// Rol del usuario en el momento de la emision.
    pub role: String,
}

/// Firmador y verificador de JWTs Ed25519.
#[derive(Clone)]
pub struct JwtSigner {
    private_key_pem: String,
    public_key_pem: String,
}

impl JwtSigner {
    /// Crea un nuevo `JwtSigner` a partir de claves PEM.
    ///
    /// Las claves deben estar en formato PKCS#8 Ed25519.
    /// La validacion de formato ocurre en [`JwtSigner::sign`] y [`JwtSigner::verify`], no aqui.
    pub fn new(private_key_pem: String, public_key_pem: String) -> Self {
        Self {
            private_key_pem,
            public_key_pem,
        }
    }

    /// Firma un conjunto de claims y retorna el JWT compacto.
    ///
    /// # Errors
    ///
    /// Retorna [`JwtError::Signing`] si la clave privada es invalida o la
    /// firma falla.
    pub fn sign(&self, claims: &Claims) -> Result<String, JwtError> {
        let key = EncodingKey::from_ed_pem(self.private_key_pem.as_bytes())
            .map_err(|e| JwtError::Signing(e.to_string()))?;
        let header = Header::new(Algorithm::EdDSA);
        jsonwebtoken::encode(&header, claims, &key).map_err(|e| JwtError::Signing(e.to_string()))
    }

    /// Verifica un JWT y retorna los claims si la firma y expiracion son validos.
    ///
    /// Emite un warning de tracing cuando la verificacion falla, incluyendo el motivo.
    ///
    /// # Errors
    ///
    /// - [`JwtError::Verification`] si la firma es invalida o el token esta expirado.
    /// - [`JwtError::InvalidToken`] si el formato del token es incorrecto.
    pub fn verify(&self, token: &str) -> Result<Claims, JwtError> {
        let key = DecodingKey::from_ed_pem(self.public_key_pem.as_bytes())
            .map_err(|e| JwtError::Verification(e.to_string()))?;
        let mut validation = Validation::new(Algorithm::EdDSA);
        // No validar `aud` por defecto; ag-auth no emite audiencia en los tokens.
        validation.validate_aud = false;
        jsonwebtoken::decode::<Claims>(token, &key, &validation)
            .map(|data| data.claims)
            .map_err(|e| {
                tracing::warn!(reason = %e, "verificacion JWT fallida");
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::InvalidToken => JwtError::InvalidToken,
                    _ => JwtError::Verification(e.to_string()),
                }
            })
    }
}

/// Errores del modulo JWT.
#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    /// Error al firmar: clave invalida o fallo interno.
    #[error("error de firma JWT: {0}")]
    Signing(String),
    /// Token con firma incorrecta, expirado u otro problema de verificacion.
    #[error("verificacion JWT fallida: {0}")]
    Verification(String),
    /// Formato del token incorrecto (no es JWT valido).
    #[error("formato de token JWT invalido")]
    InvalidToken,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Genera un par de claves Ed25519 PKCS#8 en PEM para uso en tests.
    ///
    /// Usa `ed25519-dalek` con `pkcs8` y `pem` features (disponibles en
    /// `[workspace.dependencies]`).
    fn generate_test_keypair() -> (String, String) {
        use ed25519_dalek::pkcs8::spki::der::pem::LineEnding;
        use ed25519_dalek::pkcs8::{EncodePrivateKey, EncodePublicKey};
        use ed25519_dalek::SigningKey;
        use rand_core::OsRng;

        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        let private_pem = signing_key
            .to_pkcs8_pem(LineEnding::LF)
            .expect("fallo al codificar clave privada en PKCS8 PEM")
            .to_string();
        let public_pem = verifying_key
            .to_public_key_pem(LineEnding::LF)
            .expect("fallo al codificar clave publica en SPKI PEM");

        (private_pem, public_pem)
    }

    fn make_claims(exp_offset_secs: i64) -> Claims {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("reloj del sistema invalido")
            .as_secs();
        let exp = if exp_offset_secs >= 0 {
            now + exp_offset_secs as u64
        } else {
            now.saturating_sub((-exp_offset_secs) as u64)
        };
        Claims {
            sub: "user-uuid-1234".to_string(),
            exp,
            iat: now,
            jti: "jti-abcdef".to_string(),
            role: "admin".to_string(),
        }
    }

    #[test]
    fn jwt_sign_and_verify_roundtrip() {
        let (priv_pem, pub_pem) = generate_test_keypair();
        let signer = JwtSigner::new(priv_pem, pub_pem);
        let claims = make_claims(3600);
        let token = signer.sign(&claims).expect("firma debe tener exito");
        let verified = signer
            .verify(&token)
            .expect("verificacion debe tener exito");
        assert_eq!(verified.sub, claims.sub);
        assert_eq!(verified.role, claims.role);
        assert_eq!(verified.jti, claims.jti);
    }

    #[test]
    fn jwt_expired_token_returns_error() {
        let (priv_pem, pub_pem) = generate_test_keypair();
        let signer = JwtSigner::new(priv_pem, pub_pem);
        // Crear token con exp en el pasado.
        let claims = make_claims(-3600);
        let token = signer.sign(&claims).expect("firma debe tener exito");
        let result = signer.verify(&token);
        assert!(result.is_err(), "token expirado debe retornar Err");
    }

    #[test]
    fn jwt_tampered_token_fails_verification() {
        let (priv_pem, pub_pem) = generate_test_keypair();
        let signer = JwtSigner::new(priv_pem, pub_pem);
        let claims = make_claims(3600);
        let token = signer.sign(&claims).expect("firma debe tener exito");
        // Modificar el ultimo byte de la firma para invalidar el token.
        let mut tampered = token.clone();
        tampered.pop();
        tampered.push(if token.ends_with('A') { 'B' } else { 'A' });
        let result = signer.verify(&tampered);
        assert!(result.is_err(), "token manipulado debe retornar Err");
    }

    #[test]
    fn jwt_invalid_key_returns_error() {
        let signer = JwtSigner::new("not-a-pem-key".to_string(), "not-a-pem-key".to_string());
        let claims = make_claims(3600);
        let result = signer.sign(&claims);
        assert!(
            result.is_err(),
            "clave PEM malformada debe retornar Err en sign"
        );
    }
}
