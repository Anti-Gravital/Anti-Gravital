//! Anti-Gravital store backend: native filesystem and optional S3/MinIO.

#[cfg(feature = "s3")]
pub mod s3;
#[cfg(feature = "s3")]
pub use s3::S3Store;

pub mod auth;
pub mod server;

use crate::{StorageBackend, StorageConfig, StorageError};
use bytes::Bytes;
use std::path::{Component, Path, PathBuf};

// ---------------------------------------------------------------------------
// NativeStore: implementation over local filesystem
// ---------------------------------------------------------------------------

/// Native Anti-Gravital store.
///
/// Stores objects as files under `root`. The key is the relative path.
/// Every operation passes through [`validate_key`] and [`resolve_path`] before I/O.
pub struct NativeStore {
    pub(crate) root: PathBuf,
    max_object_size: usize,
}

impl NativeStore {
    /// Root directory of the store (canonicalized).
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Stores `data` under key `key`.
    ///
    /// Uses write-then-atomic-rename to avoid reads of partial files.
    pub async fn put(&self, key: &str, data: Bytes) -> Result<(), StorageError> {
        if data.len() > self.max_object_size {
            return Err(StorageError::TooLarge {
                size: data.len(),
                limit: self.max_object_size,
            });
        }
        let dest = resolve_path(&self.root, key)?;
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        // atomic write-then-rename: avoids reads of partially written files
        let mut nonce = [0u8; 8];
        getrandom::getrandom(&mut nonce).unwrap_or_default();
        let tmp = dest.with_file_name(format!(".tmp.{:016x}", u64::from_le_bytes(nonce)));
        tokio::fs::write(&tmp, &data).await?;
        tokio::fs::rename(&tmp, &dest).await?;
        Ok(())
    }

    /// Retrieves the content of the object with key `key`.
    pub async fn get(&self, key: &str) -> Result<Bytes, StorageError> {
        let path = resolve_path(&self.root, key)?;
        match tokio::fs::read(&path).await {
            Ok(data) => Ok(Bytes::from(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Err(StorageError::NotFound(key.to_owned()))
            }
            Err(e) => Err(StorageError::Io(e)),
        }
    }

    /// Deletes the object with key `key`.
    pub async fn delete(&self, key: &str) -> Result<(), StorageError> {
        let path = resolve_path(&self.root, key)?;
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Err(StorageError::NotFound(key.to_owned()))
            }
            Err(e) => Err(StorageError::Io(e)),
        }
    }

    /// Returns `true` if an object with key `key` exists.
    pub async fn exists(&self, key: &str) -> Result<bool, StorageError> {
        let path = resolve_path(&self.root, key)?;
        Ok(tokio::fs::try_exists(&path).await.unwrap_or(false))
    }

    /// Lists object keys under `prefix`.
    pub async fn list(&self, prefix: Option<&str>) -> Result<Vec<String>, StorageError> {
        let root = self.root.clone();
        let prefix_str = prefix.map(|p| p.trim_end_matches('/').to_owned());

        let keys = tokio::task::spawn_blocking(move || collect_keys(&root, &root))
            .await
            .map_err(|e| StorageError::Io(std::io::Error::other(e.to_string())))??;

        Ok(match prefix_str {
            Some(p) => keys
                .into_iter()
                .filter(|k| k == &p || k.starts_with(&format!("{p}/")))
                .collect(),
            None => keys,
        })
    }

    /// Copies object `from` to key `to`.
    pub async fn copy(&self, from: &str, to: &str) -> Result<(), StorageError> {
        let data = self.get(from).await?;
        self.put(to, data).await
    }
}

// ---------------------------------------------------------------------------
// AgStore: enum that unifies NativeStore and S3Store
// ---------------------------------------------------------------------------

/// Storage backend — native filesystem or S3/MinIO.
pub enum AgStore {
    /// Local filesystem backend.
    Native(NativeStore),
    #[cfg(feature = "s3")]
    /// S3/MinIO backend via object_store.
    S3(S3Store),
}

impl AgStore {
    /// Builds the store according to `config.backend`.
    pub fn new(config: &StorageConfig) -> Result<Self, StorageError> {
        match &config.backend {
            StorageBackend::Native => {
                std::fs::create_dir_all(&config.root_path).map_err(StorageError::Io)?;
                let root = config.root_path.canonicalize().map_err(StorageError::Io)?;
                Ok(AgStore::Native(NativeStore {
                    root,
                    max_object_size: config.max_object_size_mb as usize * 1024 * 1024,
                }))
            }
            #[cfg(feature = "s3")]
            StorageBackend::S3 | StorageBackend::MinIO => Ok(AgStore::S3(S3Store::new(config)?)),
        }
    }

    /// Stores `data` under key `key`.
    pub async fn put(&self, key: &str, data: Bytes) -> Result<(), StorageError> {
        match self {
            AgStore::Native(s) => s.put(key, data).await,
            #[cfg(feature = "s3")]
            AgStore::S3(s) => s.put(key, data).await,
        }
    }

