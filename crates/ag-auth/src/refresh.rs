//! Blacklist de refresh tokens mediante JTI (JWT ID).
//!
//! Implementacion estateful en memoria. Los JTIs revocados se retienen
//! hasta que se llama a `clear` para liberar memoria.
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
        self.revoked
            .read()
            .expect("RefreshBlacklist envenenado")
            .len()
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
        assert_eq!(
            bl.len(),
            1,
            "revocar dos veces el mismo JTI no duplica la entrada"
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
