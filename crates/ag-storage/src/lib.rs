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
    TooLarge { size: usize, limit: usize },
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
    use super::StorageError;

    #[test]
    fn storage_error_not_found_display() {
        let e = StorageError::NotFound("avatars/user.jpg".into());
        assert_eq!(e.to_string(), "objeto no encontrado: avatars/user.jpg");
    }

    #[test]
    fn storage_error_too_large_display() {
        let e = StorageError::TooLarge { size: 200, limit: 100 };
        assert!(e.to_string().contains("200"));
        assert!(e.to_string().contains("100"));
    }

    #[test]
    fn storage_error_invalid_key_display() {
        let e = StorageError::InvalidKey("../secret".into());
        assert!(e.to_string().contains("invalida"));
    }
}
