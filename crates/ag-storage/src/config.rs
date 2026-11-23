//! Storage store configuration.

use crate::StorageError;
use ag_core::{EnvDeclaration, EnvLoader, EnvType, LoadedEnv};
use std::{net::IpAddr, path::PathBuf};

/// Active storage backend.
#[derive(Debug, Clone, PartialEq)]
pub enum StorageBackend {
    /// Local filesystem (default). No external dependencies.
    Native,
    #[cfg(feature = "s3")]
    /// AWS S3 or compatible.
    S3,
    #[cfg(feature = "s3")]
    /// Self-hosted MinIO (S3-compatible).
    MinIO,
}

/// Storage subsystem configuration.
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Active backend.
    pub backend: StorageBackend,
    /// Root directory of the native store.
    pub root_path: PathBuf,
    /// If `true`, starts an Axum HTTP server in the background.
    pub server_mode: bool,
    /// HTTP server bind address. Default: loopback only.
    pub server_bind: IpAddr,
    /// HTTP server port. Default: 4280.
    pub server_port: u16,
    /// Allows an unauthenticated non-loopback bind when explicitly enabled.
    pub allow_insecure_public: bool,
    /// Static Bearer token. Empty = no authentication (loopback development only).
    pub store_token: String,
    /// PEM Ed25519 public key for JWT Bearer validation (feature `auth`).
    ///
    /// When set and the `auth` feature is enabled, the server validates the
    /// Bearer as a JWT (Ed25519) via `ag-auth` instead of comparing the static
    /// `store_token`. `None` keeps static-token mode (the default).
    /// Variables: `STORAGE_JWT_PUBLIC_KEY` (inline PEM) or
    /// `STORAGE_JWT_PUBLIC_KEY_PATH` (file).
    pub jwt_public_key_pem: Option<String>,
    /// Maximum object size in MB. Default: 100.
    pub max_object_size_mb: u64,
    /// HTTP server request rate limit per second. Default: 100.
    pub rate_limit_rps: u32,
    /// AWS region. Default: "us-east-1".
    pub region: String,
    /// Custom endpoint (for MinIO). None = AWS.
    pub endpoint: Option<String>,
    /// AWS access key.
    pub access_key: Option<String>,
    /// AWS secret key.
    pub secret_key: Option<String>,
    /// S3/MinIO bucket name. Default: "ag-storage".
    pub bucket: String,
    /// HMAC secret key for signing URLs. Empty = signed URLs disabled.
    /// Variable: `STORAGE_SIGN_SECRET`.
    pub sign_secret: String,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::Native,
            root_path: PathBuf::from("./ag-store-data"),
            server_mode: false,
            server_bind: IpAddr::from([127, 0, 0, 1]),
            server_port: 4280,
            allow_insecure_public: false,
            store_token: String::new(),
            jwt_public_key_pem: None,
            max_object_size_mb: 100,
            rate_limit_rps: 100,
            region: "us-east-1".into(),
            endpoint: None,
            access_key: None,
            secret_key: None,
            bucket: "ag-storage".into(),
            sign_secret: String::new(),
        }
    }
}

