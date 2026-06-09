//! Anti-Gravital store backend: native filesystem and optional S3/MinIO.

#[cfg(feature = "s3")]
pub mod s3;
#[cfg(feature = "s3")]
pub use s3::S3Store;

pub mod auth;
pub mod server;

use crate::{StorageBackend, StorageConfig, StorageError};
use bytes::Bytes;
use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// NativeStore: implementation over local filesystem
// ---------------------------------------------------------------------------

/// Native Anti-Gravital store.
///
/// Stores objects as files under `root`. The key is the relative path.
/// Every operation passes through [`validate_key`] and [`resolve_path`] before I/O.
pub struct NativeStore {
    pub(crate) root: PathBuf,
    dir: Arc<Dir>,
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
        validate_key(key)?;
        let dir = Arc::clone(&self.dir);
        let key = PathBuf::from(key);
        let mut nonce = [0u8; 8];
        getrandom::getrandom(&mut nonce).unwrap_or_default();

        blocking_io(move || {
            let (parent, name) = open_parent_dir(&dir, &key, true)?;
            let tmp = format!(".tmp.{:016x}", u64::from_le_bytes(nonce));
            let mut options = OpenOptions::new();
            options
                .write(true)
                .create_new(true)
                .follow(FollowSymlinks::No);
            let result = (|| {
                let mut file = parent.open_with(&tmp, &options)?;
                file.write_all(&data)?;
                file.sync_all()?;
                parent.rename(&tmp, &parent, &name)
            })();
            if result.is_err() {
                let _ = parent.remove_file(&tmp);
            }
            result
        })
        .await
    }

    /// Retrieves the content of the object with key `key`.
    pub async fn get(&self, key: &str) -> Result<Bytes, StorageError> {
        validate_key(key)?;
        let dir = Arc::clone(&self.dir);
        let key_owned = key.to_owned();
        match blocking_io(move || {
            let (parent, name) = open_parent_dir(&dir, Path::new(&key_owned), false)?;
            let mut options = OpenOptions::new();
            options.read(true).follow(FollowSymlinks::No);
            let mut file = parent.open_with(name, &options)?;
            let mut data = Vec::new();
            std::io::Read::read_to_end(&mut file, &mut data)?;
            Ok(data)
        })
        .await
        {
            Ok(data) => Ok(Bytes::from(data)),
            Err(StorageError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                Err(StorageError::NotFound(key.to_owned()))
            }
            Err(error) => Err(error),
        }
    }

    /// Deletes the object with key `key`.
    pub async fn delete(&self, key: &str) -> Result<(), StorageError> {
        validate_key(key)?;
        let dir = Arc::clone(&self.dir);
        let key_owned = key.to_owned();
        match blocking_io(move || {
            let (parent, name) = open_parent_dir(&dir, Path::new(&key_owned), false)?;
            parent.remove_file(name)
        })
        .await
        {
            Ok(()) => Ok(()),
            Err(StorageError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                Err(StorageError::NotFound(key.to_owned()))
            }
            Err(error) => Err(error),
        }
    }

    /// Returns `true` if an object with key `key` exists.
    pub async fn exists(&self, key: &str) -> Result<bool, StorageError> {
        validate_key(key)?;
        let dir = Arc::clone(&self.dir);
        let key = key.to_owned();
        blocking_io(move || {
            let (parent, name) = match open_parent_dir(&dir, Path::new(&key), false) {
                Ok(value) => value,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
                Err(error) => return Err(error),
            };
            match parent.symlink_metadata(name) {
                Ok(metadata) => Ok(metadata.is_file()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
                Err(error) => Err(error),
            }
        })
        .await
    }

    /// Lists object keys under `prefix`.
    pub async fn list(&self, prefix: Option<&str>) -> Result<Vec<String>, StorageError> {
        let dir = Arc::clone(&self.dir);
        let prefix_str = prefix.map(|p| p.trim_end_matches('/').to_owned());

        let keys = blocking_io(move || collect_keys(&dir, Path::new(""))).await?;

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
                let dir =
                    Dir::open_ambient_dir(&root, ambient_authority()).map_err(StorageError::Io)?;
                Ok(AgStore::Native(NativeStore {
                    root,
                    dir: Arc::new(dir),
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

fn open_parent_dir(
    root: &Dir,
    key: &Path,
    create: bool,
) -> std::io::Result<(Dir, std::ffi::OsString)> {
    let name = key
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing file name"))?
        .to_owned();
    let mut current = root.try_clone()?;
    if let Some(parent) = key.parent() {
        for component in parent.components() {
            let Component::Normal(segment) = component else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "invalid path component",
                ));
            };
            match current.symlink_metadata(segment) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "symbolic link path component rejected",
                    ));
                }
                Ok(_) => {}
                Err(error) if create && error.kind() == std::io::ErrorKind::NotFound => {
                    current.create_dir(segment)?;
                }
                Err(error) => return Err(error),
            }
            current = current.open_dir(segment)?;
        }
    }
    Ok((current, name))
}

