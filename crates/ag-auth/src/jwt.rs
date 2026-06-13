//! Signing and verification of JSON Web Tokens with the Ed25519 (EdDSA) algorithm.
//!
//! Uses [`jsonwebtoken`] with keys in PEM PKCS#8 format. The keys must be
//! generated outside the crate (openssl, age, etc.) and passed via `AuthConfig`.

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// Standard claims of a JWT issued by Anti-Gravital.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Claims {
    /// Subject of the token (usually the user's UUID).
    pub sub: String,
    /// Expiration timestamp (seconds since Unix epoch).
    pub exp: u64,
    /// Issued-at timestamp (seconds since Unix epoch).
    pub iat: u64,
    /// Unique identifier of the token. Allows revocation by JTI.
    pub jti: String,
    /// User role at the moment of issuance.
    pub role: String,
}

/// Ed25519 JWT signer and verifier.
#[derive(Clone)]
pub struct JwtSigner {
    private_key_pem: String,
    public_key_pem: String,
}

impl JwtSigner {
    /// Creates a new `JwtSigner` from PEM keys.
    ///
    /// The keys must be in PKCS#8 Ed25519 format.
    /// Format validation occurs in [`JwtSigner::sign`] and [`JwtSigner::verify`], not here.
    pub fn new(private_key_pem: String, public_key_pem: String) -> Self {
        Self {
            private_key_pem,
            public_key_pem,
        }
    }

    /// Signs a set of claims and returns the compact JWT.
    ///
    /// # Errors
    ///
    /// Returns [`JwtError::Signing`] if the private key is invalid or
    /// signing fails.
    pub fn sign(&self, claims: &Claims) -> Result<String, JwtError> {
        let key = EncodingKey::from_ed_pem(self.private_key_pem.as_bytes())
            .map_err(|e| JwtError::Signing(e.to_string()))?;
        let header = Header::new(Algorithm::EdDSA);
        jsonwebtoken::encode(&header, claims, &key).map_err(|e| JwtError::Signing(e.to_string()))
    }

    /// Verifies a JWT and returns the claims if the signature and expiration are valid.
    ///
    /// Emits a tracing warning when verification fails, including the reason.
    ///
    /// # Errors
    ///
    /// - [`JwtError::Verification`] if the signature is invalid or the token is expired.
    /// - [`JwtError::InvalidToken`] if the token format is incorrect.
    pub fn verify(&self, token: &str) -> Result<Claims, JwtError> {
        let key = DecodingKey::from_ed_pem(self.public_key_pem.as_bytes())
            .map_err(|e| JwtError::Verification(e.to_string()))?;
        let mut validation = Validation::new(Algorithm::EdDSA);
        // Do not validate `aud` by default; ag-auth does not emit an audience in tokens.
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

/// Errors of the JWT module.
#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    /// Signing error: invalid key or internal failure.
    #[error("JWT signing error: {0}")]
    Signing(String),
    /// Token with an incorrect signature, expired, or another verification problem.
    #[error("JWT verification failed: {0}")]
    Verification(String),
    /// Incorrect token format (not a valid JWT).
    #[error("invalid JWT token format")]
    InvalidToken,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Generates an Ed25519 PKCS#8 PEM keypair for use in tests.
    ///
    /// Uses `ed25519-dalek` with the `pkcs8` and `pem` features (available in
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
            .expect("failed to encode private key as PKCS8 PEM")
            .to_string();
        let public_pem = verifying_key
            .to_public_key_pem(LineEnding::LF)
            .expect("failed to encode public key as SPKI PEM");

        (private_pem, public_pem)
    }

    fn make_claims(exp_offset_secs: i64) -> Claims {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("invalid system clock")
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
        let token = signer.sign(&claims).expect("signing must succeed");
        let verified = signer.verify(&token).expect("verification must succeed");
        assert_eq!(verified.sub, claims.sub);
        assert_eq!(verified.role, claims.role);
        assert_eq!(verified.jti, claims.jti);
    }

    #[test]
    fn jwt_expired_token_returns_error() {
        let (priv_pem, pub_pem) = generate_test_keypair();
        let signer = JwtSigner::new(priv_pem, pub_pem);
        // Create a token with exp in the past.
        let claims = make_claims(-3600);
        let token = signer.sign(&claims).expect("signing must succeed");
        let result = signer.verify(&token);
        assert!(result.is_err(), "expired token must return Err");
    }

    #[test]
    fn jwt_tampered_token_fails_verification() {
        let (priv_pem, pub_pem) = generate_test_keypair();
        let signer = JwtSigner::new(priv_pem, pub_pem);
        let claims = make_claims(3600);
        let token = signer.sign(&claims).expect("signing must succeed");
        // Flip the last byte of the signature to invalidate the token.
        let mut tampered = token.clone();
        tampered.pop();
        tampered.push(if token.ends_with('A') { 'B' } else { 'A' });
        let result = signer.verify(&tampered);
        assert!(result.is_err(), "tampered token must return Err");
    }

    #[test]
    fn jwt_invalid_key_returns_error() {
        let signer = JwtSigner::new("not-a-pem-key".to_string(), "not-a-pem-key".to_string());
        let claims = make_claims(3600);
        let result = signer.sign(&claims);
        assert!(result.is_err(), "malformed PEM key must return Err on sign");
    }
}
