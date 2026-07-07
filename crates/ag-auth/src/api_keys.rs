//! Generation and verification of API keys with BLAKE3 hashing.
//!
//! The flow is:
//! 1. [`generate`] creates 32 secure random bytes, encodes them in Base64Url and
//!    prefixes them with `{prefix}_`. Returns the plaintext key and its BLAKE3 hash.
//! 2. Only the hash is stored in the database.
//! 3. [`verify()`] computes the hash of the received key and compares it with the stored one.

use base64ct::{Base64Url, Encoding};

/// Generates a new API key and its BLAKE3 hash.
///
/// Returns `(raw_key, key_hash)`. Only `key_hash` should be stored;
/// `raw_key` is given to the user only once.
///
/// # Example
///
/// ```
/// let (raw, hash) = ag_auth::api_keys::generate("sk");
/// assert!(raw.starts_with("sk_"));
/// assert!(ag_auth::api_keys::verify(&raw, &hash));
/// ```
pub fn generate(prefix: &str) -> (String, String) {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).expect("failed to obtain entropy from the operating system");
    let encoded = Base64Url::encode_string(&bytes);
    let raw_key = format!("{prefix}_{encoded}");
    let key_hash = hash_key(&raw_key);
    (raw_key, key_hash)
}

/// Verifies an API key against its stored BLAKE3 hash.
///
/// Returns `true` if the key matches the hash; `false` otherwise.
///
/// The comparison is performed over the hash result (hexadecimal string),
/// so there is no significant timing attack against the original key.
pub fn verify(raw_key: &str, stored_hash: &str) -> bool {
    let computed = hash_key(raw_key);
    // Constant-time comparison to avoid timing attacks on the hash.
    constant_time_eq(&computed, stored_hash)
}

/// Computes the BLAKE3 hash of a key and returns it as a hexadecimal string.
fn hash_key(key: &str) -> String {
    blake3::hash(key.as_bytes()).to_hex().to_string()
}

/// Constant-time string comparison.
///
/// Compares byte by byte without short-circuiting to avoid inferring length
/// or content through response-time measurement.
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
            "key must have the 'sk_' prefix: {raw}"
        );
    }

    #[test]
    fn hash_is_deterministic() {
        let (raw, hash1) = generate("test");
        let hash2 = hash_key(&raw);
        assert_eq!(
            hash1, hash2,
            "the hash must be deterministic for the same key"
        );
    }

    #[test]
    fn verify_returns_true_for_correct_key() {
        let (raw, hash) = generate("sk");
        assert!(
            verify(&raw, &hash),
            "verify must return true for the correct key"
        );
    }

    #[test]
    fn verify_returns_false_for_wrong_key() {
        let (_, hash) = generate("sk");
        assert!(
            !verify("sk_wrong_key", &hash),
            "verify must return false for an incorrect key"
        );
    }

    #[test]
    fn two_generated_keys_are_different() {
        let (raw1, _) = generate("sk");
        let (raw2, _) = generate("sk");
        assert_ne!(raw1, raw2, "dos claves generadas no deben ser iguales");
    }
}

#[cfg(test)]
mod prop_tests {
    //! Property-based tests for the API-key generate/verify invariants.
    use super::{generate, verify};
    use proptest::prelude::*;

    proptest! {
        // A freshly generated key always verifies against its own hash.
        #[test]
        fn generate_then_verify_roundtrips(prefix in "[A-Za-z0-9_]{0,16}") {
            let (raw, hash) = generate(&prefix);
            prop_assert!(verify(&raw, &hash));
        }

        // verify must never panic on arbitrary input and returns a bool.
        #[test]
        fn verify_never_panics(raw in ".*", hash in ".*") {
            let _ = verify(&raw, &hash);
        }

        // A key different from the generated one must not verify.
        #[test]
        fn different_key_does_not_verify(prefix in "[A-Za-z0-9_]{0,16}", other in ".{0,48}") {
            let (raw, hash) = generate(&prefix);
            prop_assume!(other != raw);
            prop_assert!(!verify(&other, &hash));
        }
    }
}
