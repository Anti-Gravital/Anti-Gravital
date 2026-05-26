//! Public Shield configuration.
//!
//! The configuration is deserialized from a TOML file described in
//! `docs/rfc/RFC-0002-diseno-shield-mvp.md` section 4.5. All sections
//! accept secure defaults: omitting a section in TOML is equivalent to
//! using `Default`. Unknown keys are rejected with `AgError::Config` to
//! avoid silent typos.
//!
//! A documented example with all the sections is in
//! `crates/ag-core/config.example.toml`.

use std::net::SocketAddr;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{AgError, AgResult};

/// Complete Shield configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShieldConfig {
    /// Listen address.
    #[serde(default = "default_bind_addr")]
    pub bind: SocketAddr,

    /// Tokio runtime configuration.
    #[serde(default)]
    pub runtime: RuntimeConfig,

    /// CORS configuration.
    #[serde(default)]
    pub cors: CorsConfig,

    /// CSRF configuration.
    #[serde(default)]
    pub csrf: CsrfConfig,

    /// Per-IP rate limiting configuration.
    #[serde(default)]
    pub rate_limit: RateLimitConfig,

    /// Ed25519 JWT authentication configuration.
    #[serde(default)]
    pub auth: AuthConfig,

    /// TLS 1.3 configuration.
    #[serde(default)]
    pub tls: TlsConfig,
}

impl Default for ShieldConfig {
    fn default() -> Self {
        Self {
            bind: default_bind_addr(),
            runtime: RuntimeConfig::default(),
            cors: CorsConfig::default(),
            csrf: CsrfConfig::default(),
            rate_limit: RateLimitConfig::default(),
            auth: AuthConfig::default(),
            tls: TlsConfig::default(),
        }
    }
}

/// Configuracion TLS 1.3.
///
/// Por defecto deshabilitada para no exigir certificado en
/// desarrollo. Cuando se activa, `cert_path` y `key_path` deben
/// apuntar a archivos PEM con la cadena de certificados y la clave
/// privada respectivamente. Cuando el server vive detras de un
/// balanceador que termina TLS (Cloudflare, AWS ALB, Nginx) la capa
/// se deja deshabilitada.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsConfig {
    /// Activa la capa TLS.
    #[serde(default)]
    pub enabled: bool,

    /// Ruta al archivo PEM con la cadena de certificados.
    #[serde(default)]
    pub cert_path: Option<std::path::PathBuf>,

    /// Ruta al archivo PEM con la clave privada (PKCS#8, RSA o EC).
    #[serde(default)]
    pub key_path: Option<std::path::PathBuf>,
}

/// Configuracion CORS.
///
/// Por defecto la capa esta deshabilitada para no permitir cross-origin
/// implicito. Para habilitarla declare `enabled = true` y al menos un
/// origen.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

/// Configuracion CSRF.
///
/// Por defecto deshabilitada. Cuando se activa, las peticiones que mutan
/// estado deben presentar el header y la cookie configurados con
/// valores identicos (patron double-submit cookie).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CsrfConfig {
    /// Activa la capa CSRF.
    #[serde(default)]
    pub enabled: bool,

    /// Nombre del header que transporta el token. Por defecto
    /// `X-CSRF-Token`. Se compara en minusculas.
    #[serde(default = "default_csrf_header")]
    pub token_header: String,

    /// Nombre de la cookie que transporta el token. Por defecto
    /// `ag_csrf`.
    #[serde(default = "default_csrf_cookie")]
    pub token_cookie: String,
}

impl Default for CsrfConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            token_header: default_csrf_header(),
            token_cookie: default_csrf_cookie(),
        }
    }
}

fn default_csrf_header() -> String {
    "x-csrf-token".to_owned()
}

fn default_csrf_cookie() -> String {
    "ag_csrf".to_owned()
}

/// Configuracion de rate limiting por IP.
///
/// Por defecto deshabilitada. Cuando se activa aplica un token bucket
/// por direccion IP de origen con `per_ip_rps` peticiones por segundo
/// como tasa sostenida y `burst` peticiones como pico instantaneo.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RateLimitConfig {
    /// Activa la capa de rate limiting.
    #[serde(default)]
    pub enabled: bool,

    /// Peticiones por segundo permitidas por IP en regimen sostenido.
    #[serde(default = "default_per_ip_rps")]
    pub per_ip_rps: u32,

    /// Capacidad maxima del token bucket por IP.
    #[serde(default = "default_burst")]
    pub burst: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            per_ip_rps: default_per_ip_rps(),
            burst: default_burst(),
        }
    }
}