    /// Retrieves the content of the object with key `key`.
    pub async fn get(&self, key: &str) -> Result<Bytes, StorageError> {
        match self {
            AgStore::Native(s) => s.get(key).await,
            #[cfg(feature = "s3")]
            AgStore::S3(s) => s.get(key).await,
        }
    }

    /// Deletes the object with key `key`.
    pub async fn delete(&self, key: &str) -> Result<(), StorageError> {
        match self {
            AgStore::Native(s) => s.delete(key).await,
            #[cfg(feature = "s3")]
            AgStore::S3(s) => s.delete(key).await,
        }
    }

    /// Returns `true` if an object with key `key` exists.
    pub async fn exists(&self, key: &str) -> Result<bool, StorageError> {
        match self {
            AgStore::Native(s) => s.exists(key).await,
            #[cfg(feature = "s3")]
            AgStore::S3(s) => s.exists(key).await,
        }
    }

    /// Lists object keys under `prefix`.
    pub async fn list(&self, prefix: Option<&str>) -> Result<Vec<String>, StorageError> {
        match self {
            AgStore::Native(s) => s.list(prefix).await,
            #[cfg(feature = "s3")]
            AgStore::S3(s) => s.list(prefix).await,
        }
    }

    /// Copies object `from` to key `to`.
    pub async fn copy(&self, from: &str, to: &str) -> Result<(), StorageError> {
        match self {
            AgStore::Native(s) => s.copy(from, to).await,
            #[cfg(feature = "s3")]
            AgStore::S3(s) => s.copy(from, to).await,
        }
    }

    /// Returns the root path if the backend is native.
    pub fn root(&self) -> Option<&std::path::Path> {
        match self {
            AgStore::Native(s) => Some(&s.root),
            #[cfg(feature = "s3")]
            AgStore::S3(_) => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Security: key validation and path confinement
// ---------------------------------------------------------------------------

/// Validates that `key` is a safe object key.
///
/// Rejects with [`StorageError::InvalidKey`] if the key:
/// - Is empty or exceeds 1024 bytes.
/// - Contains null bytes or control characters (< 0x20, except `/`).
/// - Contains `.` or `..` segments.
/// - Starts or ends with `/`.
/// - Contains `//` sequences.
pub fn validate_key(key: &str) -> Result<(), StorageError> {
    if key.is_empty() {
        return Err(StorageError::InvalidKey("empty key".into()));
    }
    if key.len() > 1024 {
        return Err(StorageError::InvalidKey("key exceeds 1024 bytes".into()));
    }
    if key.starts_with('/') || key.ends_with('/') {
        return Err(StorageError::InvalidKey(
            "key cannot start or end with '/'".into(),
        ));
    }
    if key.contains("//") {
        return Err(StorageError::InvalidKey("key cannot contain '//'".into()));
    }
    for byte in key.bytes() {
        if byte == 0 {
            return Err(StorageError::InvalidKey("key contains null byte".into()));
        }
        if byte < 0x20 && byte != b'/' {
            return Err(StorageError::InvalidKey(
                "key contains control character".into(),
            ));
        }
    }
    for segment in key.split('/') {
        if segment == "." || segment == ".." {
            return Err(StorageError::InvalidKey(format!(
                "forbidden path segment: '{segment}'"
            )));
        }
    }
    Ok(())
}

/// Normalizes a path without touching disk (resolves `.` and `..` lexically).
fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            c => out.push(c),
        }
    }
    out
}

