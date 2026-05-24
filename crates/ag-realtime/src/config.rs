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
    // TLS
    /// Si `true`, activa TLS con CA del sistema.
    /// Variable de entorno: `NATS_TLS`.
    pub nats_tls: bool,
    /// Ruta a CA personalizada (nivel 2: CA custom).
    /// Variable de entorno: `NATS_TLS_CA`.
    pub nats_tls_ca_path: Option<String>,
    /// Ruta al certificado de cliente (nivel 3: mTLS).
    /// Variable de entorno: `NATS_TLS_CERT`.
    pub nats_tls_cert_path: Option<String>,
    /// Ruta a la clave privada del cliente (nivel 3: mTLS).
    /// Variable de entorno: `NATS_TLS_KEY`.
    pub nats_tls_key_path: Option<String>,
    // JetStream
    /// Si `true`, publica y consume via JetStream con ACK.
    /// Variable de entorno: `NATS_JETSTREAM`.
    pub jetstream_enabled: bool,
    /// Nombre del stream JetStream. Default: `AG_EVENTS`.
    /// Variable de entorno: `NATS_JS_STREAM`.
    pub jetstream_stream_name: String,
    /// Limite de mensajes del stream. Default: 1_000_000.
    /// Variable de entorno: `NATS_JS_MAX_MSGS`.
    pub jetstream_max_msgs: i64,
    /// Limite de bytes del stream. Default: 1 GiB.
    /// Variable de entorno: `NATS_JS_MAX_BYTES`.
    pub jetstream_max_bytes: i64,
}

impl Default for RealtimeConfig {
    fn default() -> Self {
        Self {
            nats_mode: NatsMode::InProcess,
            nats_url: "nats://localhost:4222".to_string(),
            broadcast_capacity: 1024,
            nats_tls: false,
            nats_tls_ca_path: None,
            nats_tls_cert_path: None,
            nats_tls_key_path: None,
            jetstream_enabled: false,
            jetstream_stream_name: "AG_EVENTS".to_string(),
            jetstream_max_msgs: 1_000_000,
            jetstream_max_bytes: 1_073_741_824,
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
    /// - `NATS_TLS`: `"true"` activa TLS con CA del sistema.
    /// - `NATS_TLS_CA`: ruta a CA personalizada.
    /// - `NATS_TLS_CERT`: ruta al certificado del cliente (mTLS).
    /// - `NATS_TLS_KEY`: ruta a la clave privada del cliente (mTLS).
    /// - `NATS_JETSTREAM`: `"true"` activa JetStream con ACK.
    /// - `NATS_JS_STREAM`: nombre del stream JetStream (default: `AG_EVENTS`).
    /// - `NATS_JS_MAX_MSGS`: limite de mensajes (default: 1_000_000).
    /// - `NATS_JS_MAX_BYTES`: limite de bytes (default: 1_073_741_824).
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
            nats_tls: std::env::var("NATS_TLS")
                .map(|v| v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            nats_tls_ca_path: std::env::var("NATS_TLS_CA").ok(),
            nats_tls_cert_path: std::env::var("NATS_TLS_CERT").ok(),
            nats_tls_key_path: std::env::var("NATS_TLS_KEY").ok(),
            jetstream_enabled: std::env::var("NATS_JETSTREAM")
                .map(|v| v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            jetstream_stream_name: std::env::var("NATS_JS_STREAM")
                .unwrap_or_else(|_| "AG_EVENTS".to_string()),
            jetstream_max_msgs: std::env::var("NATS_JS_MAX_MSGS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1_000_000),
            jetstream_max_bytes: std::env::var("NATS_JS_MAX_BYTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1_073_741_824),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Serializa acceso a variables de entorno entre tests paralelos.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

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
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("NATS_MODE", "external");
        std::env::set_var("NATS_URL", "nats://myserver:4222");
        let cfg = RealtimeConfig::from_env();
        std::env::remove_var("NATS_MODE");
        std::env::remove_var("NATS_URL");
        assert!(matches!(cfg.nats_mode, NatsMode::External));
        assert_eq!(cfg.nats_url, "nats://myserver:4222");
    }

    #[test]
    fn from_env_unknown_mode_defaults_to_inprocess() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("NATS_MODE", "other_value");
        let cfg = RealtimeConfig::from_env();
        std::env::remove_var("NATS_MODE");
        assert!(matches!(cfg.nats_mode, NatsMode::InProcess));
    }

    #[test]
    fn from_env_custom_broadcast_capacity() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("RT_BROADCAST_CAPACITY", "256");
        let cfg = RealtimeConfig::from_env();
        std::env::remove_var("RT_BROADCAST_CAPACITY");
        assert_eq!(cfg.broadcast_capacity, 256);
    }

    #[test]
    fn from_env_invalid_capacity_uses_default() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("RT_BROADCAST_CAPACITY", "not_a_number");
        let cfg = RealtimeConfig::from_env();
        std::env::remove_var("RT_BROADCAST_CAPACITY");
        assert_eq!(cfg.broadcast_capacity, 1024);
    }

    #[test]
    fn from_env_no_vars_uses_defaults() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("NATS_MODE");
        std::env::remove_var("NATS_URL");
        std::env::remove_var("RT_BROADCAST_CAPACITY");
        let cfg = RealtimeConfig::from_env();
        assert!(matches!(cfg.nats_mode, NatsMode::InProcess));
        assert_eq!(cfg.nats_url, "nats://localhost:4222");
        assert_eq!(cfg.broadcast_capacity, 1024);
    }

    #[test]
    fn from_env_external_mode() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("NATS_MODE", "external");
        let cfg = RealtimeConfig::from_env();
        std::env::remove_var("NATS_MODE");
        assert!(matches!(cfg.nats_mode, NatsMode::External));
    }

    #[test]
    fn from_env_custom_url() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("NATS_URL", "nats://my-nats:4222");
        let cfg = RealtimeConfig::from_env();
        std::env::remove_var("NATS_URL");
        assert_eq!(cfg.nats_url, "nats://my-nats:4222");
    }

    #[test]
    fn from_env_custom_capacity() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("RT_BROADCAST_CAPACITY", "2048");
        let cfg = RealtimeConfig::from_env();
        std::env::remove_var("RT_BROADCAST_CAPACITY");
        assert_eq!(cfg.broadcast_capacity, 2048);
    }

    #[test]
    fn from_env_tls_enabled() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("NATS_TLS", "true");
        let cfg = RealtimeConfig::from_env();
        std::env::remove_var("NATS_TLS");
        assert!(cfg.nats_tls);
    }

    #[test]
    fn from_env_jetstream_stream_name() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("NATS_JS_STREAM", "MY_STREAM");
        let cfg = RealtimeConfig::from_env();
        std::env::remove_var("NATS_JS_STREAM");
        assert_eq!(cfg.jetstream_stream_name, "MY_STREAM");
    }

    #[test]
    fn defaults_tls_off_jetstream_off() {
        let cfg = RealtimeConfig::default();
        assert!(!cfg.nats_tls);
        assert!(!cfg.jetstream_enabled);
        assert_eq!(cfg.jetstream_stream_name, "AG_EVENTS");
    }
}