/// Declared environment schema for `ag-storage` (RFC-0020 PR 4, audit A2).
///
/// Every key the crate reads is listed once here: name, secret classification
/// and, where the raw variable has one, its default. This is the single
/// auditable answer to "which variables does this service need".
///
/// All keys are declared as [`EnvType::String`] on purpose. The previous
/// `from_env` used `var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(DEFAULT)`
/// *per field*: an unset or malformed value fell back independently of every
/// other field. [`EnvLoader`] validates the whole set at once and rejects the
/// load if a single value fails its declared type, so a typed declaration
/// (`Int`/`Bool`/...) would silently change that per-field fallback (for
/// example `STORAGE_PORT=abc` plus a valid `STORAGE_ROOT` would lose the root).
/// Parsing therefore stays in [`StorageConfig::from_loaded_env`], mirroring the
/// original expressions byte for byte; the loader still provides the declared
/// list, per-field defaults, secret redaction and process-environment
/// isolation (only declared keys are read).
pub(crate) fn storage_env_declarations() -> Vec<EnvDeclaration> {
    vec![
        EnvDeclaration::new("STORAGE_BACKEND", EnvType::String).required(false),
        EnvDeclaration::new("STORAGE_ROOT", EnvType::String)
            .required(false)
            .with_default("./ag-store-data"),
        EnvDeclaration::new("STORAGE_SERVER", EnvType::String).required(false),
        EnvDeclaration::new("STORAGE_BIND", EnvType::String).required(false),
        EnvDeclaration::new("STORAGE_PORT", EnvType::String).required(false),
        EnvDeclaration::new("STORAGE_ALLOW_INSECURE_PUBLIC", EnvType::String).required(false),
        EnvDeclaration::new("STORE_TOKEN", EnvType::String)
            .required(false)
            .secret(true)
            .with_default(""),
        EnvDeclaration::new("STORAGE_JWT_PUBLIC_KEY", EnvType::String)
            .required(false)
            .secret(true),
        EnvDeclaration::new("STORAGE_JWT_PUBLIC_KEY_PATH", EnvType::String).required(false),
        EnvDeclaration::new("STORAGE_MAX_OBJECT_SIZE_MB", EnvType::String).required(false),
        EnvDeclaration::new("STORAGE_RATE_LIMIT_RPS", EnvType::String).required(false),
        EnvDeclaration::new("STORAGE_REGION", EnvType::String)
            .required(false)
            .with_default("us-east-1"),
        EnvDeclaration::new("STORAGE_ENDPOINT", EnvType::String).required(false),
        EnvDeclaration::new("AWS_ACCESS_KEY_ID", EnvType::String).required(false),
        EnvDeclaration::new("AWS_SECRET_ACCESS_KEY", EnvType::String)
            .required(false)
            .secret(true),
        EnvDeclaration::new("STORAGE_BUCKET", EnvType::String)
            .required(false)
            .with_default("ag-storage"),
        EnvDeclaration::new("STORAGE_SIGN_SECRET", EnvType::String)
            .required(false)
            .secret(true)
            .with_default(""),
    ]
}

pub(crate) fn storage_loader() -> EnvLoader {
    EnvLoader::new(storage_env_declarations())
}

impl StorageConfig {
    /// Reads the configuration from environment variables.
    /// Undefined values use the defaults from [`StorageConfig::default`].
    ///
    /// Production path: resolves the declared schema through
    /// [`EnvLoader::load_process`], which reads only declared keys and never
    /// prints a value. A non-UTF-8 process value is the only possible load
    /// error for this all-string schema; it is handled below by replaying the
    /// declared keys leniently so that field keeps its previous fallback.
    pub fn from_env() -> Self {
        let loader = storage_loader();
        let env = match loader.load_process() {
            Ok(env) => env,
            Err(_) => {
                let vars: Vec<(String, String)> = loader
                    .declarations()
                    .iter()
                    .filter_map(|declaration| {
                        std::env::var(declaration.name())
                            .ok()
                            .map(|value| (declaration.name().to_owned(), value))
                    })
                    .collect();
                loader
                    .load_from(vars)
                    .expect("all declarations are optional strings and always resolve")
            }
        };
        Self::from_loaded_env(&env)
    }

    /// Test seam: resolves the same declared schema from an injected in-memory
    /// source instead of the process environment. Using this in tests removes
    /// the `set_var`/`remove_var` global races (audit A1) without a global
    /// mutex. Scoped to tests so the public API is unchanged.
    #[cfg(test)]
    pub(crate) fn from_env_source<I, K, V>(vars: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let env = storage_loader()
            .load_from(vars)
            .expect("all declarations are optional strings and always resolve");
        Self::from_loaded_env(&env)
    }

