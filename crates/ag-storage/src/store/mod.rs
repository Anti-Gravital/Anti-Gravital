//! Backend nativo del store.
pub mod auth;
pub mod server;

use crate::{StorageConfig, StorageError};
use bytes::Bytes;
use std::path::PathBuf;

pub struct AgStore {
    root: PathBuf,
    max_object_size: usize,
}

impl AgStore {
    pub fn new(config: &StorageConfig) -> Result<Self, StorageError> {
        std::fs::create_dir_all(&config.root_path).map_err(StorageError::Io)?;
        let root = config.root_path.canonicalize().map_err(StorageError::Io)?;
        Ok(Self { root, max_object_size: config.max_object_size_mb as usize * 1024 * 1024 })
    }
    pub async fn put(&self, _key: &str, _data: Bytes) -> Result<(), StorageError> { todo!() }
    pub async fn get(&self, _key: &str) -> Result<Bytes, StorageError> { todo!() }
    pub async fn delete(&self, _key: &str) -> Result<(), StorageError> { todo!() }
    pub async fn exists(&self, _key: &str) -> Result<bool, StorageError> { todo!() }
    pub async fn list(&self, _prefix: Option<&str>) -> Result<Vec<String>, StorageError> { todo!() }
    pub async fn copy(&self, _from: &str, _to: &str) -> Result<(), StorageError> { todo!() }
}
