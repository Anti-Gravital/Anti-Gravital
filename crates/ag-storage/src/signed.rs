//! URLs firmadas con HMAC-SHA256 para ag-storage.
//!
//! Una URL firmada es valida solo si:
//! 1. La firma HMAC coincide (integridad + autenticidad).
//! 2. El timestamp `expires_at` no ha pasado.
//!
//! Formato del token: `{base64url(hmac)}_{expires_at_secs}`
//! La firma cubre: `key_bytes || expires_at_be_bytes`.

use base64ct::{Base64Url, Encoding};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Error al verificar una URL firmada.
#[derive(Debug, PartialEq)]
pub enum SignedUrlError {
    /// La firma no coincide (posible manipulacion).
    InvalidSignature,
    /// La URL ha expirado.
    Expired,
    /// Formato del token invalido.
    InvalidFormat,
    /// Clave secreta vacia (URLs firmadas no configuradas).
    NoSecret,
}

impl std::fmt::Display for SignedUrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignedUrlError::InvalidSignature => write!(f, "firma invalida"),
            SignedUrlError::Expired => write!(f, "URL expirada"),
            SignedUrlError::InvalidFormat => write!(f, "formato de token invalido"),
            SignedUrlError::NoSecret => write!(f, "STORAGE_SIGN_SECRET no configurado"),
        }
    }
}

impl std::error::Error for SignedUrlError {}

/// Genera la parte de firma del token para `key` con expiracion `expires_at`.
///
/// Retorna `"<base64url_hmac>_<expires_at>"`.
pub fn sign_url(secret: &str, key: &str, expires_at: u64) -> Result<String, SignedUrlError> {
    if secret.is_empty() {
        return Err(SignedUrlError::NoSecret);
    }
    let mac = compute_hmac(secret.as_bytes(), key, expires_at);
    let sig_b64 = Base64Url::encode_string(&mac);
    Ok(format!("{sig_b64}_{expires_at}"))
}

/// Verifica un token generado por `sign_url`.
///
/// Retorna `Ok(())` si la firma es valida y el token no ha expirado.
pub fn verify_signed_url(secret: &str, key: &str, token: &str) -> Result<(), SignedUrlError> {
    if secret.is_empty() {
        return Err(SignedUrlError::NoSecret);
    }

    let (sig_b64, expires_str) = token
        .rsplit_once('_')
        .ok_or(SignedUrlError::InvalidFormat)?;

    let expires_at: u64 = expires_str.parse().map_err(|_| SignedUrlError::InvalidFormat)?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if now > expires_at {
        return Err(SignedUrlError::Expired);
    }

    let expected_mac = compute_hmac(secret.as_bytes(), key, expires_at);
    let provided_sig = Base64Url::decode_vec(sig_b64)
        .map_err(|_| SignedUrlError::InvalidSignature)?;

    if expected_mac.len() != provided_sig.len() {
        return Err(SignedUrlError::InvalidSignature);
    }
    let mismatch = expected_mac
        .iter()
        .zip(provided_sig.iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b));
    if mismatch != 0 {
        return Err(SignedUrlError::InvalidSignature);
    }

    Ok(())
}

fn compute_hmac(secret: &[u8], key: &str, expires_at: u64) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(secret)
        .expect("HMAC acepta claves de cualquier longitud");
    mac.update(key.as_bytes());
    mac.update(&expires_at.to_be_bytes());
    mac.finalize().into_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn future_ts() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600
    }

    fn past_ts() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .saturating_sub(3600)
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let secret = "super-secret-key";
        let key = "avatars/user-42.jpg";
        let expires = future_ts();
        let token = sign_url(secret, key, expires).expect("sign debe tener exito");
        verify_signed_url(secret, key, &token).expect("verify debe tener exito");
    }

    #[test]
    fn expired_token_rejected() {
        let secret = "secret";
        let key = "docs/report.pdf";
        let token = sign_url(secret, key, past_ts()).unwrap();
        let result = verify_signed_url(secret, key, &token);
        assert_eq!(result, Err(SignedUrlError::Expired));
    }

    #[test]
    fn wrong_key_rejected() {
        let secret = "secret";
        let expires = future_ts();
        let token = sign_url(secret, "original-key", expires).unwrap();
        let result = verify_signed_url(secret, "other-key", &token);
        assert_eq!(result, Err(SignedUrlError::InvalidSignature));
    }

    #[test]
    fn wrong_secret_rejected() {
        let expires = future_ts();
        let key = "file.txt";
        let token = sign_url("correct-secret", key, expires).unwrap();
        let result = verify_signed_url("wrong-secret", key, &token);
        assert_eq!(result, Err(SignedUrlError::InvalidSignature));
    }

    #[test]
    fn tampered_token_rejected() {
        let secret = "secret";
        let expires = future_ts();
        let token = sign_url(secret, "file.txt", expires).unwrap();
        // Sustituir el ultimo caracter de la firma (antes del '_') para simular
        // manipulacion del token sin usar unsafe.
        let underscore_pos = token.find('_').unwrap();
        let tampered = if underscore_pos > 0 {
            let mut chars: Vec<char> = token.chars().collect();
            let idx = underscore_pos - 1;
            chars[idx] = if chars[idx] == 'A' { 'B' } else { 'A' };
            chars.into_iter().collect::<String>()
        } else {
            format!("X{}", &token[1..])
        };
        let result = verify_signed_url(secret, "file.txt", &tampered);
        assert!(result.is_err());
    }

    #[test]
    fn empty_secret_returns_no_secret_error() {
        let result = sign_url("", "key.txt", future_ts());
        assert_eq!(result, Err(SignedUrlError::NoSecret));
    }

    #[test]
    fn invalid_token_format_rejected() {
        let result = verify_signed_url("secret", "key.txt", "not-a-token");
        assert_eq!(result, Err(SignedUrlError::InvalidFormat));
    }

    #[test]
    fn different_keys_produce_different_tokens() {
        let secret = "s";
        let expires = future_ts();
        let t1 = sign_url(secret, "file1.txt", expires).unwrap();
        let t2 = sign_url(secret, "file2.txt", expires).unwrap();
        assert_ne!(t1, t2);
    }
}
