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

/// TLS 1.3 configuration.
///
/// Disabled by default so as not to require a certificate in
/// development. When enabled, `cert_path` and `key_path` must point to
/// PEM files with the certificate chain and the private key
/// respectively. When the server lives behind a load balancer that
/// terminates TLS (Cloudflare, AWS ALB, Nginx) the layer is left
/// disabled.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsConfig {
    /// Enables the TLS layer.
    #[serde(default)]
    pub enabled: bool,

    /// Path to the PEM file with the certificate chain.
    #[serde(default)]
    pub cert_path: Option<std::path::PathBuf>,

    /// Path to the PEM file with the private key (PKCS#8, RSA or EC).
    #[serde(default)]
    pub key_path: Option<std::path::PathBuf>,
}

/// CORS configuration.
///
/// By default the layer is disabled so as not to allow implicit
/// cross-origin requests. To enable it declare `enabled = true` and at
/// least one origin.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorsConfig {
    /// Enables the CORS layer.
    #[serde(default)]
    pub enabled: bool,

    /// Allowed origins. Example: `["https://app.example.com"]`.
    #[serde(default)]
    pub allow_origins: Vec<String>,

    /// Allowed HTTP methods. Example: `["GET", "POST"]`.
    #[serde(default)]
    pub allow_methods: Vec<String>,

    /// Allowed headers. Example: `["content-type", "authorization"]`.
    #[serde(default)]
    pub allow_headers: Vec<String>,

    /// Whether credentials are allowed on cross-origin requests.
    #[serde(default)]
    pub allow_credentials: bool,
}

/// CSRF configuration.
///
/// Disabled by default. When enabled, state-mutating requests must
/// present the configured header and cookie with identical values
/// (double-submit cookie pattern).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CsrfConfig {
    /// Enables the CSRF layer.
    #[serde(default)]
    pub enabled: bool,

    /// Name of the header that carries the token. Defaults to
    /// `X-CSRF-Token`. Compared in lowercase.
    #[serde(default = "default_csrf_header")]
    pub token_header: String,

    /// Name of the cookie that carries the token. Defaults to
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

/// Per-IP rate limiting configuration.
///
/// Disabled by default. When enabled it applies a token bucket per
/// source IP address with `per_ip_rps` requests per second as the
/// sustained rate and `burst` requests as the instantaneous peak.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RateLimitConfig {
    /// Enables the rate limiting layer.
    #[serde(default)]
    pub enabled: bool,

    /// Requests per second allowed per IP under sustained load.
    #[serde(default = "default_per_ip_rps")]
    pub per_ip_rps: u32,

    /// Maximum capacity of the per-IP token bucket.
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

/// Ed25519 JWT authentication configuration.
///
/// Disabled by default. When enabled, the pipeline requires a valid
/// `Authorization: Bearer <token>` header on every request the Auth
/// layer covers. The public key is provided as inline PEM
/// (`public_key_pem`) or as a path (`public_key_path`).
///
/// Optionally it validates that the `iss` claim matches
/// `expected_issuer` and that `aud` contains `expected_audience`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthConfig {
    /// Enables the JWT authentication layer.
    #[serde(default)]
    pub enabled: bool,

    /// Ed25519 public key in PEM format. Mutually exclusive with
    /// `public_key_path`.
    #[serde(default)]
    pub public_key_pem: Option<String>,

    /// Path to a PEM file with the Ed25519 public key. Mutually
    /// exclusive with `public_key_pem`.
    #[serde(default)]
    pub public_key_path: Option<std::path::PathBuf>,

    /// Issuer expected in the `iss` claim. `None` disables the
    /// verification.
    #[serde(default)]
    pub expected_issuer: Option<String>,

    /// Audience expected in the `aud` claim. `None` disables the
    /// verification.
    #[serde(default)]
    pub expected_audience: Option<String>,
}

impl ShieldConfig {
    /// Loads the configuration from a TOML string.
    ///
    /// # Errors
    ///
    /// Returns `AgError::Config` if the TOML does not parse or contains
    /// unknown keys.
    pub fn from_toml_str(toml_text: &str) -> AgResult<Self> {
        toml::from_str(toml_text).map_err(|e| AgError::Config(e.to_string()))
    }

    /// Loads the configuration from a TOML file on disk.
    ///
    /// # Errors
    ///
    /// Returns `AgError::Config` if the file does not exist, cannot be
    /// read, or the contents are not valid TOML.
    pub fn from_path(path: impl AsRef<Path>) -> AgResult<Self> {
        let path = path.as_ref();
        let bytes = std::fs::read_to_string(path).map_err(|e| {
            AgError::Config(format!("cannot read config at {}: {e}", path.display()))
        })?;
        Self::from_toml_str(&bytes)
    }

    /// Serializes the configuration to a TOML string.
    ///
    /// # Errors
    ///
    /// Returns `AgError::Config` in the extremely unlikely case that
    /// serialization fails (types not representable in TOML).
    pub fn to_toml_string(&self) -> AgResult<String> {
        toml::to_string(self).map_err(|e| AgError::Config(e.to_string()))
    }
}

/// Tokio runtime configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfig {
    /// Number of workers. `None` means one per available CPU.
    #[serde(default)]
    pub workers: Option<usize>,

    /// Maximum number of blocking threads.
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
        // The example ships secure defaults: all layers disabled.
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