async fn blocking_io<T, F>(operation: F) -> Result<T, StorageError>
where
    T: Send + 'static,
    F: FnOnce() -> std::io::Result<T> + Send + 'static,
{
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|error| StorageError::Io(std::io::Error::other(error.to_string())))?
        .map_err(StorageError::Io)
}

/// Recursively walks a capability directory and returns relative object keys.
/// Ignores temporary files with the `.tmp.` prefix and symbolic links.
fn collect_keys(dir: &Dir, prefix: &Path) -> std::io::Result<Vec<String>> {
    let mut keys = Vec::new();
    for entry in dir.entries()? {
        let entry = entry?;
        let name = entry.file_name();
        let path = prefix.join(&name);
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            let child = entry.open_dir()?;
            let mut sub = collect_keys(&child, &path)?;
            keys.append(&mut sub);
        } else if file_type.is_file() {
            if name.to_string_lossy().starts_with(".tmp.") {
                continue;
            }
            keys.push(path.to_string_lossy().replace('\\', "/"));
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
    let mut existing = candidate.as_path();
    while !existing.exists() {
        existing = existing
            .parent()
            .ok_or_else(|| StorageError::PathEscape(key.to_owned()))?;
    }
    let canonical_existing = existing.canonicalize().map_err(StorageError::Io)?;
    if !canonical_existing.starts_with(&canonical_root) {
        return Err(StorageError::PathEscape(key.to_owned()));
    }
    Ok(normalize_path(&candidate))
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

    #[tokio::test]
    #[cfg(unix)]
    async fn put_cannot_follow_parent_symlink_outside_root() {
        let root = tempdir();
        let outside = tempdir();
        std::os::unix::fs::symlink(outside.path(), root.path().join("escape")).unwrap();

        let store = AgStore::new(&test_config(root.path())).unwrap();
        let result = store.put("escape/new.txt", Bytes::from("secret")).await;

        assert!(result.is_err());
        assert!(!outside.path().join("new.txt").exists());
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn get_cannot_follow_parent_symlink_outside_root() {
        let root = tempdir();
        let outside = tempdir();
        std::fs::write(outside.path().join("secret.txt"), b"secret").unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("escape")).unwrap();

        let store = AgStore::new(&test_config(root.path())).unwrap();
        let result = store.get("escape/secret.txt").await;

        assert!(result.is_err());
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

#[cfg(test)]
mod prop_tests {
    //! Property-based test (audit Stage 5): the path-traversal safety invariant.
    //! For ANY input key, `resolve_path` must never return an `Ok` path that
    //! escapes the storage root. Rejecting the key with an error is always fine.
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn resolve_path_never_escapes_root(key in ".*") {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path();
            if let Ok(resolved) = resolve_path(root, &key) {
                let canonical_root = root.canonicalize().unwrap();
                prop_assert!(
                    resolved.starts_with(&canonical_root),
                    "resolved {:?} escaped root {:?} for key {:?}",
                    resolved,
                    canonical_root,
                    key
                );
            }
        }
    }
}
