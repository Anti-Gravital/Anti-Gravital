//! Servidor HTTP Axum embebido.
use crate::{StorageConfig, StorageError};
use std::sync::Arc;
use super::AgStore;

pub async fn run_server(_store: Arc<AgStore>, _config: &StorageConfig) -> Result<(), StorageError> {
    Ok(())
}
