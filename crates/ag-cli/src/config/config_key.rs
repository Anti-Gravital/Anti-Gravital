//! Configuration keys and value types exposed by `ag config`.
//!
//! Moved from `config.rs` without changing any validation rule or value kind.

use std::net::SocketAddr;
use std::str::FromStr;

/// A leaf configuration key exposed by `ag config`.
///
/// The canonical spelling is the dotted TOML path understood by
/// `ag_core::ShieldConfig`. Secrets (for example an inline JWT public key PEM)
/// are deliberately not listed: the form and the flags only accept file paths
/// or environment-variable names for secret material (RFC-0018 section 4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfigKey {
    Bind,
    RuntimeWorkers,
    RuntimeBlockingThreads,
    CorsEnabled,
    CorsAllowOrigins,
    CorsAllowMethods,
    CorsAllowHeaders,
    CorsAllowCredentials,
    CsrfEnabled,
    CsrfTokenHeader,
    CsrfTokenCookie,
    RateLimitEnabled,
    RateLimitPerIpRps,
    RateLimitBurst,
    AuthEnabled,
    AuthPublicKeyPath,
    AuthExpectedIssuer,
    AuthExpectedAudience,
    TlsEnabled,
    TlsCertPath,
    TlsKeyPath,
}

impl ConfigKey {
    /// Every key, in the order shown by the form.
    pub(crate) const ALL: [ConfigKey; 21] = [
        ConfigKey::Bind,
        ConfigKey::RuntimeWorkers,
        ConfigKey::RuntimeBlockingThreads,
        ConfigKey::CorsEnabled,
        ConfigKey::CorsAllowOrigins,
        ConfigKey::CorsAllowMethods,
        ConfigKey::CorsAllowHeaders,
        ConfigKey::CorsAllowCredentials,
        ConfigKey::CsrfEnabled,
        ConfigKey::CsrfTokenHeader,
        ConfigKey::CsrfTokenCookie,
        ConfigKey::RateLimitEnabled,
        ConfigKey::RateLimitPerIpRps,
        ConfigKey::RateLimitBurst,
        ConfigKey::AuthEnabled,
        ConfigKey::AuthPublicKeyPath,
        ConfigKey::AuthExpectedIssuer,
        ConfigKey::AuthExpectedAudience,
        ConfigKey::TlsEnabled,
        ConfigKey::TlsCertPath,
        ConfigKey::TlsKeyPath,
    ];

