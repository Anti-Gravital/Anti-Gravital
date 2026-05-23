//! Backend nativo del store — operaciones sobre filesystem local.

pub mod auth;
pub mod server;

use crate::{StorageConfig, StorageError};
use bytes::Bytes;
use std::path::{Component, Path, PathBuf};

/// Store nativo Anti-Gravital.
///
/// Almacena objetos como archivos bajo `root`. La clave es la ruta relativa.
/// Toda operacion pasa por [`validate_key`] y [`resolve_path`] antes de I/O.
pub struct AgStore {
    root: PathBuf,
    max_object_size: usize,
}

impl AgStore {
    /// Crea el store, creando `root_path` si no existe.
    pub fn new(config: &StorageConfig) -> Result<Self, StorageError> {
        std::fs::create_dir_all(&config.root_path).map_err(StorageError::Io)?;
        let root = config.root_path.canonicalize().map_err(StorageError::Io)?;
        Ok(Self {
            root,
            max_object_size: config.max_object_size_mb as usize * 1024 * 1024,
        })
    }

    /// Directorio raiz del store (canonicalizado).
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Almacena `data` bajo la clave `key`.
    ///
    /// Usa write-then-atomic-rename para evitar lecturas de archivos parciales.
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
        // write-then-rename atomico: evita lecturas de archivos parcialmente escritos
        let mut nonce = [0u8; 8];
        getrandom::getrandom(&mut nonce).unwrap_or_default();
        let tmp = dest.with_file_name(format!(".tmp.{:016x}", u64::from_le_bytes(nonce)));
        tokio::fs::write(&tmp, &data).await?;
        tokio::fs::rename(&tmp, &dest).await?;
        Ok(())
    }

    /// Recupera el contenido del objeto con clave `key`.
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

    /// Borra el objeto con clave `key`.
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

    /// Retorna `true` si existe un objeto con clave `key`.
    pub async fn exists(&self, key: &str) -> Result<bool, StorageError> {
        let path = resolve_path(&self.root, key)?;
        Ok(tokio::fs::try_exists(&path).await.unwrap_or(false))
    }

    /// Lista las claves de objetos bajo `prefix`.
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

    /// Copia el objeto `from` a la clave `to`.
    pub async fn copy(&self, from: &str, to: &str) -> Result<(), StorageError> {
        let data = self.get(from).await?;
        self.put(to, data).await
    }
}

// ---------------------------------------------------------------------------
// Seguridad: validacion de clave y confinamiento de path
// ---------------------------------------------------------------------------

/// Valida que `key` sea una clave de objeto segura.
///
/// Rechaza con [`StorageError::InvalidKey`] si la clave:
/// - Esta vacia o supera 1024 bytes.
/// - Contiene bytes nulos o caracteres de control (< 0x20, excepto `/`).
/// - Contiene segmentos `.` o `..`.
/// - Empieza o termina con `/`.
/// - Contiene secuencias `//`.
pub fn validate_key(key: &str) -> Result<(), StorageError> {
    if key.is_empty() {
        return Err(StorageError::InvalidKey("clave vacia".into()));
    }
    if key.len() > 1024 {
        return Err(StorageError::InvalidKey("clave supera 1024 bytes".into()));
    }
    if key.starts_with('/') || key.ends_with('/') {
        return Err(StorageError::InvalidKey(
            "clave no puede empezar ni terminar con '/'".into(),
        ));
    }
    if key.contains("//") {
        return Err(StorageError::InvalidKey(
            "clave no puede contener '//'".into(),
        ));
    }
    for byte in key.bytes() {
        if byte == 0 {
            return Err(StorageError::InvalidKey("clave contiene byte nulo".into()));
        }
        if byte < 0x20 && byte != b'/' {
            return Err(StorageError::InvalidKey(
                "clave contiene caracter de control".into(),
            ));
        }
    }
    for segment in key.split('/') {
        if segment == "." || segment == ".." {
            return Err(StorageError::InvalidKey(format!(
                "segmento de path prohibido: '{segment}'"
            )));
        }
    }
    Ok(())
}

/// Normaliza un path sin tocar disco (resuelve `.` y `..` lexicamente).
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

/// Recorre `dir` recursivamente y retorna rutas relativas a `root`.
/// Ignora archivos temporales con prefijo `.tmp.`.
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

/// Resuelve `key` a un path absoluto dentro de `root`.
///
/// Garantiza que el path resultante este dentro de `root`.
/// Retorna [`StorageError::PathEscape`] si el path resuelto escapa del root.
pub fn resolve_path(root: &Path, key: &str) -> Result<PathBuf, StorageError> {
    validate_key(key)?;
    let canonical_root = root.canonicalize().map_err(StorageError::Io)?;
    let candidate = canonical_root.join(key);
    let resolved = if candidate.exists() {
        candidate.canonicalize().map_err(StorageError::Io)?
    } else {
        // Seguro porque validate_key (llamada arriba) ya rechazo todos los
        // segmentos '.' y '..'. normalize_path es solo defensa adicional.
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

    // --- Tests de seguridad ---

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
    fn symlink_escape_is_blocked() {
        let dir = tempdir();
        let root = dir.path();
        // Crear un symlink que apunta fuera del root
        let link = root.join("escape");
        std::os::unix::fs::symlink("/etc", &link).unwrap();
        // Intentar acceder via la clave — canonicalize deberia detectar el escape
        let result = resolve_path(root, "escape");
        // Si el symlink apunta a /etc, el path canonicalizado sera /etc,
        // que no empieza con root -> PathEscape
        // (si /etc no existe en el sandbox, el error es Io, tambien correcto)
        assert!(matches!(
            result,
            Err(StorageError::PathEscape(_)) | Err(StorageError::Io(_))
        ));
    }

    // --- Tests funcionales ---

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
