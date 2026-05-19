//! Configuracion publica del Shield.
//!
//! La configuracion se deserializa desde un archivo TOML descrito en
//! `docs/rfc/RFC-0002-diseno-shield-mvp.md` seccion 4.5. Esta version
//! cubre los campos minimos del bootstrap; el resto de campos llegan
//! con sus respectivas capas.

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::error::{AgError, AgResult};

/// Configuracion completa del Shield.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldConfig {
    /// Direccion de escucha.
    #[serde(default = "default_bind_addr")]
    pub bind: SocketAddr,

    /// Configuracion del runtime Tokio.
    #[serde(default)]
    pub runtime: RuntimeConfig,

    /// Configuracion CORS.
    #[serde(default)]
    pub cors: CorsConfig,
}

impl Default for ShieldConfig {
    fn default() -> Self {
        Self {
            bind: default_bind_addr(),
            runtime: RuntimeConfig::default(),
            cors: CorsConfig::default(),
        }
    }
}

/// Configuracion CORS.
///
/// Por defecto la capa esta deshabilitada para no permitir cross-origin
/// implicito. Para habilitarla declare `enabled = true` y al menos un
/// origen.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Activa la capa CORS.
    #[serde(default)]
    pub enabled: bool,

    /// Origenes permitidos. Ej: `["https://app.example.com"]`.
    #[serde(default)]
    pub allow_origins: Vec<String>,

    /// Metodos HTTP permitidos. Ej: `["GET", "POST"]`.
    #[serde(default)]
    pub allow_methods: Vec<String>,

    /// Headers permitidos. Ej: `["content-type", "authorization"]`.
    #[serde(default)]
    pub allow_headers: Vec<String>,

    /// Si se permiten credenciales en peticiones cross-origin.
    #[serde(default)]
    pub allow_credentials: bool,
}

impl ShieldConfig {
    /// Carga la configuracion desde una cadena TOML.
    ///
    /// # Errores
    ///
    /// Devuelve `AgError::Config` si el TOML no parsea o contiene claves
    /// desconocidas.
    pub fn from_toml_str(toml_text: &str) -> AgResult<Self> {
        toml::from_str(toml_text).map_err(|e| AgError::Config(e.to_string()))
    }
}

/// Configuracion del runtime Tokio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Numero de workers. `None` significa uno por CPU disponible.
    #[serde(default)]
    pub workers: Option<usize>,

    /// Maximo de threads bloqueantes.
    #[serde(default = "default_blocking_threads")]
    pub blocking_threads: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            workers: None,
            blocking_threads: default_blocking_threads(),
        }
    }
}

fn default_bind_addr() -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], 8080))
}

const fn default_blocking_threads() -> usize {
    512
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bind_is_0_0_0_0_8080() {
        let config = ShieldConfig::default();
        assert_eq!(config.bind.port(), 8080);
        assert!(config.bind.ip().is_unspecified());
    }

    #[test]
    fn from_toml_parses_minimal() {
        let cfg = ShieldConfig::from_toml_str(r#"bind = "127.0.0.1:9090""#).unwrap();
        assert_eq!(cfg.bind.port(), 9090);
    }

    #[test]
    fn from_toml_rejects_invalid() {
        let err = ShieldConfig::from_toml_str("not valid toml [").unwrap_err();
        assert_eq!(err.code(), "config_error");
    }

    #[test]
    fn runtime_default_blocking_threads() {
        let rt = RuntimeConfig::default();
        assert_eq!(rt.blocking_threads, 512);
        assert!(rt.workers.is_none());
    }
}