/// Recursively walks `dir` and returns paths relative to `root`.
/// Ignores temporary files with the `.tmp.` prefix.
fn collect_keys(
    dir: &std::path::Path,
    root: &std::path::Path,
) -> Result<Vec<String>, StorageError> {
    let mut keys = Vec::new();
    if !dir.exists() {
        return Ok(keys);
    }
    for entry in std::fs::read_dir(dir).map_err(StorageError::Io)? {
        let entry = entry.map_err(StorageError::Io)?;
        let path = entry.path();
        if path.is_dir() {
            let mut sub = collect_keys(&path, root)?;
            keys.append(&mut sub);
        } else if path.is_file() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name.starts_with(".tmp.") {
                continue;
            }
            if let Ok(rel) = path.strip_prefix(root) {
                keys.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    Ok(keys)
}

/// Resolves `key` to an absolute path inside `root`.
///
/// Guarantees that the resulting path stays within `root`.
/// Returns [`StorageError::PathEscape`] if the resolved path escapes the root.
pub fn resolve_path(root: &Path, key: &str) -> Result<PathBuf, StorageError> {
    validate_key(key)?;
    let canonical_root = root.canonicalize().map_err(StorageError::Io)?;
    let candidate = canonical_root.join(key);
    let resolved = if candidate.exists() {
        candidate.canonicalize().map_err(StorageError::Io)?
    } else {
        // Safe because validate_key (called above) already rejected all
        // '.' and '..' segments. normalize_path is just additional defense.
        normalize_path(&candidate)
    };
    if !resolved.starts_with(&canonical_root) {
        return Err(StorageError::PathEscape(key.to_owned()));
    }
    Ok(resolved)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn tempdir() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    // --- Security tests ---

    #[test]
    fn key_with_dotdot_is_rejected() {
        assert!(matches!(
            validate_key("../secret"),
            Err(StorageError::InvalidKey(_))
        ));
        assert!(matches!(
            validate_key("foo/../../../etc/passwd"),
            Err(StorageError::InvalidKey(_))
        ));
    }

    #[test]
    fn key_with_null_byte_is_rejected() {
        assert!(matches!(
            validate_key("foo\0bar"),
            Err(StorageError::InvalidKey(_))
        ));
    }

    #[test]
    fn key_starting_with_slash_is_rejected() {
        assert!(matches!(
            validate_key("/etc/passwd"),
            Err(StorageError::InvalidKey(_))
        ));
        assert!(matches!(
            validate_key("foo/"),
            Err(StorageError::InvalidKey(_))
        ));
    }

    #[test]
    fn valid_keys_are_accepted() {
        assert!(validate_key("avatars/user-123.jpg").is_ok());
        assert!(validate_key("docs/reports/q1.pdf").is_ok());
        assert!(validate_key("file.txt").is_ok());
        assert!(validate_key("a/b/c/d.json").is_ok());
    }

    #[test]
    #[cfg(unix)]
    fn symlink_escape_is_blocked() {
        let dir = tempdir();
        let root = dir.path();
        // Create a symlink pointing outside the root
        let link = root.join("escape");
        std::os::unix::fs::symlink("/etc", &link).unwrap();
        // Attempt access via the key — canonicalize should detect the escape
        let result = resolve_path(root, "escape");
        // If the symlink points to /etc, the canonicalized path will be /etc,
        // which does not start with root -> PathEscape
        // (if /etc does not exist in the sandbox, the error is Io, also correct)
        assert!(matches!(
            result,
            Err(StorageError::PathEscape(_)) | Err(StorageError::Io(_))
        ));
    }

    // --- Functional tests ---

    fn test_config(root: &std::path::Path) -> crate::StorageConfig {
        crate::StorageConfig {
            root_path: root.to_path_buf(),
            max_object_size_mb: 10,
            ..crate::StorageConfig::default()
        }
    }

    #[tokio::test]
    async fn put_get_roundtrip() {
        let dir = tempdir();
        let cfg = test_config(dir.path());
        let store = AgStore::new(&cfg).unwrap();
        let data = Bytes::from("contenido de prueba");
        store.put("docs/test.txt", data.clone()).await.unwrap();
        let result = store.get("docs/test.txt").await.unwrap();
        assert_eq!(result, data);
    }

    #[tokio::test]
    async fn get_not_found_returns_error() {
        let dir = tempdir();
        let cfg = test_config(dir.path());
        let store = AgStore::new(&cfg).unwrap();
        let result = store.get("no-existe.txt").await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }

    #[tokio::test]
    async fn delete_removes_object() {
        let dir = tempdir();
        let cfg = test_config(dir.path());
        let store = AgStore::new(&cfg).unwrap();
        store.put("temp.txt", Bytes::from("x")).await.unwrap();
        store.delete("temp.txt").await.unwrap();
        assert!(!store.exists("temp.txt").await.unwrap());
    }

    #[tokio::test]
    async fn exists_returns_false_for_missing() {
        let dir = tempdir();
        let cfg = test_config(dir.path());
        let store = AgStore::new(&cfg).unwrap();
        assert!(!store.exists("ghost.txt").await.unwrap());
    }

    #[tokio::test]
    async fn oversized_upload_is_rejected() {
        let dir = tempdir();
        let mut cfg = test_config(dir.path());
        cfg.max_object_size_mb = 0; // 0 MB = limit is 0 bytes, any payload fails
        let store = AgStore::new(&cfg).unwrap();
        let data = Bytes::from(vec![0u8; 1]);
        let result = store.put("big.bin", data).await;
        assert!(matches!(result, Err(StorageError::TooLarge { .. })));
    }

    #[tokio::test]
    async fn list_returns_keys_by_prefix() {
        let dir = tempdir();
        let cfg = test_config(dir.path());
        let store = AgStore::new(&cfg).unwrap();
        store
            .put("avatars/alice.jpg", Bytes::from("a"))
            .await
            .unwrap();
        store
            .put("avatars/bob.jpg", Bytes::from("b"))
            .await
            .unwrap();
        store
            .put("docs/readme.txt", Bytes::from("c"))
            .await
            .unwrap();

        let all = store.list(None).await.unwrap();
        assert_eq!(all.len(), 3);

        let avatars = store.list(Some("avatars")).await.unwrap();
        assert_eq!(avatars.len(), 2);
        assert!(avatars.iter().all(|k| k.starts_with("avatars/")));
    }

    #[tokio::test]
    async fn copy_duplicates_object() {
        let dir = tempdir();
        let cfg = test_config(dir.path());
        let store = AgStore::new(&cfg).unwrap();
        let data = Bytes::from("original");
        store.put("original.txt", data.clone()).await.unwrap();
        store
            .copy("original.txt", "backup/original.txt")
            .await
            .unwrap();

        assert_eq!(store.get("original.txt").await.unwrap(), data);
        assert_eq!(store.get("backup/original.txt").await.unwrap(), data);
    }
}
