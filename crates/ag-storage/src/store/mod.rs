//! Backend nativo del store.
pub mod auth;
pub mod server;

use crate::{StorageConfig, StorageError};
use bytes::Bytes;
use std::path::PathBuf;

pub struct AgStore {
    #[allow(dead_code)]
    root: PathBuf,
    #[allow(dead_code)]
    max_object_size: usize,
}

impl AgStore {
    pub fn new(config: &StorageConfig) -> Result<Self, StorageError> {
        std::fs::create_dir_all(&config.root_path).map_err(StorageError::Io)?;
        let root = config.root_path.canonicalize().map_err(StorageError::Io)?;
        Ok(Self {
            root,
            max_object_size: config.max_object_size_mb as usize * 1024 * 1024,
        })
    }

    pub async fn put(&self, _key: &str, _data: Bytes) -> Result<(), StorageError> {
        // TECH-DEBT:
        // motivo: implementacion completa en Task 5 (write-then-rename atomico).
        // impacto: put no funcional hasta Task 5.
        // eliminacion esperada: Task 5 ag-storage.
        todo!()
    }

    pub async fn get(&self, _key: &str) -> Result<Bytes, StorageError> {
        // TECH-DEBT:
        // motivo: implementacion completa en Task 5.
        // impacto: get no funcional hasta Task 5.
        // eliminacion esperada: Task 5 ag-storage.
        todo!()
    }

    pub async fn delete(&self, _key: &str) -> Result<(), StorageError> {
        // TECH-DEBT:
        // motivo: implementacion completa en Task 5.
        // impacto: delete no funcional hasta Task 5.
        // eliminacion esperada: Task 5 ag-storage.
        todo!()
    }

    pub async fn exists(&self, _key: &str) -> Result<bool, StorageError> {
        // TECH-DEBT:
        // motivo: implementacion completa en Task 5.
        // impacto: exists no funcional hasta Task 5.
        // eliminacion esperada: Task 5 ag-storage.
        todo!()
    }

    pub async fn list(&self, _prefix: Option<&str>) -> Result<Vec<String>, StorageError> {
        // TECH-DEBT:
        // motivo: implementacion completa en Task 6.
        // impacto: list no funcional hasta Task 6.
        // eliminacion esperada: Task 6 ag-storage.
        todo!()
    }

    pub async fn copy(&self, _from: &str, _to: &str) -> Result<(), StorageError> {
        // TECH-DEBT:
        // motivo: implementacion completa en Task 6.
        // impacto: copy no funcional hasta Task 6.
        // eliminacion esperada: Task 6 ag-storage.
        todo!()
    }
}
