//! Configuracion del cache multinivel.

/// Configuracion del cache L1 (moka, en memoria) y L2 (Redis, opcional).
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Numero maximo de entradas en L1. Default: 10_000.
    pub l1_max_capacity: u64,
    /// TTL por defecto para entradas L1 en segundos. Default: 300.
    pub l1_ttl_secs: u64,
    /// URL de conexion Redis para L2. None desactiva L2.
    /// Variable de entorno: REDIS_URL
    pub redis_url: Option<String>,
    /// Numero de conexiones en el pool Redis. Default: 10.
    pub redis_pool_size: u32,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            l1_max_capacity: 10_000,
            l1_ttl_secs: 300,
            redis_url: None,
            redis_pool_size: 10,
        }
    }
}

impl CacheConfig {
    /// Lee la configuracion desde variables de entorno.
    pub fn from_env() -> Self {
        Self {
            l1_max_capacity: std::env::var("CACHE_L1_MAX_CAPACITY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10_000),
            l1_ttl_secs: std::env::var("CACHE_L1_TTL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            redis_url: std::env::var("REDIS_URL").ok(),
            redis_pool_size: std::env::var("REDIS_POOL_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_redis() {
        let cfg = CacheConfig::default();
        assert!(cfg.redis_url.is_none());
        assert_eq!(cfg.l1_max_capacity, 10_000);
        assert_eq!(cfg.l1_ttl_secs, 300);
        assert_eq!(cfg.redis_pool_size, 10);
    }

    #[test]
    fn cache_config_l2_disabled_when_no_redis_url() {
        // sin REDIS_URL en el entorno, redis_url es None
        std::env::remove_var("REDIS_URL");
        let cfg = CacheConfig::from_env();
        assert!(cfg.redis_url.is_none());
    }
}