const fn default_per_ip_rps() -> u32 {
    100
}

const fn default_burst() -> u32 {
    200
}

/// Configuracion de autenticacion JWT Ed25519.
///
/// Por defecto deshabilitada. Cuando se activa, la pipeline exige un
/// header `Authorization: Bearer <token>` valido en todas las
/// peticiones que la capa Auth cubre. La clave publica se entrega como
/// PEM inline (`public_key_pem`) o como ruta (`public_key_path`).
///
/// Opcionalmente se valida que el claim `iss` coincida con
/// `expected_issuer` y que `aud` contenga `expected_audience`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthConfig {
    /// Activa la capa de autenticacion JWT.
    #[serde(default)]
    pub enabled: bool,

    /// Clave publica Ed25519 en formato PEM. Mutuamente excluyente con
    /// `public_key_path`.
    #[serde(default)]
    pub public_key_pem: Option<String>,

    /// Ruta a un archivo PEM con la clave publica Ed25519. Mutuamente
    /// excluyente con `public_key_pem`.
    #[serde(default)]
    pub public_key_path: Option<std::path::PathBuf>,

    /// Issuer esperado en el claim `iss`. `None` desactiva la
    /// verificacion.
    #[serde(default)]
    pub expected_issuer: Option<String>,

    /// Audience esperado en el claim `aud`. `None` desactiva la
    /// verificacion.
    #[serde(default)]
    pub expected_audience: Option<String>,
}

impl ShieldConfig {
    /// Carga la configuracion desde una cadena TOML.
    ///
    /// # Errores
    ///
    /// Devuelve `AgError::Config` si el TOML no parsea o contiene
    /// claves desconocidas.
    pub fn from_toml_str(toml_text: &str) -> AgResult<Self> {
        toml::from_str(toml_text).map_err(|e| AgError::Config(e.to_string()))
    }

    /// Carga la configuracion desde un archivo TOML en disco.
    ///
    /// # Errores
    ///
    /// Devuelve `AgError::Config` si el archivo no existe, no se puede
    /// leer, o el contenido no es un TOML valido.
    pub fn from_path(path: impl AsRef<Path>) -> AgResult<Self> {
        let path = path.as_ref();
        let bytes = std::fs::read_to_string(path).map_err(|e| {
            AgError::Config(format!("cannot read config at {}: {e}", path.display()))
        })?;
        Self::from_toml_str(&bytes)
    }

    /// Serializa la configuracion a cadena TOML.
    ///
    /// # Errores
    ///
    /// Devuelve `AgError::Config` en el caso extremadamente improbable
    /// de que la serializacion falle (tipos no representables en TOML).
    pub fn to_toml_string(&self) -> AgResult<String> {
        toml::to_string(self).map_err(|e| AgError::Config(e.to_string()))
    }
}

