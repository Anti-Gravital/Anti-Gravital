//! Anti-Gravital native store: filesystem, embedded Axum HTTP server,
//! optional S3/MinIO adapters and image processing.
//!
//! # Minimal usage (embedded)
//!
//! ```no_run
//! use ag_storage::{AgStorage, StorageConfig};
//! use bytes::Bytes;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let storage = AgStorage::new(StorageConfig::default()).await?;
//! storage.put("docs/readme.txt", Bytes::from("hola")).await?;
//! let data = storage.get("docs/readme.txt").await?;
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod image;
pub mod signed;
pub mod store;

pub use config::{StorageBackend, StorageConfig};
pub use image::ImageProcessor;
pub use signed::{sign_url, verify_signed_url, SignedUrlError};
pub use store::AgStore;

use thiserror::Error;

/// Storage subsystem error.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StorageError {
    /// No object found with the given key.
    #[error("object not found: {0}")]
    NotFound(String),
    /// Invalid object key (forbidden characters, path traversal, etc.).
    #[error("invalid key: {0}")]
    InvalidKey(String),
    /// Attempt to escape the store root directory.
    #[error("access outside the store denied")]
    PathEscape(String),
    /// Payload exceeds the configured limit.
    #[error("object too large: {size} bytes (limit: {limit} bytes)")]
    TooLarge {
        /// Size of the received object in bytes.
        size: usize,
        /// Maximum allowed size in bytes.
        limit: usize,
    },
    /// Operating system I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Image processing error.
    #[error("image error: {0}")]
    Image(String),
    /// Invalid configuration.
    #[error("configuration error: {0}")]
    Config(String),
    #[cfg(feature = "s3")]
    /// S3/MinIO backend error.
    #[error("S3 error: {0}")]
    S3(#[from] object_store::Error),
}

/// Main facade of the storage subsystem.
///
/// Construct with [`AgStorage::new`] passing a [`StorageConfig`].
/// If `config.server_mode` is `true`, it spins up an HTTP server in the background.
pub struct AgStorage {
    store: std::sync::Arc<AgStore>,
    config: StorageConfig,
}

impl AgStorage {
    /// Creates a new instance. If `config.server_mode` is `true`, starts
    /// the HTTP server in the background via `tokio::spawn`.
    pub async fn new(config: StorageConfig) -> Result<Self, StorageError> {
        config.validate_server_security()?;
        let store = std::sync::Arc::new(AgStore::new(&config)?);
        if config.server_mode {
            let srv_store = std::sync::Arc::clone(&store);
            let srv_config = config.clone();
            tokio::spawn(async move {
                if let Err(e) = store::server::run_server(srv_store, &srv_config).await {
                    tracing::error!(error = %e, "ag-storage server error");
                }
            });
        }
        Ok(Self { store, config })
    }

    /// Stores `data` under the key `key`.
    pub async fn put(&self, key: &str, data: bytes::Bytes) -> Result<(), StorageError> {
        self.store.put(key, data).await
    }

    /// Retrieves the content of the object with key `key`.
    pub async fn get(&self, key: &str) -> Result<bytes::Bytes, StorageError> {
        self.store.get(key).await
    }

    /// Deletes the object with key `key`.
    pub async fn delete(&self, key: &str) -> Result<(), StorageError> {
        self.store.delete(key).await
    }

    /// Returns `true` if an object with key `key` exists.
    pub async fn exists(&self, key: &str) -> Result<bool, StorageError> {
        self.store.exists(key).await
    }

    /// Lists object keys, optionally filtered by prefix.
    pub async fn list(&self, prefix: Option<&str>) -> Result<Vec<String>, StorageError> {
        self.store.list(prefix).await
    }

    /// Copies the object `from` to the key `to`.
    pub async fn copy(&self, from: &str, to: &str) -> Result<(), StorageError> {
        self.store.copy(from, to).await
    }

    /// Returns the access URL for the object.
    /// - Embedded native: `file://{root}/{key}`
    /// - Active server: `http://localhost:{port}/v1/objects/{key}`
    pub fn object_url(&self, key: &str) -> Result<String, StorageError> {
        if self.config.server_mode {
            Ok(format!(
                "http://{}:{}/v1/objects/{}",
                self.config.server_bind, self.config.server_port, key
            ))
        } else {
            let path = self.config.root_path.join(key);
            Ok(format!("file://{}", path.display()))
        }
    }

    /// Returns an [`ImageProcessor`] for processing images.
    pub fn processor(&self) -> ImageProcessor {
        ImageProcessor::new()
    }

