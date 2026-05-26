//! Storage store configuration.

use std::path::PathBuf;

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
    /// HTTP server port. Default: 4280.
    pub server_port: u16,
    /// Static Bearer token. Empty = no authentication (dev mode).
    pub store_token: String,
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
            server_port: 4280,
            store_token: String::new(),
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

impl StorageConfig {
    /// Reads the configuration from environment variables.
    /// Undefined values use the defaults from [`StorageConfig::default`].
    pub fn from_env() -> Self {
        let backend = match std::env::var("STORAGE_BACKEND")
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
            root_path: std::env::var("STORAGE_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("./ag-store-data")),
            server_mode: std::env::var("STORAGE_SERVER")
                .map(|v| v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            server_port: std::env::var("STORAGE_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4280),
            store_token: std::env::var("STORE_TOKEN").unwrap_or_default(),
            max_object_size_mb: std::env::var("STORAGE_MAX_OBJECT_SIZE_MB")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            rate_limit_rps: std::env::var("STORAGE_RATE_LIMIT_RPS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            region: std::env::var("STORAGE_REGION").unwrap_or_else(|_| "us-east-1".into()),
            endpoint: std::env::var("STORAGE_ENDPOINT").ok(),
            access_key: std::env::var("AWS_ACCESS_KEY_ID").ok(),
            secret_key: std::env::var("AWS_SECRET_ACCESS_KEY").ok(),
            bucket: std::env::var("STORAGE_BUCKET").unwrap_or_else(|_| "ag-storage".into()),
            sign_secret: std::env::var("STORAGE_SIGN_SECRET").unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_are_native() {
        let cfg = StorageConfig::default();
        assert_eq!(cfg.backend, StorageBackend::Native);
        assert_eq!(cfg.root_path, PathBuf::from("./ag-store-data"));
    }

    #[test]
    fn config_server_mode_off_by_default() {
        let cfg = StorageConfig::default();
        assert!(!cfg.server_mode);
        assert_eq!(cfg.server_port, 4280);
    }

    #[test]
    fn config_from_env_reads_port() {
        let prev = std::env::var("STORAGE_PORT").ok();
        std::env::set_var("STORAGE_PORT", "9000");
        let cfg = StorageConfig::from_env();
        match prev {
            Some(v) => std::env::set_var("STORAGE_PORT", v),
            None => std::env::remove_var("STORAGE_PORT"),
        }
        assert_eq!(cfg.server_port, 9000);
    }

    #[test]
    fn config_from_env_reads_server_mode() {
        let prev = std::env::var("STORAGE_SERVER").ok();
        std::env::set_var("STORAGE_SERVER", "true");
        let cfg = StorageConfig::from_env();
        match prev {
            Some(v) => std::env::set_var("STORAGE_SERVER", v),
            None => std::env::remove_var("STORAGE_SERVER"),
        }
        assert!(cfg.server_mode);
    }

    #[test]
    fn config_sign_secret_default_empty() {
        let cfg = StorageConfig::default();
        assert!(cfg.sign_secret.is_empty());
    }

    #[test]
    fn config_from_env_reads_sign_secret() {
        std::env::set_var("STORAGE_SIGN_SECRET", "my-secret-key");
        let cfg = StorageConfig::from_env();
        std::env::remove_var("STORAGE_SIGN_SECRET");
        assert_eq!(cfg.sign_secret, "my-secret-key");
    }
}
