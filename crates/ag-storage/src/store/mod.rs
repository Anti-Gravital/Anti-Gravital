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
    // TECH-DEBT:
    // motivo: usado en Task 5 para rechazar payloads grandes.
    // impacto: sin efecto hasta Task 5.
    // eliminacion esperada: Task 5 ag-storage.
    #[allow(dead_code)]
    max_object_size: usize,
}

impl AgStore {
    /// Crea el store, creando `root_path` si no existe.
    pub fn new(config: &StorageConfig) -> Result<Self, StorageError> {
        std::fs::create_dir_all(&config.root_path).map_err(StorageError::Io)?;
        let root = config
            .root_path
            .canonicalize()
            .map_err(StorageError::Io)?;
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
        // TECH-DEBT:
        // motivo: implementacion completa en Task 5 (write-then-rename atomico).
        // impacto: put no funcional hasta Task 5.
        // eliminacion esperada: Task 5 ag-storage.
        let _ = (key, data);
        todo!()
    }

    /// Recupera el contenido del objeto con clave `key`.
    pub async fn get(&self, key: &str) -> Result<Bytes, StorageError> {
        // TECH-DEBT:
        // motivo: implementacion completa en Task 5.
        // impacto: get no funcional hasta Task 5.
        // eliminacion esperada: Task 5 ag-storage.
        let _ = key;
        todo!()
    }

    /// Borra el objeto con clave `key`.
    pub async fn delete(&self, key: &str) -> Result<(), StorageError> {
        // TECH-DEBT:
        // motivo: implementacion completa en Task 5.
        // impacto: delete no funcional hasta Task 5.
        // eliminacion esperada: Task 5 ag-storage.
        let _ = key;
        todo!()
    }

    /// Retorna `true` si existe un objeto con clave `key`.
    pub async fn exists(&self, key: &str) -> Result<bool, StorageError> {
        // TECH-DEBT:
        // motivo: implementacion completa en Task 5.
        // impacto: exists no funcional hasta Task 5.
        // eliminacion esperada: Task 5 ag-storage.
        let _ = key;
        todo!()
    }

    /// Lista las claves de objetos bajo `prefix`.
    pub async fn list(&self, prefix: Option<&str>) -> Result<Vec<String>, StorageError> {
        // TECH-DEBT:
        // motivo: implementacion completa en Task 6.
        // impacto: list no funcional hasta Task 6.
        // eliminacion esperada: Task 6 ag-storage.
        let _ = prefix;
        todo!()
    }

    /// Copia el objeto `from` a la clave `to`.
    pub async fn copy(&self, from: &str, to: &str) -> Result<(), StorageError> {
        // TECH-DEBT:
        // motivo: implementacion completa en Task 6.
        // impacto: copy no funcional hasta Task 6.
        // eliminacion esperada: Task 6 ag-storage.
        let _ = (from, to);
        todo!()
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
        return Err(StorageError::InvalidKey(
            "clave supera 1024 bytes".into(),
        ));
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
            return Err(StorageError::InvalidKey(
                "clave contiene byte nulo".into(),
            ));
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
}
