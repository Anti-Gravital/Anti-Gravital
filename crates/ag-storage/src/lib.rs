//! Store nativo Anti-Gravital: filesystem, servidor HTTP Axum embebido,
//! adaptadores S3/MinIO opcionales y procesamiento de imagen.
//!
//! # Uso minimo (embebido)
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
pub mod store;

pub use config::{StorageBackend, StorageConfig};
pub use image::ImageProcessor;
pub use store::AgStore;

use thiserror::Error;

/// Error del subsistema de almacenamiento.
#[derive(Debug, Error)]
pub enum StorageError {
    /// Objeto no encontrado con la clave dada.
    #[error("objeto no encontrado: {0}")]
    NotFound(String),
    /// Clave de objeto invalida (caracteres prohibidos, path traversal, etc.).
    #[error("clave invalida: {0}")]
    InvalidKey(String),
    /// Intento de escapar del directorio raiz del store.
    #[error("acceso fuera del store denegado")]
    PathEscape(String),
    /// Payload supera el limite configurado.
    #[error("objeto demasiado grande: {size} bytes (limite: {limit} bytes)")]
    TooLarge {
        /// Tamano del objeto recibido en bytes.
        size: usize,
        /// Limite maximo permitido en bytes.
        limit: usize,
    },
    /// Error de I/O del sistema operativo.
    #[error("error de I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Error de procesamiento de imagen.
    #[error("error de imagen: {0}")]
    Image(String),
    /// Configuracion invalida.
    #[error("error de configuracion: {0}")]
    Config(String),
    #[cfg(feature = "s3")]
    /// Error del backend S3/MinIO.
    #[error("error S3: {0}")]
    S3(#[from] object_store::Error),
}

/// Facade principal del subsistema de almacenamiento.
///
/// Construir con [`AgStorage::new`] pasando un [`StorageConfig`].
/// Si `config.server_mode` es `true`, levanta un servidor HTTP en background.
pub struct AgStorage {
    store: std::sync::Arc<AgStore>,
    config: StorageConfig,
}

impl AgStorage {
    /// Crea una nueva instancia. Si `config.server_mode` es `true`, inicia
    /// el servidor HTTP en background via `tokio::spawn`.
    pub async fn new(config: StorageConfig) -> Result<Self, StorageError> {
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

    /// Almacena `data` bajo la clave `key`.
    pub async fn put(&self, key: &str, data: bytes::Bytes) -> Result<(), StorageError> {
        self.store.put(key, data).await
    }

    /// Recupera el contenido del objeto con clave `key`.
    pub async fn get(&self, key: &str) -> Result<bytes::Bytes, StorageError> {
        self.store.get(key).await
    }

    /// Borra el objeto con clave `key`.
    pub async fn delete(&self, key: &str) -> Result<(), StorageError> {
        self.store.delete(key).await
    }

    /// Retorna `true` si existe un objeto con clave `key`.
    pub async fn exists(&self, key: &str) -> Result<bool, StorageError> {
        self.store.exists(key).await
    }

    /// Lista las claves de objetos, opcionalmente filtradas por prefijo.
    pub async fn list(&self, prefix: Option<&str>) -> Result<Vec<String>, StorageError> {
        self.store.list(prefix).await
    }

    /// Copia el objeto `from` a la clave `to`.
    pub async fn copy(&self, from: &str, to: &str) -> Result<(), StorageError> {
        self.store.copy(from, to).await
    }

    /// Retorna la URL de acceso al objeto.
    /// - Native embebido: `file://{root}/{key}`
    /// - Servidor activo: `http://localhost:{port}/v1/objects/{key}`
    pub fn object_url(&self, key: &str) -> Result<String, StorageError> {
        if self.config.server_mode {
            Ok(format!(
                "http://localhost:{}/v1/objects/{}",
                self.config.server_port, key
            ))
        } else {
            let path = self.config.root_path.join(key);
            Ok(format!("file://{}", path.display()))
        }
    }

    /// Retorna un [`ImageProcessor`] para procesar imagenes.
    pub fn processor(&self) -> ImageProcessor {
        ImageProcessor::new()
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
        assert_eq!(e.to_string(), "objeto no encontrado: avatars/user.jpg");
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
        assert!(e.to_string().contains("invalida"));
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
        assert!(url.starts_with("file://"), "URL nativa debe ser file://");
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
        assert!(url.contains("14280"), "URL server debe contener el puerto");
        assert!(url.starts_with("http://"));
    }
}
