//! Refresh token blacklist via JTI (JWT ID).
//!
//! Stateful in-memory implementation. Revoked JTIs are retained
//! until `clear` is called to free memory.
//! Not persistent across restarts — compatible with stateless architectures
//! where the same pod handles active sessions.

use std::collections::HashSet;
use std::sync::RwLock;

/// In-memory blacklist for revoked JTIs.
///
/// Thread-safe via `RwLock`. Operate with `Arc<RefreshBlacklist>` in multi-thread applications.
pub struct RefreshBlacklist {
    revoked: RwLock<HashSet<String>>,
}

impl Default for RefreshBlacklist {
    fn default() -> Self {
        Self::new()
    }
}

impl RefreshBlacklist {
    /// Creates an empty blacklist.
    pub fn new() -> Self {
        Self {
            revoked: RwLock::new(HashSet::new()),
        }
    }

    /// Revokes a JTI. Tokens with this JTI will be rejected.
    pub fn revoke(&self, jti: &str) {
        self.revoked
            .write()
            .expect("RefreshBlacklist poisoned")
            .insert(jti.to_string());
    }

    /// Returns `true` if the JTI has been revoked.
    pub fn is_revoked(&self, jti: &str) -> bool {
        self.revoked
            .read()
            .expect("RefreshBlacklist poisoned")
            .contains(jti)
    }

    /// Removes all JTIs from the blacklist.
    ///
    /// Call periodically in production to free memory.
    /// In production, maintain a structure with an expiration timestamp
    /// to remove only the JTIs whose token has expired.
    pub fn clear(&self) {
        self.revoked
            .write()
            .expect("RefreshBlacklist poisoned")
            .clear();
    }

    /// Returns the number of revoked JTIs in the blacklist.
    pub fn len(&self) -> usize {
        self.revoked
            .read()
            .expect("RefreshBlacklist poisoned")
            .len()
    }

    /// Returns `true` if the blacklist is empty.
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
        assert!(!bl.is_revoked("jti-2"), "jti-2 should not be revoked");
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
        assert_eq!(
            bl.len(),
            1,
            "revoking the same JTI twice does not duplicate the entry"
        );
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