    /// Generates a signed token for temporary access to `key`.
    ///
    /// `ttl_secs`: time to live in seconds. Errors if `sign_secret` is empty.
    pub fn signed_url(&self, key: &str, ttl_secs: u64) -> Result<String, SignedUrlError> {
        let expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            + ttl_secs;
        crate::signed::sign_url(&self.config.sign_secret, key, expires_at)
    }

    /// Verifies that the token grants access to `key`.
    pub fn verify_signed_url(&self, key: &str, token: &str) -> Result<(), SignedUrlError> {
        crate::signed::verify_signed_url(&self.config.sign_secret, key, token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    fn temp_config() -> (tempfile::TempDir, StorageConfig) {
        let dir = tempfile::tempdir().unwrap();
        let config = StorageConfig {
            root_path: dir.path().to_path_buf(),
            server_mode: false,
            ..StorageConfig::default()
        };
        (dir, config)
    }

    #[test]
    fn storage_error_not_found_display() {
        let e = StorageError::NotFound("avatars/user.jpg".into());
        assert_eq!(e.to_string(), "object not found: avatars/user.jpg");
    }

    #[test]
    fn storage_error_too_large_display() {
        let e = StorageError::TooLarge {
            size: 200,
            limit: 100,
        };
        assert!(e.to_string().contains("200"));
        assert!(e.to_string().contains("100"));
    }

    #[test]
    fn storage_error_invalid_key_display() {
        let e = StorageError::InvalidKey("../secret".into());
        assert!(e.to_string().contains("invalid"));
    }

    #[tokio::test]
    async fn new_creates_successfully() {
        let (_dir, config) = temp_config();
        let storage = AgStorage::new(config).await.unwrap();
        let _ = storage.processor();
    }

    #[tokio::test]
    async fn put_get_roundtrip() {
        let (_dir, config) = temp_config();
        let storage = AgStorage::new(config).await.unwrap();
        let data = Bytes::from("hello storage");
        storage.put("files/test.txt", data.clone()).await.unwrap();
        let got = storage.get("files/test.txt").await.unwrap();
        assert_eq!(got, data);
    }

    #[tokio::test]
    async fn exists_returns_true_after_put() {
        let (_dir, config) = temp_config();
        let storage = AgStorage::new(config).await.unwrap();
        assert!(!storage.exists("x.bin").await.unwrap());
        storage.put("x.bin", Bytes::from("x")).await.unwrap();
        assert!(storage.exists("x.bin").await.unwrap());
    }

    #[tokio::test]
    async fn delete_removes_object() {
        let (_dir, config) = temp_config();
        let storage = AgStorage::new(config).await.unwrap();
        storage.put("del.txt", Bytes::from("bye")).await.unwrap();
        storage.delete("del.txt").await.unwrap();
        assert!(!storage.exists("del.txt").await.unwrap());
    }

    #[tokio::test]
    async fn list_returns_put_keys() {
        let (_dir, config) = temp_config();
        let storage = AgStorage::new(config).await.unwrap();
        storage.put("ns/a.txt", Bytes::from("a")).await.unwrap();
        storage.put("ns/b.txt", Bytes::from("b")).await.unwrap();
        let keys = storage.list(Some("ns/")).await.unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[tokio::test]
    async fn copy_duplicates_object() {
        let (_dir, config) = temp_config();
        let storage = AgStorage::new(config).await.unwrap();
        storage
            .put("src.txt", Bytes::from("original"))
            .await
            .unwrap();
        storage.copy("src.txt", "dst.txt").await.unwrap();
        let got = storage.get("dst.txt").await.unwrap();
        assert_eq!(got, Bytes::from("original"));
    }

    #[tokio::test]
    async fn object_url_native_mode() {
        let (_dir, config) = temp_config();
        let storage = AgStorage::new(config).await.unwrap();
        let url = storage.object_url("docs/readme.txt").unwrap();
        assert!(url.starts_with("file://"), "native URL must be file://");
    }

    #[tokio::test]
    async fn object_url_server_mode() {
        let dir = tempfile::tempdir().unwrap();
        let config = StorageConfig {
            root_path: dir.path().to_path_buf(),
            server_mode: true,
            server_port: 14280,
            ..StorageConfig::default()
        };
        let storage = AgStorage::new(config).await.unwrap();
        let url = storage.object_url("docs/readme.txt").unwrap();
        assert!(url.contains("14280"), "server URL must contain the port");
        assert!(url.starts_with("http://"));
    }
}