/// Configuracion del runtime Tokio.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
    fn from_toml_rejects_invalid_syntax() {
        let err = ShieldConfig::from_toml_str("not valid toml [").unwrap_err();
        assert_eq!(err.code(), "config_error");
    }

    #[test]
    fn from_toml_rejects_unknown_top_level_key() {
        let err = ShieldConfig::from_toml_str("totally_invented_field = 1").unwrap_err();
        assert_eq!(err.code(), "config_error");
    }

    #[test]
    fn from_toml_rejects_unknown_nested_key() {
        let err = ShieldConfig::from_toml_str(
            r#"
            [cors]
            enabled = true
            unknown_typo = "oops"
            "#,
        )
        .unwrap_err();
        assert_eq!(err.code(), "config_error");
    }

    #[test]
    fn runtime_default_blocking_threads() {
        let rt = RuntimeConfig::default();
        assert_eq!(rt.blocking_threads, 512);
        assert!(rt.workers.is_none());
    }

    #[test]
    fn empty_toml_yields_default_config() {
        let cfg = ShieldConfig::from_toml_str("").unwrap();
        let expected = ShieldConfig::default();
        assert_eq!(cfg.bind, expected.bind);
        assert_eq!(
            cfg.runtime.blocking_threads,
            expected.runtime.blocking_threads
        );
        assert_eq!(cfg.cors.enabled, expected.cors.enabled);
        assert_eq!(cfg.csrf.enabled, expected.csrf.enabled);
        assert_eq!(cfg.rate_limit.enabled, expected.rate_limit.enabled);
        assert_eq!(cfg.auth.enabled, expected.auth.enabled);
        assert_eq!(cfg.tls.enabled, expected.tls.enabled);
    }

    #[test]
    fn from_toml_parses_full_cors_section() {
        let cfg = ShieldConfig::from_toml_str(
            r#"
            [cors]
            enabled = true
            allow_origins = ["https://a.example", "https://b.example"]
            allow_methods = ["GET", "POST"]
            allow_headers = ["content-type"]
            allow_credentials = true
            "#,
        )
        .unwrap();
        assert!(cfg.cors.enabled);
        assert_eq!(cfg.cors.allow_origins.len(), 2);
        assert_eq!(cfg.cors.allow_methods, vec!["GET", "POST"]);
        assert!(cfg.cors.allow_credentials);
    }

    #[test]
    fn from_toml_parses_csrf_with_custom_names() {
        let cfg = ShieldConfig::from_toml_str(
            r#"
            [csrf]
            enabled = true
            token_header = "x-my-csrf"
            token_cookie = "my_csrf_cookie"
            "#,
        )
        .unwrap();
        assert!(cfg.csrf.enabled);
        assert_eq!(cfg.csrf.token_header, "x-my-csrf");
        assert_eq!(cfg.csrf.token_cookie, "my_csrf_cookie");
    }

    #[test]
    fn from_toml_parses_rate_limit_section() {
        let cfg = ShieldConfig::from_toml_str(
            r#"
            [rate_limit]
            enabled = true
            per_ip_rps = 200
            burst = 500
            "#,
        )
        .unwrap();
        assert!(cfg.rate_limit.enabled);
        assert_eq!(cfg.rate_limit.per_ip_rps, 200);
        assert_eq!(cfg.rate_limit.burst, 500);
    }

    #[test]
    fn from_toml_parses_auth_with_inline_pem() {
        let cfg = ShieldConfig::from_toml_str(
            r#"
            [auth]
            enabled = true
            public_key_pem = "PEM CONTENT"
            expected_issuer = "https://issuer.example/"
            "#,
        )
        .unwrap();
        assert!(cfg.auth.enabled);
        assert_eq!(cfg.auth.public_key_pem.as_deref(), Some("PEM CONTENT"));
        assert_eq!(
            cfg.auth.expected_issuer.as_deref(),
            Some("https://issuer.example/")
        );
        assert!(cfg.auth.public_key_path.is_none());
    }

    #[test]
    fn from_toml_parses_tls_section() {
        let cfg = ShieldConfig::from_toml_str(
            r#"
            [tls]
            enabled = true
            cert_path = "/etc/ag/cert.pem"
            key_path = "/etc/ag/key.pem"
            "#,
        )
        .unwrap();
        assert!(cfg.tls.enabled);
        assert_eq!(
            cfg.tls.cert_path.as_deref(),
            Some(std::path::Path::new("/etc/ag/cert.pem"))
        );
    }

    #[test]
    fn round_trip_serialize_deserialize_default() {
        let cfg = ShieldConfig::default();
        let toml_text = cfg.to_toml_string().unwrap();
        let reparsed = ShieldConfig::from_toml_str(&toml_text).unwrap();
        assert_eq!(cfg.bind, reparsed.bind);
        assert_eq!(cfg.cors.enabled, reparsed.cors.enabled);
        assert_eq!(cfg.csrf.token_header, reparsed.csrf.token_header);
        assert_eq!(
            cfg.runtime.blocking_threads,
            reparsed.runtime.blocking_threads
        );
    }

    #[test]
    fn from_path_loads_example_config() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("config.example.toml");
        let cfg = ShieldConfig::from_path(&path).expect("config.example.toml must parse cleanly");
        // El ejemplo trae defaults seguros: todas las capas deshabilitadas.
        assert!(!cfg.cors.enabled);
        assert!(!cfg.csrf.enabled);
        assert!(!cfg.rate_limit.enabled);
        assert!(!cfg.auth.enabled);
        assert!(!cfg.tls.enabled);
    }

    #[test]
    fn from_path_returns_config_error_when_missing() {
        let err = ShieldConfig::from_path("/does/not/exist/anti-gravital.toml").unwrap_err();
        assert_eq!(err.code(), "config_error");
    }
}
