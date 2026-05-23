//! Cache multinivel para el ecosistema Anti-Gravital.
//!
//! Ofrece dos niveles transparentes:
//! - **L1**: moka en memoria (TinyLFU, sin locks contenciosos, siempre disponible).
//! - **L2**: Redis via fred (opcional, para cache distribuida entre instancias).
//!
//! # Estado
//!
//! L1 completamente operativo. L2 (Redis/fred) queda pendiente como TECH-DEBT
//! para la segunda iteracion de ag-cache en Fase 4 — la API de fred v10
//! requiere ajuste de features y configuracion de conexion en CI.
//!
//! # Uso minimo (solo L1)
//!
//! ```no_run
//! use ag_cache::{AgCache, CacheConfig};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let cache = AgCache::new(CacheConfig::default()).await?;
//! cache.set("user:123", b"datos".to_vec(), &[]).await;
//! let val: Option<Vec<u8>> = cache.get("user:123").await;
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod l1;
pub mod tags;

pub use config::CacheConfig;

use l1::L1Cache;
use std::time::Duration;

/// Error del subsistema de cache.
#[derive(Debug)]
pub enum CacheError {
    /// Error de conexion o comunicacion con Redis.
    Redis(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::Redis(msg) => write!(f, "error Redis: {msg}"),
        }
    }
}

impl std::error::Error for CacheError {}

/// Cache multinivel con L1 (moka) y L2 opcional (Redis).
///
/// Construir con [`AgCache::new`] pasando una [`CacheConfig`]. Si
/// `config.redis_url` es `None`, solo L1 esta activo.
pub struct AgCache {
    l1: L1Cache,
    // TECH-DEBT:
    // motivo: L2 Redis requiere fred con conexion real; la API de fred v10
    //         no expone las features documentadas (tokio-runtime, codec).
    //         Se integra en la segunda iteracion de ag-cache en Fase 4.
    // impacto: sin L2, la invalidacion distribuida entre instancias no funciona.
    // eliminacion esperada: segunda iteracion ag-cache, Fase 4.
}

impl AgCache {
    /// Crea un nuevo [`AgCache`] con la configuracion dada.
    ///
    /// Si `config.redis_url` es `Some`, emite un aviso en tracing pero L2
    /// no se activa hasta que este implementado (ver TECH-DEBT en el codigo).
    pub async fn new(config: CacheConfig) -> Result<Self, CacheError> {
        let ttl = Duration::from_secs(config.l1_ttl_secs);
        let l1 = L1Cache::new(config.l1_max_capacity, ttl);

        if config.redis_url.is_some() {
            tracing::warn!(
                "REDIS_URL configurada pero L2 Redis no esta activo en esta version (TECH-DEBT)"
            );
        }

        Ok(Self { l1 })
    }

    /// Obtiene bytes crudos desde el cache.
    ///
    /// Busca primero en L1. Si hay un hit, registra `cache hit L1` en tracing.
    /// Si no hay resultado, registra `cache miss`.
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let result = self.l1.get_bytes(key).await;
        if result.is_some() {
            tracing::debug!(key, "cache hit L1");
        } else {
            tracing::debug!(key, "cache miss");
        }
        result
    }

    /// Almacena bytes en el cache con tags opcionales para invalidacion.
    ///
    /// Escribe en L1. Si `tags` esta vacio, no registra tags.
    pub async fn set(&self, key: &str, value: Vec<u8>, tags: &[&str]) {
        if tags.is_empty() {
            self.l1.set_bytes(key, value).await;
        } else {
            self.l1.set_bytes_tagged(key, value, tags).await;
        }
    }

    /// Invalida todas las entradas asociadas al tag dado en L1.
    pub async fn invalidate_tag(&self, tag: &str) {
        self.l1.invalidate_tag(tag).await;
    }

    /// Elimina la entrada con la clave dada.
    pub async fn delete(&self, key: &str) {
        self.l1.delete(key).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_without_redis_works() {
        let cache = AgCache::new(CacheConfig::default()).await.unwrap();
        cache.set("k", b"v".to_vec(), &[]).await;
        assert_eq!(cache.get("k").await, Some(b"v".to_vec()));
    }

    #[tokio::test]
    async fn invalidate_tag_clears_entries() {
        let cache = AgCache::new(CacheConfig::default()).await.unwrap();
        cache.set("u:1", b"a".to_vec(), &["users"]).await;
        cache.set("u:2", b"b".to_vec(), &["users"]).await;
        cache.invalidate_tag("users").await;
        assert!(cache.get("u:1").await.is_none());
        assert!(cache.get("u:2").await.is_none());
    }

    #[tokio::test]
    async fn delete_removes_single_entry() {
        let cache = AgCache::new(CacheConfig::default()).await.unwrap();
        cache.set("k", b"v".to_vec(), &[]).await;
        cache.delete("k").await;
        assert!(cache.get("k").await.is_none());
    }

    #[tokio::test]
    async fn get_returns_none_for_missing_key() {
        let cache = AgCache::new(CacheConfig::default()).await.unwrap();
        assert!(cache.get("nonexistent").await.is_none());
    }
}
