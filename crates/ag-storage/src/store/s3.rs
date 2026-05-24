//! Backend S3/MinIO de ag-storage.
//!
//! Solo disponible con la feature `s3`. Usa el crate `object_store`
//! (Apache-2.0) que soporta AWS S3 y cualquier backend S3-compatible (MinIO).

use crate::{StorageConfig, StorageError};
use bytes::Bytes;
use futures_util::StreamExt;
use object_store::{aws::AmazonS3Builder, path::Path, ObjectStore};
use std::sync::Arc;

/// Backend de almacenamiento S3/MinIO.
pub struct S3Store {
    inner: Arc<dyn ObjectStore>,
}

impl S3Store {
    /// Construye el store S3 o MinIO segun la configuracion.
    pub fn new(config: &StorageConfig) -> Result<Self, StorageError> {
        let mut builder = AmazonS3Builder::new()
            .with_bucket_name(&config.bucket)
            .with_region(&config.region);

        if let Some(ref key) = config.access_key {
            builder = builder.with_access_key_id(key);
        }
        if let Some(ref secret) = config.secret_key {
            builder = builder.with_secret_access_key(secret);
        }
        if let Some(ref endpoint) = config.endpoint {
            builder = builder.with_endpoint(endpoint).with_allow_http(true);
        }

        let store = builder
            .build()
            .map_err(|e| StorageError::Config(format!("S3 build: {e}")))?;

        Ok(Self {
            inner: Arc::new(store),
        })
    }

    fn to_path(key: &str) -> Path {
        Path::from(key)
    }

    /// Almacena un objeto.
    pub async fn put(&self, key: &str, data: Bytes) -> Result<(), StorageError> {
        self.inner
            .put(&Self::to_path(key), data.into())
            .await
            .map(|_| ())
            .map_err(StorageError::S3)
    }

    /// Recupera un objeto.
    pub async fn get(&self, key: &str) -> Result<Bytes, StorageError> {
        match self.inner.get(&Self::to_path(key)).await {
            Ok(r) => r.bytes().await.map_err(StorageError::S3),
            Err(object_store::Error::NotFound { .. }) => {
                Err(StorageError::NotFound(key.to_owned()))
            }
            Err(e) => Err(StorageError::S3(e)),
        }
    }

    /// Borra un objeto.
    pub async fn delete(&self, key: &str) -> Result<(), StorageError> {
        match self.inner.delete(&Self::to_path(key)).await {
            Ok(()) => Ok(()),
            Err(object_store::Error::NotFound { .. }) => {
                Err(StorageError::NotFound(key.to_owned()))
            }
            Err(e) => Err(StorageError::S3(e)),
        }
    }

    /// Retorna true si existe el objeto.
    pub async fn exists(&self, key: &str) -> Result<bool, StorageError> {
        match self.inner.head(&Self::to_path(key)).await {
            Ok(_) => Ok(true),
            Err(object_store::Error::NotFound { .. }) => Ok(false),
            Err(e) => Err(StorageError::S3(e)),
        }
    }

    /// Lista objetos con prefijo opcional.
    pub async fn list(&self, prefix: Option<&str>) -> Result<Vec<String>, StorageError> {
        let path_prefix = prefix.map(Path::from);
        let mut stream = self.inner.list(path_prefix.as_ref());
        let mut keys = Vec::new();
        while let Some(meta) = stream.next().await {
            let meta = meta.map_err(StorageError::S3)?;
            keys.push(meta.location.to_string());
        }
        Ok(keys)
    }

    /// Copia un objeto.
    pub async fn copy(&self, from: &str, to: &str) -> Result<(), StorageError> {
        self.inner
            .copy(&Self::to_path(from), &Self::to_path(to))
            .await
            .map_err(StorageError::S3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has_s3_env() -> bool {
        std::env::var("AWS_ACCESS_KEY_ID").is_ok() || std::env::var("MINIO_URL").is_ok()
    }

    #[test]
    fn s3_store_build_without_credentials() {
        if has_s3_env() {
            return;
        }
        let config = StorageConfig {
            bucket: "test-bucket".into(),
            region: "us-east-1".into(),
            access_key: None,
            secret_key: None,
            endpoint: None,
            ..StorageConfig::default()
        };
        let result = S3Store::new(&config);
        let _ = result;
    }

    #[tokio::test]
    async fn s3_put_get_roundtrip_with_minio() {
        if !has_s3_env() {
            return;
        }
        let config = StorageConfig {
            bucket: std::env::var("MINIO_BUCKET").unwrap_or_else(|_| "ag-test".into()),
            region: "us-east-1".into(),
            access_key: std::env::var("AWS_ACCESS_KEY_ID").ok(),
            secret_key: std::env::var("AWS_SECRET_ACCESS_KEY").ok(),
            endpoint: std::env::var("MINIO_URL").ok(),
            ..StorageConfig::default()
        };
        let store = S3Store::new(&config).expect("debe construirse");
        let data = Bytes::from("contenido de prueba s3");
        store
            .put("test/ag-s3-test.txt", data.clone())
            .await
            .expect("put");
        let result = store.get("test/ag-s3-test.txt").await.expect("get");
        assert_eq!(result, data);
        store.delete("test/ag-s3-test.txt").await.expect("delete");
    }
}
