//! Configuracion del subsistema de tiempo real.

/// Modo de conexion al bus de eventos NATS.
#[derive(Debug, Clone, Default)]
pub enum NatsMode {
    /// Bus en memoria via tokio::sync::broadcast (sin servidor NATS externo).
    #[default]
    InProcess,
    /// Conexion a un servidor NATS externo.
    External,
}

/// Configuracion del subsistema de tiempo real.
#[derive(Debug, Clone)]
pub struct RealtimeConfig {
    /// Modo del bus de eventos. Default: InProcess.
    pub nats_mode: NatsMode,
    /// URL del servidor NATS externo (solo relevante en modo External).
    /// Variable de entorno: `NATS_URL`.
    pub nats_url: String,
    /// Capacidad del canal broadcast interno (solo modo InProcess).
    pub broadcast_capacity: usize,
}

impl Default for RealtimeConfig {
    fn default() -> Self {
        Self {
            nats_mode: NatsMode::InProcess,
            nats_url: "nats://localhost:4222".to_string(),
            broadcast_capacity: 1024,
        }
    }
}

impl RealtimeConfig {
    /// Lee la configuracion desde variables de entorno.
    ///
    /// Variables reconocidas:
    /// - `NATS_MODE`: `"external"` activa el modo External; cualquier otro valor usa InProcess.
    /// - `NATS_URL`: URL del servidor NATS (default: `nats://localhost:4222`).
    /// - `RT_BROADCAST_CAPACITY`: capacidad del canal broadcast (default: 1024).
    pub fn from_env() -> Self {
        let nats_mode = match std::env::var("NATS_MODE").as_deref() {
            Ok("external") => NatsMode::External,
            _ => NatsMode::InProcess,
        };
        Self {
            nats_mode,
            nats_url: std::env::var("NATS_URL")
                .unwrap_or_else(|_| "nats://localhost:4222".to_string()),
            broadcast_capacity: std::env::var("RT_BROADCAST_CAPACITY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1024),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_uses_inprocess_mode() {
        let cfg = RealtimeConfig::default();
        assert!(matches!(cfg.nats_mode, NatsMode::InProcess));
    }

    #[test]
    fn default_capacity_is_1024() {
        let cfg = RealtimeConfig::default();
        assert_eq!(cfg.broadcast_capacity, 1024);
    }

    #[test]
    fn default_nats_url() {
        let cfg = RealtimeConfig::default();
        assert_eq!(cfg.nats_url, "nats://localhost:4222");
    }

    #[test]
    fn from_env_external_mode_and_custom_url() {
        std::env::set_var("NATS_MODE", "external");
        std::env::set_var("NATS_URL", "nats://myserver:4222");
        let cfg = RealtimeConfig::from_env();
        assert!(matches!(cfg.nats_mode, NatsMode::External));
        assert_eq!(cfg.nats_url, "nats://myserver:4222");
        std::env::remove_var("NATS_MODE");
        std::env::remove_var("NATS_URL");
    }

    #[test]
    fn from_env_unknown_mode_defaults_to_inprocess() {
        std::env::set_var("NATS_MODE", "other_value");
        let cfg = RealtimeConfig::from_env();
        assert!(matches!(cfg.nats_mode, NatsMode::InProcess));
        std::env::remove_var("NATS_MODE");
    }

    #[test]
    fn from_env_custom_broadcast_capacity() {
        std::env::set_var("RT_BROADCAST_CAPACITY", "256");
        let cfg = RealtimeConfig::from_env();
        assert_eq!(cfg.broadcast_capacity, 256);
        std::env::remove_var("RT_BROADCAST_CAPACITY");
    }

    #[test]
    fn from_env_invalid_capacity_uses_default() {
        std::env::set_var("RT_BROADCAST_CAPACITY", "not_a_number");
        let cfg = RealtimeConfig::from_env();
        assert_eq!(cfg.broadcast_capacity, 1024);
        std::env::remove_var("RT_BROADCAST_CAPACITY");
    }

    #[test]
    fn from_env_no_vars_uses_defaults() {
        std::env::remove_var("NATS_MODE");
        std::env::remove_var("NATS_URL");
        std::env::remove_var("RT_BROADCAST_CAPACITY");
        let cfg = RealtimeConfig::from_env();
        assert!(matches!(cfg.nats_mode, NatsMode::InProcess));
        assert_eq!(cfg.nats_url, "nats://localhost:4222");
        assert_eq!(cfg.broadcast_capacity, 1024);
    }
}
