//! Configuracion del store de almacenamiento.

use std::path::PathBuf;

/// Backend de almacenamiento activo.
#[derive(Debug, Clone, PartialEq)]
pub enum StorageBackend {
    /// Filesystem local (default). Sin dependencias externas.
    Native,
    #[cfg(feature = "s3")]
    /// AWS S3 o compatible.
    S3,
    #[cfg(feature = "s3")]
    /// MinIO self-hosted (S3-compatible).
    MinIO,
}

/// Configuracion del subsistema de almacenamiento.
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Backend activo.
    pub backend: StorageBackend,
    /// Directorio raiz del store nativo.
    pub root_path: PathBuf,
    /// Si `true`, levanta un servidor HTTP Axum en background.
    pub server_mode: bool,
    /// Puerto del servidor HTTP. Default: 4280.
    pub server_port: u16,
    /// Token Bearer estatico. Vacio = sin autenticacion (modo dev).
    pub store_token: String,
    /// Tamano maximo de objeto en MB. Default: 100.
    pub max_object_size_mb: u64,
    /// Limite de requests por segundo del servidor HTTP. Default: 100.
    pub rate_limit_rps: u32,
    /// Region AWS. Default: "us-east-1".
    pub region: String,
    /// Endpoint personalizado (para MinIO). None = AWS.
    pub endpoint: Option<String>,
    /// Access key AWS.
    pub access_key: Option<String>,
    /// Secret key AWS.
    pub secret_key: Option<String>,
    /// Nombre del bucket S3/MinIO. Default: "ag-storage".
    pub bucket: String,
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
        }
    }
}

impl StorageConfig {
    /// Lee la configuracion desde variables de entorno.
    /// Valores no definidos usan los defaults de [`StorageConfig::default`].
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
}