    /// Canonical dotted TOML path.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ConfigKey::Bind => "bind",
            ConfigKey::RuntimeWorkers => "runtime.workers",
            ConfigKey::RuntimeBlockingThreads => "runtime.blocking_threads",
            ConfigKey::CorsEnabled => "cors.enabled",
            ConfigKey::CorsAllowOrigins => "cors.allow_origins",
            ConfigKey::CorsAllowMethods => "cors.allow_methods",
            ConfigKey::CorsAllowHeaders => "cors.allow_headers",
            ConfigKey::CorsAllowCredentials => "cors.allow_credentials",
            ConfigKey::CsrfEnabled => "csrf.enabled",
            ConfigKey::CsrfTokenHeader => "csrf.token_header",
            ConfigKey::CsrfTokenCookie => "csrf.token_cookie",
            ConfigKey::RateLimitEnabled => "rate_limit.enabled",
            ConfigKey::RateLimitPerIpRps => "rate_limit.per_ip_rps",
            ConfigKey::RateLimitBurst => "rate_limit.burst",
            ConfigKey::AuthEnabled => "auth.enabled",
            ConfigKey::AuthPublicKeyPath => "auth.public_key_path",
            ConfigKey::AuthExpectedIssuer => "auth.expected_issuer",
            ConfigKey::AuthExpectedAudience => "auth.expected_audience",
            ConfigKey::TlsEnabled => "tls.enabled",
            ConfigKey::TlsCertPath => "tls.cert_path",
            ConfigKey::TlsKeyPath => "tls.key_path",
        }
    }

    /// Parses a `--key` value. Dotted paths are the documented spelling.
    pub(crate) fn parse(value: &str) -> Option<ConfigKey> {
        let normalized = value.trim().replace('-', "_");
        ConfigKey::ALL
            .into_iter()
            .find(|key| key.as_str() == normalized)
    }

    /// Section heading used to group fields in the form.
    #[cfg(feature = "tui")]
    pub(crate) fn section(self) -> &'static str {
        match self {
            ConfigKey::Bind => "Bind address",
            ConfigKey::RuntimeWorkers | ConfigKey::RuntimeBlockingThreads => "Runtime",
            ConfigKey::CorsEnabled
            | ConfigKey::CorsAllowOrigins
            | ConfigKey::CorsAllowMethods
            | ConfigKey::CorsAllowHeaders
            | ConfigKey::CorsAllowCredentials => "CORS",
            ConfigKey::CsrfEnabled | ConfigKey::CsrfTokenHeader | ConfigKey::CsrfTokenCookie => {
                "CSRF"
            }
            ConfigKey::RateLimitEnabled
            | ConfigKey::RateLimitPerIpRps
            | ConfigKey::RateLimitBurst => "Rate limiting",
            ConfigKey::AuthEnabled
            | ConfigKey::AuthPublicKeyPath
            | ConfigKey::AuthExpectedIssuer
            | ConfigKey::AuthExpectedAudience => "Authentication",
            ConfigKey::TlsEnabled | ConfigKey::TlsCertPath | ConfigKey::TlsKeyPath => "TLS",
        }
    }

    /// One-line description shown next to the field in the form.
    #[cfg(feature = "tui")]
    pub(crate) fn description(self) -> &'static str {
        match self {
            ConfigKey::Bind => "Listener address as host:port.",
            ConfigKey::RuntimeWorkers => "Tokio worker threads (omit for one per CPU).",
            ConfigKey::RuntimeBlockingThreads => "Maximum blocking threads.",
            ConfigKey::CorsEnabled => "Enable cross-origin requests.",
            ConfigKey::CorsAllowOrigins => "Allowed origins, comma separated.",
            ConfigKey::CorsAllowMethods => "Allowed HTTP methods, comma separated.",
            ConfigKey::CorsAllowHeaders => "Allowed request headers, comma separated.",
            ConfigKey::CorsAllowCredentials => "Allow credentials in cross-origin requests.",
            ConfigKey::CsrfEnabled => "Enable stateless double-submit CSRF protection.",
            ConfigKey::CsrfTokenHeader => "Header that carries the CSRF token.",
            ConfigKey::CsrfTokenCookie => "Cookie that carries the CSRF token.",
            ConfigKey::RateLimitEnabled => "Enable per-IP rate limiting.",
            ConfigKey::RateLimitPerIpRps => "Sustained requests per second per IP.",
            ConfigKey::RateLimitBurst => "Token-bucket capacity per IP.",
            ConfigKey::AuthEnabled => "Require a valid bearer token.",
            ConfigKey::AuthPublicKeyPath => {
                "Path to the Ed25519 public key PEM (no secrets in the form)."
            }
            ConfigKey::AuthExpectedIssuer => "Expected `iss` claim.",
            ConfigKey::AuthExpectedAudience => "Expected `aud` claim.",
            ConfigKey::TlsEnabled => "Enable TLS 1.3 termination.",
            ConfigKey::TlsCertPath => "Path to the certificate chain PEM.",
            ConfigKey::TlsKeyPath => "Path to the private key PEM.",
        }
    }

    /// Textual value type of this key, used for inline validation in the form.
    pub(crate) fn kind(self) -> ValueKind {
        match self {
            ConfigKey::Bind => ValueKind::Bind,
            ConfigKey::RuntimeWorkers
            | ConfigKey::RuntimeBlockingThreads
            | ConfigKey::RateLimitPerIpRps
            | ConfigKey::RateLimitBurst => ValueKind::Integer,
            ConfigKey::CorsEnabled
            | ConfigKey::CorsAllowCredentials
            | ConfigKey::CsrfEnabled
            | ConfigKey::RateLimitEnabled
            | ConfigKey::AuthEnabled
            | ConfigKey::TlsEnabled => ValueKind::Bool,
            ConfigKey::CorsAllowOrigins
            | ConfigKey::CorsAllowMethods
            | ConfigKey::CorsAllowHeaders => ValueKind::StringList,
            ConfigKey::CsrfTokenHeader
            | ConfigKey::CsrfTokenCookie
            | ConfigKey::AuthPublicKeyPath
            | ConfigKey::AuthExpectedIssuer
            | ConfigKey::AuthExpectedAudience
            | ConfigKey::TlsCertPath
            | ConfigKey::TlsKeyPath => ValueKind::String,
        }
    }

    /// `(table, key)` in the TOML document; `None` table means a root key.
    pub(crate) fn location(self) -> (Option<&'static str>, &'static str) {
        match self.as_str().split_once('.') {
            Some((table, key)) => (Some(table), key),
            None => (None, self.as_str()),
        }
    }
}

/// The type of a configuration value, used for inline and pre-write validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueKind {
    Bool,
    Integer,
    String,
    StringList,
    Bind,
}

impl ValueKind {
    /// Validates the textual form of a value. Returns a user-readable problem.
    pub(crate) fn validate(self, value: &str) -> Result<(), String> {
        match self {
            ValueKind::Bool => value
                .parse::<bool>()
                .map(|_| ())
                .map_err(|_| "expected true or false".to_owned()),
            ValueKind::Integer => {
                // TOML integers are signed 64-bit. Parsing as u64 here let
                // values above i64::MAX pass validation and then silently
                // become 1 in build_value (issue #188).
                let parsed = value.parse::<i64>().map_err(|_| {
                    "expected a positive integer up to 9223372036854775807".to_owned()
                })?;
                if parsed <= 0 {
                    Err("expected a positive integer".to_owned())
                } else {
                    Ok(())
                }
            }
            ValueKind::String => Ok(()),
            ValueKind::StringList => {
                if value.trim().is_empty() {
                    return Ok(());
                }
                if value.split(',').any(|part| part.trim().is_empty()) {
                    return Err("use a comma separated list without empty items".to_owned());
                }
                Ok(())
            }
            ValueKind::Bind => SocketAddr::from_str(value)
                .map(|_| ())
                .map_err(|error| format!("expected host:port ({error})")),
        }
    }
}

/// Validates one key/value pair with a message prefixed by the key name.
#[cfg(feature = "tui")]
pub(crate) fn validate_change(key: ConfigKey, value: &str) -> Result<(), String> {
    key.kind()
        .validate(value)
        .map_err(|problem| format!("{}: {problem}", key.as_str()))
}
