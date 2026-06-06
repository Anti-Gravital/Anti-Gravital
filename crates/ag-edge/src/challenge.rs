//! ACME HTTP-01 challenge responder state.
//!
//! During an HTTP-01 challenge (RFC 8555 section 8.3) the CA fetches
//! `http://<domain>/.well-known/acme-challenge/<token>` and expects the key
//! authorization string in response. The control plane registers the
//! `(token, key_authorization)` pair here; the edge HTTP listener serves it.
//!
//! The store is cleared per token after validation by the issuance flow.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// The well-known path prefix for ACME HTTP-01 challenges.
pub const ACME_CHALLENGE_PREFIX: &str = "/.well-known/acme-challenge/";

/// Thread-safe map of challenge token to key authorization.
#[derive(Debug, Clone, Default)]
pub struct Http01ChallengeStore {
    inner: Arc<RwLock<HashMap<String, String>>>,
}

impl Http01ChallengeStore {
    /// Creates an empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a token and its key authorization.
    pub fn set(&self, token: impl Into<String>, key_authorization: impl Into<String>) {
        self.inner
            .write()
            .expect("challenge store lock poisoned")
            .insert(token.into(), key_authorization.into());
    }

    /// Returns the key authorization for a token, if present.
    pub fn get(&self, token: &str) -> Option<String> {
        self.inner
            .read()
            .expect("challenge store lock poisoned")
            .get(token)
            .cloned()
    }

    /// Removes a token after validation.
    pub fn remove(&self, token: &str) {
        self.inner
            .write()
            .expect("challenge store lock poisoned")
            .remove(token);
    }

    /// Number of outstanding challenges.
    pub fn len(&self) -> usize {
        self.inner
            .read()
            .expect("challenge store lock poisoned")
            .len()
    }

    /// Whether there are no outstanding challenges.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Extracts the ACME challenge token from a request path, if it is a
/// well-known ACME challenge path. The token must be a single non-empty path
/// segment (no further slashes), which prevents path traversal.
pub fn acme_challenge_token(path: &str) -> Option<&str> {
    let token = path.strip_prefix(ACME_CHALLENGE_PREFIX)?;
    if token.is_empty() || token.contains('/') {
        return None;
    }
    Some(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_remove() {
        let store = Http01ChallengeStore::new();
        assert!(store.is_empty());
        store.set("tok123", "tok123.keyauth");
        assert_eq!(store.get("tok123").as_deref(), Some("tok123.keyauth"));
        assert_eq!(store.len(), 1);
        store.remove("tok123");
        assert!(store.get("tok123").is_none());
        assert!(store.is_empty());
    }

    #[test]
    fn token_extraction() {
        assert_eq!(
            acme_challenge_token("/.well-known/acme-challenge/abc123"),
            Some("abc123")
        );
    }

    #[test]
    fn rejects_non_challenge_paths() {
        assert_eq!(acme_challenge_token("/"), None);
        assert_eq!(acme_challenge_token("/index.html"), None);
        assert_eq!(acme_challenge_token("/.well-known/acme-challenge/"), None);
    }

    #[test]
    fn rejects_traversal_in_token() {
        assert_eq!(
            acme_challenge_token("/.well-known/acme-challenge/a/b"),
            None
        );
        assert_eq!(
            acme_challenge_token("/.well-known/acme-challenge/../secret"),
            None
        );
    }
}