    /// Builds a config from a resolved environment. Parsing and fallbacks are
    /// identical to the pre-migration direct reads, field by field.
    fn from_loaded_env(env: &LoadedEnv) -> Self {
        let backend = match env
            .string("STORAGE_BACKEND")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "s3" => {
                #[cfg(feature = "s3")]
                {
                    StorageBackend::S3
                }
                #[cfg(not(feature = "s3"))]
                {
                    tracing::warn!(
                        "STORAGE_BACKEND=s3 pero feature s3 no esta activa; usando Native"
                    );
                    StorageBackend::Native
                }
            }
            "minio" => {
                #[cfg(feature = "s3")]
                {
                    StorageBackend::MinIO
                }
                #[cfg(not(feature = "s3"))]
                {
                    tracing::warn!(
                        "STORAGE_BACKEND=minio pero feature s3 no esta activa; usando Native"
                    );
                    StorageBackend::Native
                }
            }
            _ => StorageBackend::Native,
        };

        Self {
            backend,
            root_path: env
                .string("STORAGE_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("./ag-store-data")),
            server_mode: env
                .string("STORAGE_SERVER")
                .map(|v| v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            server_bind: env
                .string("STORAGE_BIND")
                .and_then(|value| value.parse().ok())
                .unwrap_or_else(|| IpAddr::from([127, 0, 0, 1])),
            server_port: env
                .string("STORAGE_PORT")
                .and_then(|v| v.parse().ok())
                .unwrap_or(4280),
            allow_insecure_public: env
                .string("STORAGE_ALLOW_INSECURE_PUBLIC")
                .map(|value| value.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            store_token: env
                .secret("STORE_TOKEN")
                .map(|v| v.expose().to_owned())
                .unwrap_or_default(),
            jwt_public_key_pem: env
                .secret("STORAGE_JWT_PUBLIC_KEY")
                .map(|v| v.expose().to_owned())
                .or_else(|| {
                    env.string("STORAGE_JWT_PUBLIC_KEY_PATH")
                        .and_then(|path| std::fs::read_to_string(path).ok())
                }),
            max_object_size_mb: env
                .string("STORAGE_MAX_OBJECT_SIZE_MB")
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            rate_limit_rps: env
                .string("STORAGE_RATE_LIMIT_RPS")
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            region: env
                .string("STORAGE_REGION")
                .map(str::to_owned)
                .unwrap_or_else(|| "us-east-1".into()),
            endpoint: env.string("STORAGE_ENDPOINT").map(str::to_owned),
            access_key: env.string("AWS_ACCESS_KEY_ID").map(str::to_owned),
            secret_key: env
                .secret("AWS_SECRET_ACCESS_KEY")
                .map(|v| v.expose().to_owned()),
            bucket: env
                .string("STORAGE_BUCKET")
                .map(str::to_owned)
                .unwrap_or_else(|| "ag-storage".into()),
            sign_secret: env
                .secret("STORAGE_SIGN_SECRET")
                .map(|v| v.expose().to_owned())
                .unwrap_or_default(),
        }
    }

    /// True when an authentication method is configured (static token, or a
    /// JWT public key when the `auth` feature is enabled).
    pub fn is_authenticated(&self) -> bool {
        if !self.store_token.is_empty() {
            return true;
        }
        #[cfg(feature = "auth")]
        if self
            .jwt_public_key_pem
            .as_ref()
            .is_some_and(|pem| !pem.trim().is_empty())
        {
            return true;
        }
        false
    }

    /// Validates that server exposure is authenticated or deliberately local.
    pub fn validate_server_security(&self) -> Result<(), StorageError> {
        if self.server_mode
            && !self.server_bind.is_loopback()
            && !self.is_authenticated()
            && !self.allow_insecure_public
        {
            return Err(StorageError::Config(
                "unauthenticated public storage bind rejected; set STORE_TOKEN, use a loopback STORAGE_BIND, or explicitly set STORAGE_ALLOW_INSECURE_PUBLIC=true".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
