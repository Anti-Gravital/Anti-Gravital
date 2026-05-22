//! Generacion y verificacion de API keys con hash BLAKE3.
//!
//! El flujo es:
//! 1. [`generate`] crea 32 bytes aleatorios seguros, los codifica en Base64Url y los
//!    prefija con `{prefix}_`. Retorna la clave en texto plano y su hash BLAKE3.
//! 2. Solo el hash se almacena en la base de datos.
//! 3. [`verify`] computa el hash de la clave recibida y lo compara con el almacenado.

use base64ct::{Base64Url, Encoding};

/// Genera una nueva API key y su hash BLAKE3.
///
/// Retorna `(raw_key, key_hash)`. Solo `key_hash` debe almacenarse;
/// `raw_key` se entrega al usuario una unica vez.
///
/// # Ejemplo
///
/// ```
/// let (raw, hash) = ag_auth::api_keys::generate("sk");
/// assert!(raw.starts_with("sk_"));
/// assert!(ag_auth::api_keys::verify(&raw, &hash));
/// ```
pub fn generate(prefix: &str) -> (String, String) {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).expect("fallo al obtener entropia del sistema operativo");
    let encoded = Base64Url::encode_string(&bytes);
    let raw_key = format!("{prefix}_{encoded}");
    let key_hash = hash_key(&raw_key);
    (raw_key, key_hash)
}

/// Verifica una API key contra su hash BLAKE3 almacenado.
///
/// Retorna `true` si la clave corresponde al hash; `false` en caso contrario.
///
/// La comparacion se realiza sobre el resultado del hash (cadena hexadecimal),
/// por lo que no hay timing attack significativo contra la clave original.
pub fn verify(raw_key: &str, stored_hash: &str) -> bool {
    let computed = hash_key(raw_key);
    // Comparacion en tiempo constante para evitar timing attacks sobre el hash.
    constant_time_eq(&computed, stored_hash)
}

/// Computa el hash BLAKE3 de una clave y lo retorna como cadena hexadecimal.
fn hash_key(key: &str) -> String {
    blake3::hash(key.as_bytes()).to_hex().to_string()
}

/// Comparacion de cadenas en tiempo constante.
///
/// Compara byte a byte sin cortocircuito para evitar inferencia de longitud
/// o contenido mediante medicion de tiempo de respuesta.
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_key_has_prefix() {
        let (raw, _) = generate("sk");
        assert!(
            raw.starts_with("sk_"),
            "key debe tener prefijo 'sk_': {raw}"
        );
    }

    #[test]
    fn hash_is_deterministic() {
        let (raw, hash1) = generate("test");
        let hash2 = hash_key(&raw);
        assert_eq!(
            hash1, hash2,
            "el hash debe ser deterministico para la misma clave"
        );
    }

    #[test]
    fn verify_returns_true_for_correct_key() {
        let (raw, hash) = generate("sk");
        assert!(
            verify(&raw, &hash),
            "verify debe retornar true para la clave correcta"
        );
    }

    #[test]
    fn verify_returns_false_for_wrong_key() {
        let (_, hash) = generate("sk");
        assert!(
            !verify("sk_clave_incorrecta", &hash),
            "verify debe retornar false para una clave incorrecta"
        );
    }

    #[test]
    fn two_generated_keys_are_different() {
        let (raw1, _) = generate("sk");
        let (raw2, _) = generate("sk");
        assert_ne!(raw1, raw2, "dos claves generadas no deben ser iguales");
    }
}
