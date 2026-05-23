# ag-storage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implementar `ag-storage` — store nativo Anti-Gravital basado en filesystem, con servidor HTTP Axum embebido, seguridad por construccion (path confinement, key validation, write-then-rename), procesamiento de imagen y adaptadores S3/MinIO opcionales via feature flag.

**Architecture:** `AgStore` almacena objetos en disco usando la clave como ruta relativa bajo un directorio raiz configurable. `AgStorage` es la facade publica que orquesta el store, el servidor HTTP opcional y el `ImageProcessor`. Toda clave entrante pasa por `validate_key` + `resolve_path` antes de cualquier I/O. El servidor HTTP Axum corre en background cuando `server_mode = true`.

**Tech Stack:** Rust 1.79, axum 0.7, tower-http 0.6, governor 0.7 (rate limiting), image 0.25 (JPEG/PNG/WebP), thiserror 1, bytes 1, getrandom 0.2, tokio 1.

---

## Mapa de archivos

| Archivo | Accion | Responsabilidad |
|---|---|---|
| `Cargo.toml` (workspace) | Modificar | Agregar `image 0.25` como dep workspace |
| `crates/ag-storage/Cargo.toml` | Modificar | Deps base + features `auth` y `s3` |
| `crates/ag-storage/src/lib.rs` | Crear | `AgStorage` facade, `StorageError`, `Permission` |
| `crates/ag-storage/src/config.rs` | Crear | `StorageConfig`, `StorageBackend` |
| `crates/ag-storage/src/store/mod.rs` | Crear | `AgStore` + `validate_key` + `resolve_path` + todas las ops |
| `crates/ag-storage/src/store/auth.rs` | Crear | Middleware Bearer token para Axum |
| `crates/ag-storage/src/store/server.rs` | Crear | Router Axum, handlers, rate limiting |
| `crates/ag-storage/src/image.rs` | Crear | `ImageProcessor` (resize/thumbnail/webp) |
| `crates/ag-storage/README.md` | Modificar | Actualizar estado y usage |

---

## Task 1: Cargo.toml — dependencias y features

**Files:**
- Modify: `Cargo.toml` (workspace)
- Modify: `crates/ag-storage/Cargo.toml`

- [ ] **Step 1: Agregar `image` a las dependencias del workspace**

En `Cargo.toml` (raiz), dentro de `[workspace.dependencies]`, agregar despues de la linea `moka = ...`:

```toml
image = { version = "0.25", default-features = false, features = ["jpeg", "png", "webp"] }
```

- [ ] **Step 2: Reemplazar el contenido de `crates/ag-storage/Cargo.toml`**

```toml
[package]
name = "ag-storage"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
homepage.workspace = true
documentation.workspace = true
readme = "README.md"
description = "Store nativo Anti-Gravital: filesystem, servidor HTTP, S3/MinIO, procesamiento de imagen"
keywords.workspace = true
categories.workspace = true
publish = false

[lints]
workspace = true

[features]
default = []
auth = ["dep:ag-auth"]
s3   = ["dep:object_store"]

[dependencies]
ag-auth       = { path = "../ag-auth", optional = true }
axum          = { workspace = true }
bytes         = { workspace = true }
getrandom     = { workspace = true }
governor      = { workspace = true }
http          = { workspace = true }
image         = { workspace = true }
serde         = { workspace = true }
serde_json    = { workspace = true }
thiserror     = { workspace = true }
tokio         = { workspace = true }
tower         = { workspace = true }
tower-http    = { workspace = true }
tracing       = { workspace = true }
object_store  = { version = "0.11", features = ["aws"], optional = true }

[dev-dependencies]
tokio = { workspace = true, features = ["test-util"] }
```

- [ ] **Step 3: Verificar que el workspace compila con los cambios**

```bash
cargo check -p ag-storage
```

Resultado esperado: `Finished` sin errores (el crate solo tiene el stub lib.rs por ahora).

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml crates/ag-storage/Cargo.toml Cargo.lock
git commit -m "chore(storage): Cargo.toml crate y deps workspace — image, governor, getrandom"
```

---

## Task 2: StorageError — todos los variantes

**Files:**
- Create: `crates/ag-storage/src/lib.rs`

- [ ] **Step 1: Escribir el test que verifica Display de errores**

En `crates/ag-storage/src/lib.rs` reemplazar el contenido existente con:

```rust
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
    #[error("configuracion invalida: {0}")]
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
```

- [ ] **Step 2: Verificar que compila (los modulos no existen aun — se esperan errores de modulo)**

```bash
cargo check -p ag-storage 2>&1 | grep "error\[" | head -10
```

Resultado esperado: errores `module not found` para `config`, `image`, `store` — normal porque los modulos no existen todavia.

- [ ] **Step 3: Verificar que StorageError implementa Display correctamente — test inline**

Al final de `crates/ag-storage/src/lib.rs`, agregar:

```rust
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
```

No se pueden correr todavia (dep de modulos pendientes). Continuar al Task 3.

---

## Task 3: config.rs — StorageConfig y StorageBackend

**Files:**
- Create: `crates/ag-storage/src/config.rs`

- [ ] **Step 1: Escribir los tests primero**

Crear `crates/ag-storage/src/config.rs` con este contenido completo:

```rust
//! Configuracion del store de almacenamiento.

use std::path::PathBuf;

/// Backend de almacenamiento activo.
#[derive(Debug, Clone, PartialEq)]
pub enum StorageBackend {
    /// Filesystem local (default). Sin dependencias externas.
    Native,
    #[cfg(feature = "s3")]
    /// AWS S3 o compatible.
    S3,
    #[cfg(feature = "s3")]
    /// MinIO self-hosted (S3-compatible).
    MinIO,
}

/// Configuracion del subsistema de almacenamiento.
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Backend activo.
    pub backend: StorageBackend,
    /// Directorio raiz del store nativo.
    pub root_path: PathBuf,
    /// Si `true`, levanta un servidor HTTP Axum en background.
    pub server_mode: bool,
    /// Puerto del servidor HTTP. Default: 4280.
    pub server_port: u16,
    /// Token Bearer estatico. Vacio = sin autenticacion (modo dev).
    pub store_token: String,
    /// Tamano maximo de objeto en MB. Default: 100.
    pub max_object_size_mb: u64,
    /// Limite de requests por segundo del servidor HTTP. Default: 100.
    pub rate_limit_rps: u32,
    // Campos S3/MinIO (ignorados en backend Native)
    /// Region AWS. Default: "us-east-1".
    pub region: String,
    /// Endpoint personalizado (para MinIO). None = AWS.
    pub endpoint: Option<String>,
    /// Access key AWS.
    pub access_key: Option<String>,
    /// Secret key AWS.
    pub secret_key: Option<String>,
    /// Nombre del bucket S3/MinIO. Default: "ag-storage".
    pub bucket: String,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::Native,
            root_path: PathBuf::from("./ag-store-data"),
            server_mode: false,
            server_port: 4280,
            store_token: String::new(),
            max_object_size_mb: 100,
            rate_limit_rps: 100,
            region: "us-east-1".into(),
            endpoint: None,
            access_key: None,
            secret_key: None,
            bucket: "ag-storage".into(),
        }
    }
}

impl StorageConfig {
    /// Lee la configuracion desde variables de entorno.
    /// Valores no definidos usan los defaults de [`StorageConfig::default`].
    pub fn from_env() -> Self {
        let backend = match std::env::var("STORAGE_BACKEND")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "s3" => {
                #[cfg(feature = "s3")]
                { StorageBackend::S3 }
                #[cfg(not(feature = "s3"))]
                {
                    tracing::warn!("STORAGE_BACKEND=s3 pero feature s3 no esta activa; usando Native");
                    StorageBackend::Native
                }
            }
            "minio" => {
                #[cfg(feature = "s3")]
                { StorageBackend::MinIO }
                #[cfg(not(feature = "s3"))]
                {
                    tracing::warn!("STORAGE_BACKEND=minio pero feature s3 no esta activa; usando Native");
                    StorageBackend::Native
                }
            }
            _ => StorageBackend::Native,
        };

        Self {
            backend,
            root_path: std::env::var("STORAGE_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("./ag-store-data")),
            server_mode: std::env::var("STORAGE_SERVER")
                .map(|v| v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            server_port: std::env::var("STORAGE_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4280),
            store_token: std::env::var("STORE_TOKEN").unwrap_or_default(),
            max_object_size_mb: std::env::var("STORAGE_MAX_OBJECT_SIZE_MB")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            rate_limit_rps: std::env::var("STORAGE_RATE_LIMIT_RPS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            region: std::env::var("STORAGE_REGION").unwrap_or_else(|_| "us-east-1".into()),
            endpoint: std::env::var("STORAGE_ENDPOINT").ok(),
            access_key: std::env::var("AWS_ACCESS_KEY_ID").ok(),
            secret_key: std::env::var("AWS_SECRET_ACCESS_KEY").ok(),
            bucket: std::env::var("STORAGE_BUCKET").unwrap_or_else(|_| "ag-storage".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_are_native() {
        let cfg = StorageConfig::default();
        assert_eq!(cfg.backend, StorageBackend::Native);
        assert_eq!(cfg.root_path, PathBuf::from("./ag-store-data"));
    }

    #[test]
    fn config_server_mode_off_by_default() {
        let cfg = StorageConfig::default();
        assert!(!cfg.server_mode);
        assert_eq!(cfg.server_port, 4280);
    }

    #[test]
    fn config_from_env_reads_port() {
        std::env::set_var("STORAGE_PORT", "9000");
        let cfg = StorageConfig::from_env();
        std::env::remove_var("STORAGE_PORT");
        assert_eq!(cfg.server_port, 9000);
    }
}
```

- [ ] **Step 2: Correr los tests de config (los otros modulos fallan todavia)**

```bash
cargo test -p ag-storage config:: 2>&1 | tail -15
```

Resultado esperado: los 3 tests de config pasan, posibles errores de compilacion por modulos pendientes en lib.rs.

Si hay errores de modulos faltantes, agregar stubs temporales en lib.rs reemplazando los `pub mod` con declaraciones vacias antes de correr:

```bash
# Si falla por modulos: agregar temporalmente en lib.rs
# pub mod image {}
# pub mod store {}
# y volver a correr
```

- [ ] **Step 3: Commit**

```bash
git add crates/ag-storage/src/lib.rs crates/ag-storage/src/config.rs
git commit -m "feat(storage): StorageConfig from_env, StorageError — 3 tests config"
```

---

## Task 4: store/mod.rs — seguridad: validate_key + resolve_path

**Files:**
- Create: `crates/ag-storage/src/store/mod.rs`

- [ ] **Step 1: Crear el directorio y escribir los tests de seguridad primero**

```bash
mkdir -p crates/ag-storage/src/store
```

Crear `crates/ag-storage/src/store/mod.rs` con el siguiente contenido:

```rust
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
        // (si /etc no existe en el sandbox, el error es Io, lo que tambien es correcto)
        assert!(matches!(
            result,
            Err(StorageError::PathEscape(_)) | Err(StorageError::Io(_))
        ));
    }

    fn tempdir() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }
}
```

- [ ] **Step 2: Agregar `tempfile` como dev-dependency**

En `crates/ag-storage/Cargo.toml`, agregar en `[dev-dependencies]`:

```toml
tempfile = "3"
```

Tambien agregar en `[workspace.dependencies]` del `Cargo.toml` raiz:

```toml
tempfile = "3"
```

- [ ] **Step 3: Crear stubs vacios para auth.rs y server.rs (necesarios para compilar)**

Crear `crates/ag-storage/src/store/auth.rs`:

```rust
//! Middleware de autenticacion Bearer token para el servidor HTTP.
// Implementacion completa en Task 7.
```

Crear `crates/ag-storage/src/store/server.rs`:

```rust
//! Servidor HTTP Axum embebido del store.
// Implementacion completa en Task 8.

use crate::{StorageConfig, StorageError};
use std::sync::Arc;
use super::AgStore;

pub async fn run_server(
    _store: Arc<AgStore>,
    _config: &StorageConfig,
) -> Result<(), StorageError> {
    Ok(()) // stub
}
```

- [ ] **Step 4: Actualizar lib.rs para referenciar store correctamente**

En `crates/ag-storage/src/lib.rs` asegurarse de que las `pub mod` sean:

```rust
pub mod config;
pub mod image;
pub mod store;
```

Y que `AgStore` se re-exporte correctamente:

```rust
pub use store::AgStore;
```

Tambien crear el stub de `image.rs`:

Crear `crates/ag-storage/src/image.rs` con stub temporal:

```rust
//! Procesamiento de imagen. Implementacion completa en Task 9.

/// Procesador de imagenes Anti-Gravital.
pub struct ImageProcessor;

impl ImageProcessor {
    pub fn new() -> Self { Self }
}
```

- [ ] **Step 5: Correr los tests de seguridad**

```bash
cargo test -p ag-storage store::tests 2>&1 | tail -20
```

Resultado esperado:
```
test store::tests::key_with_dotdot_is_rejected ... ok
test store::tests::key_with_null_byte_is_rejected ... ok
test store::tests::key_starting_with_slash_is_rejected ... ok
test store::tests::valid_keys_are_accepted ... ok
test store::tests::symlink_escape_is_blocked ... ok
```

- [ ] **Step 6: Commit**

```bash
git add crates/ag-storage/src/store/ crates/ag-storage/src/image.rs crates/ag-storage/src/lib.rs Cargo.toml Cargo.lock
git commit -m "feat(storage): validate_key + resolve_path — seguridad path confinement, 5 tests"
```

---

## Task 5: AgStore — operaciones de disco

**Files:**
- Modify: `crates/ag-storage/src/store/mod.rs`

- [ ] **Step 1: Escribir los tests de operaciones antes de implementar**

Al final del bloque `#[cfg(test)]` en `store/mod.rs`, agregar despues del test `valid_keys_are_accepted`:

```rust
    // --- Tests funcionales ---

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
        cfg.max_object_size_mb = 0; // 0 MB = cualquier upload falla
        let store = AgStore::new(&cfg).unwrap();
        let big = Bytes::from(vec![0u8; 1]);
        let result = store.put("big.bin", big).await;
        assert!(matches!(result, Err(StorageError::TooLarge { .. })));
    }

    fn test_config(root: &Path) -> StorageConfig {
        use crate::config::StorageBackend;
        StorageConfig {
            backend: StorageBackend::Native,
            root_path: root.to_path_buf(),
            max_object_size_mb: 10,
            ..StorageConfig::default()
        }
    }
```

- [ ] **Step 2: Implementar put, get, delete, exists en AgStore**

En `store/mod.rs`, despues de `impl AgStore { pub fn new(...) { ... } pub fn root(...) { ... } }`, agregar:

```rust
impl AgStore {
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
        let tmp = {
            let mut nonce = [0u8; 8];
            getrandom::getrandom(&mut nonce).unwrap_or_default();
            dest.with_file_name(format!(
                ".tmp.{:016x}",
                u64::from_le_bytes(nonce)
            ))
        };
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
}
```

Agregar `use getrandom;` al inicio del modulo (despues de los otros `use`):

```rust
use getrandom;
```

- [ ] **Step 3: Correr los tests de operaciones**

```bash
cargo test -p ag-storage store::tests 2>&1 | tail -25
```

Resultado esperado:
```
test store::tests::put_get_roundtrip ... ok
test store::tests::get_not_found_returns_error ... ok
test store::tests::delete_removes_object ... ok
test store::tests::exists_returns_false_for_missing ... ok
test store::tests::oversized_upload_is_rejected ... ok
```

- [ ] **Step 4: Commit**

```bash
git add crates/ag-storage/src/store/mod.rs Cargo.lock Cargo.toml
git commit -m "feat(storage): AgStore put/get/delete/exists — write-then-rename atomico, 5 tests"
```

---

## Task 6: AgStore — list y copy

**Files:**
- Modify: `crates/ag-storage/src/store/mod.rs`

- [ ] **Step 1: Escribir los tests de list y copy**

En el bloque `#[cfg(test)]` de `store/mod.rs`, agregar:

```rust
    #[tokio::test]
    async fn list_returns_keys_by_prefix() {
        let dir = tempdir();
        let cfg = test_config(dir.path());
        let store = AgStore::new(&cfg).unwrap();
        store.put("avatars/alice.jpg", Bytes::from("a")).await.unwrap();
        store.put("avatars/bob.jpg", Bytes::from("b")).await.unwrap();
        store.put("docs/readme.txt", Bytes::from("c")).await.unwrap();

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
        store.copy("original.txt", "backup/original.txt").await.unwrap();

        // original intacto
        assert_eq!(store.get("original.txt").await.unwrap(), data);
        // copia existe
        assert_eq!(store.get("backup/original.txt").await.unwrap(), data);
    }
```

- [ ] **Step 2: Implementar list y copy en AgStore**

Dentro del segundo `impl AgStore { ... }` (el que tiene put/get/delete/exists), agregar:

```rust
    /// Lista las claves de objetos bajo `prefix`.
    ///
    /// Si `prefix` es `None`, lista todas las claves del store.
    /// La busqueda es recursiva dentro del directorio raiz.
    pub async fn list(&self, prefix: Option<&str>) -> Result<Vec<String>, StorageError> {
        let root = self.root.clone();
        let prefix_str = prefix
            .map(|p| p.trim_end_matches('/').to_owned());

        let keys = tokio::task::spawn_blocking(move || collect_keys(&root, &root))
            .await
            .map_err(|e| {
                StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
            })??;

        Ok(match prefix_str {
            Some(p) => keys
                .into_iter()
                .filter(|k| k == &p || k.starts_with(&format!("{p}/")))
                .collect(),
            None => keys,
        })
    }

    /// Copia el objeto `from` a la clave `to`.
    ///
    /// Implementado como get + put para aprovechar la escritura atomica.
    pub async fn copy(&self, from: &str, to: &str) -> Result<(), StorageError> {
        let data = self.get(from).await?;
        self.put(to, data).await
    }
```

Agregar la funcion auxiliar `collect_keys` fuera del bloque `impl` (a nivel de modulo):

```rust
/// Recorre `dir` recursivamente y retorna rutas relativas a `root`.
fn collect_keys(dir: &Path, root: &Path) -> Result<Vec<String>, StorageError> {
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
            // ignorar archivos temporales (.tmp.*)
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
```

- [ ] **Step 3: Correr todos los tests del store**

```bash
cargo test -p ag-storage store::tests 2>&1 | tail -20
```

Resultado esperado: 12 tests pasan (5 seguridad + 5 ops + 2 nuevos).

- [ ] **Step 4: Commit**

```bash
git add crates/ag-storage/src/store/mod.rs
git commit -m "feat(storage): AgStore list/copy — busqueda recursiva prefijo, 2 tests"
```

---

## Task 7: store/auth.rs — middleware Bearer token

**Files:**
- Modify: `crates/ag-storage/src/store/auth.rs`

- [ ] **Step 1: Implementar el middleware**

Reemplazar el contenido de `crates/ag-storage/src/store/auth.rs`:

```rust
//! Middleware de autenticacion Bearer token para el servidor HTTP.
//!
//! Sin feature `auth`: compara el token contra `STORE_TOKEN` (string estatico).
//! Con feature `auth`: valida el Bearer como JWT Ed25519 via `ag-auth`.

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

/// Estado del middleware: el token estatico configurado.
///
/// Si el token esta vacio, el servidor acepta todo (modo desarrollo).
pub type AuthToken = Arc<String>;

/// Middleware que exige `Authorization: Bearer <token>`.
///
/// Permite el paso sin autenticacion si el token configurado esta vacio.
pub async fn bearer_auth_middleware(
    axum::extract::State(token): axum::extract::State<AuthToken>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if token.is_empty() {
        // modo dev: sin token configurado, todas las rutas son publicas
        return next.run(request).await;
    }

    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(h) if h.starts_with("Bearer ") && h[7..] == **token => {
            next.run(request).await
        }
        _ => StatusCode::UNAUTHORIZED.into_response(),
    }
}
```

- [ ] **Step 2: Verificar que compila**

```bash
cargo check -p ag-storage 2>&1 | grep "^error" | head -10
```

Resultado esperado: sin errores en `auth.rs`.

- [ ] **Step 3: Commit**

```bash
git add crates/ag-storage/src/store/auth.rs
git commit -m "feat(storage): middleware Bearer token — modo dev sin auth si STORE_TOKEN vacio"
```

---

## Task 8: store/server.rs — servidor HTTP Axum

**Files:**
- Modify: `crates/ag-storage/src/store/server.rs`

- [ ] **Step 1: Implementar el servidor completo**

Reemplazar el contenido de `crates/ag-storage/src/store/server.rs`:

```rust
//! Servidor HTTP Axum embebido del store Anti-Gravital.
//!
//! Se levanta en background cuando `StorageConfig::server_mode` es `true`.
//! Expone la API REST v1 del store con autenticacion Bearer token y
//! rate limiting global via governor.

use super::{auth::bearer_auth_middleware, AgStore};
use crate::{StorageConfig, StorageError};
use axum::{
    body::Body,
    extract::{Path, Query, Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{delete, get, head, post, put},
    Json, Router,
};
use bytes::Bytes;
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use serde::{Deserialize, Serialize};
use std::{num::NonZeroU32, sync::Arc};
use tower::ServiceBuilder;
use axum::extract::DefaultBodyLimit;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Arranca el servidor HTTP y escucha hasta que el proceso termina.
pub async fn run_server(
    store: Arc<AgStore>,
    config: &StorageConfig,
) -> Result<(), StorageError> {
    let addr = format!("0.0.0.0:{}", config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(StorageError::Io)?;
    tracing::info!(port = config.server_port, "ag-storage server escuchando");
    let app = build_router(store, config);
    axum::serve(listener, app)
        .await
        .map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Construye el router Axum con todas las rutas y middlewares.
///
/// Separado de `run_server` para facilitar tests sin bind de puerto real.
pub fn build_router(store: Arc<AgStore>, config: &StorageConfig) -> Router {
    let limiter: Arc<DefaultDirectRateLimiter> = Arc::new(RateLimiter::direct(
        Quota::per_second(NonZeroU32::new(config.rate_limit_rps).unwrap_or(NonZeroU32::MIN)),
    ));
    let token = Arc::new(config.store_token.clone());
    let max_body = config.max_object_size_mb as usize * 1024 * 1024;

    // Rutas protegidas: requieren auth + rate limit
    let protected = Router::new()
        .route("/v1/objects/", get(list_objects))
        .route("/v1/objects/*key", get(get_object))
        .route("/v1/objects/*key", put(put_object))
        .route("/v1/objects/*key", delete(delete_object))
        .route("/v1/objects/*key", head(head_object))
        .route("/v1/copy", post(copy_object))
        .with_state(store)
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn_with_state(
                    Arc::clone(&limiter),
                    rate_limit_middleware,
                ))
                .layer(axum::middleware::from_fn_with_state(
                    token,
                    bearer_auth_middleware,
                )),
        );

    // Rutas publicas (sin auth ni rate limit)
    let public = Router::new().route("/v1/health", get(health));

    Router::new()
        .merge(protected)
        .merge(public)
        .layer(DefaultBodyLimit::max(max_body))
}

// ---------------------------------------------------------------------------
// Rate limit middleware
// ---------------------------------------------------------------------------

async fn rate_limit_middleware(
    State(limiter): State<Arc<DefaultDirectRateLimiter>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if limiter.check().is_err() {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    next.run(request).await
}

// ---------------------------------------------------------------------------
// Content-Type seguro
// ---------------------------------------------------------------------------

/// Retorna (content_type, needs_attachment) para una clave de objeto.
///
/// Usa una lista positiva; cualquier extension desconocida recibe
/// `application/octet-stream` con `Content-Disposition: attachment`.
fn content_type_for(key: &str) -> (&'static str, bool) {
    let ext = std::path::Path::new(key)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "jpg" | "jpeg" => ("image/jpeg", false),
        "png" => ("image/png", false),
        "gif" => ("image/gif", false),
        "webp" => ("image/webp", false),
        "svg" => ("image/svg+xml", false),
        "ico" => ("image/x-icon", false),
        "txt" => ("text/plain; charset=utf-8", false),
        "html" | "htm" => ("text/html; charset=utf-8", false),
        "css" => ("text/css; charset=utf-8", false),
        "json" => ("application/json", false),
        "pdf" => ("application/pdf", false),
        "xml" => ("application/xml", false),
        _ => ("application/octet-stream", true),
    }
}

fn etag_for(data: &Bytes) -> String {
    let hash = blake3::hash(data);
    format!("\"{}\"", &hash.to_hex()[..16])
}

// ---------------------------------------------------------------------------
// Error de aplicacion del servidor
// ---------------------------------------------------------------------------

enum AppError {
    Storage(StorageError),
}

impl From<StorageError> for AppError {
    fn from(e: StorageError) -> Self {
        AppError::Storage(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Storage(StorageError::NotFound(_)) => {
                StatusCode::NOT_FOUND.into_response()
            }
            AppError::Storage(StorageError::InvalidKey(_))
            | AppError::Storage(StorageError::PathEscape(_)) => {
                StatusCode::BAD_REQUEST.into_response()
            }
            AppError::Storage(StorageError::TooLarge { .. }) => {
                StatusCode::PAYLOAD_TOO_LARGE.into_response()
            }
            AppError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn put_object(
    State(store): State<Arc<AgStore>>,
    Path(key): Path<String>,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    store.put(&key, body).await?;
    Ok(StatusCode::CREATED)
}

async fn get_object(
    State(store): State<Arc<AgStore>>,
    Path(key): Path<String>,
) -> Result<Response, AppError> {
    let data = store.get(&key).await?;
    let (ct, is_attachment) = content_type_for(&key);
    let etag = etag_for(&data);
    let len = data.len();
    let mut builder = Response::builder()
        .header(header::CONTENT_TYPE, ct)
        .header(header::CONTENT_LENGTH, len)
        .header("X-AG-Store-Key", &key)
        .header(header::ETAG, etag);
    if is_attachment {
        builder = builder.header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"",
                std::path::Path::new(&key)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            ),
        );
    }
    Ok(builder.body(Body::from(data)).unwrap())
}

async fn delete_object(
    State(store): State<Arc<AgStore>>,
    Path(key): Path<String>,
) -> Result<StatusCode, AppError> {
    store.delete(&key).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn head_object(
    State(store): State<Arc<AgStore>>,
    Path(key): Path<String>,
) -> Result<StatusCode, AppError> {
    if store.exists(&key).await? {
        Ok(StatusCode::OK)
    } else {
        Err(AppError::Storage(StorageError::NotFound(key)))
    }
}

#[derive(Deserialize)]
struct ListParams {
    prefix: Option<String>,
}

#[derive(Serialize)]
struct ListResponse {
    keys: Vec<String>,
    prefix: Option<String>,
    count: usize,
}

async fn list_objects(
    State(store): State<Arc<AgStore>>,
    Query(params): Query<ListParams>,
) -> Result<Json<ListResponse>, AppError> {
    let prefix = params.prefix.as_deref();
    let keys = store.list(prefix).await?;
    let count = keys.len();
    Ok(Json(ListResponse {
        keys,
        prefix: prefix.map(String::from),
        count,
    }))
}

#[derive(Deserialize)]
struct CopyParams {
    from: String,
    to: String,
}

async fn copy_object(
    State(store): State<Arc<AgStore>>,
    Query(params): Query<CopyParams>,
) -> Result<StatusCode, AppError> {
    store.copy(&params.from, &params.to).await?;
    Ok(StatusCode::CREATED)
}

// ---------------------------------------------------------------------------
// Tests del servidor
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn temp_store() -> (tempfile::TempDir, Arc<AgStore>) {
        let dir = tempfile::tempdir().unwrap();
        let cfg = StorageConfig {
            root_path: dir.path().to_path_buf(),
            ..StorageConfig::default()
        };
        let store = Arc::new(AgStore::new(&cfg).unwrap());
        (dir, store)
    }

    #[tokio::test]
    async fn server_health_check() {
        let (_dir, store) = temp_store();
        let config = StorageConfig::default();
        let app = build_router(store, &config);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn server_put_get_roundtrip() {
        let (_dir, store) = temp_store();
        let config = StorageConfig::default();
        let app = build_router(Arc::clone(&store), &config);

        // PUT
        let put_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/v1/objects/test/hello.txt")
                    .header(header::CONTENT_TYPE, "text/plain")
                    .body(Body::from("hola mundo"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(put_res.status(), StatusCode::CREATED);

        // GET
        let get_res = app
            .oneshot(
                Request::builder()
                    .uri("/v1/objects/test/hello.txt")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(get_res.status(), StatusCode::OK);
        assert_eq!(get_res.headers()["X-AG-Store-Key"], "test/hello.txt");
    }
}
```

- [ ] **Step 2: Agregar `blake3` a las deps de ag-storage**

En `crates/ag-storage/Cargo.toml`, agregar:

```toml
blake3 = { workspace = true }
```

- [ ] **Step 3: Correr los tests del servidor**

```bash
cargo test -p ag-storage store::server::tests 2>&1 | tail -15
```

Resultado esperado:
```
test store::server::tests::server_health_check ... ok
test store::server::tests::server_put_get_roundtrip ... ok
```

- [ ] **Step 4: Correr todos los tests hasta ahora**

```bash
cargo test -p ag-storage 2>&1 | tail -20
```

Resultado esperado: todos los tests pasan (no se especifica un numero exacto porque los modulos de imagen todavia tienen el stub).

- [ ] **Step 5: Commit**

```bash
git add crates/ag-storage/src/store/server.rs crates/ag-storage/Cargo.toml Cargo.lock
git commit -m "feat(storage): servidor HTTP Axum — PUT/GET/DELETE/HEAD/list/copy, rate limit, Content-Type seguro, 2 tests"
```

---

## Task 9: image.rs — ImageProcessor

**Files:**
- Modify: `crates/ag-storage/src/image.rs`

- [ ] **Step 1: Reemplazar el stub con la implementacion completa**

```rust
//! Procesamiento de imagenes para el ecosistema Anti-Gravital.
//!
//! Soporta JPEG, PNG y WebP. AVIF pendiente como TECH-DEBT.

use crate::StorageError;
use bytes::Bytes;
use image::{imageops::FilterType, DynamicImage, ImageFormat};
use std::io::Cursor;

/// Procesador de imagenes Anti-Gravital.
///
/// Obtener via [`crate::AgStorage::processor`].
pub struct ImageProcessor;

impl ImageProcessor {
    pub(crate) fn new() -> Self {
        Self
    }

    /// Redimensiona la imagen para que quepa dentro de `max_w x max_h`
    /// preservando el aspect ratio. Usa filtro Lanczos3 (alta calidad).
    pub fn resize(
        &self,
        data: impl AsRef<[u8]>,
        max_w: u32,
        max_h: u32,
    ) -> Result<Bytes, StorageError> {
        let img = load(data.as_ref())?;
        let fmt = detect_format(data.as_ref());
        let resized = img.resize(max_w, max_h, FilterType::Lanczos3);
        encode(resized, fmt)
    }

    /// Genera un thumbnail de la imagen con dimensiones maximas `max_w x max_h`.
    ///
    /// Preserva el aspect ratio. Usa filtro Nearest (rapido, menor calidad que resize).
    pub fn thumbnail(
        &self,
        data: impl AsRef<[u8]>,
        max_w: u32,
        max_h: u32,
    ) -> Result<Bytes, StorageError> {
        let img = load(data.as_ref())?;
        let fmt = detect_format(data.as_ref());
        let thumb = img.thumbnail(max_w, max_h);
        encode(thumb, fmt)
    }

    /// Convierte la imagen a WebP lossless.
    ///
    /// # TECH-DEBT
    /// `_quality` esta ignorado — `image` 0.25 solo expone WebP lossless.
    /// Para lossy con control de calidad, usar el crate `webp` en la segunda
    /// iteracion de ag-storage.
    /// - impacto: archivos WebP son lossless (pueden ser mas grandes que JPEG).
    /// - eliminacion esperada: segunda iteracion ag-storage en Fase 4.
    pub fn to_webp(&self, data: impl AsRef<[u8]>, _quality: u8) -> Result<Bytes, StorageError> {
        let img = load(data.as_ref())?;
        encode(img, ImageFormat::WebP)
    }
}

fn load(data: &[u8]) -> Result<DynamicImage, StorageError> {
    image::load_from_memory(data).map_err(|e| StorageError::Image(e.to_string()))
}

fn detect_format(data: &[u8]) -> ImageFormat {
    image::guess_format(data).unwrap_or(ImageFormat::Jpeg)
}

fn encode(img: DynamicImage, fmt: ImageFormat) -> Result<Bytes, StorageError> {
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, fmt)
        .map_err(|e| StorageError::Image(e.to_string()))?;
    Ok(Bytes::from(buf.into_inner()))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Genera una imagen JPEG 100x100 en memoria para tests.
    fn test_jpeg_100x100() -> Vec<u8> {
        let img = DynamicImage::new_rgb8(100, 100);
        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, ImageFormat::Jpeg).unwrap();
        buf.into_inner()
    }

    #[test]
    fn image_resize_reduces_dimensions() {
        let processor = ImageProcessor::new();
        let src = test_jpeg_100x100();
        let result = processor.resize(&src, 50, 50).unwrap();
        let resized = image::load_from_memory(&result).unwrap();
        assert!(resized.width() <= 50, "ancho: {}", resized.width());
        assert!(resized.height() <= 50, "alto: {}", resized.height());
    }

    #[test]
    fn image_thumbnail_max_dimensions() {
        let processor = ImageProcessor::new();
        let src = test_jpeg_100x100();
        let result = processor.thumbnail(&src, 30, 30).unwrap();
        let thumb = image::load_from_memory(&result).unwrap();
        assert!(thumb.width() <= 30, "ancho: {}", thumb.width());
        assert!(thumb.height() <= 30, "alto: {}", thumb.height());
    }

    #[test]
    fn image_to_webp_produces_bytes() {
        let processor = ImageProcessor::new();
        let src = test_jpeg_100x100();
        let result = processor.to_webp(&src, 85).unwrap();
        assert!(!result.is_empty());
        // verificar que el resultado es un WebP valido
        assert!(image::load_from_memory(&result).is_ok());
    }
}
```

- [ ] **Step 2: Correr los tests de imagen**

```bash
cargo test -p ag-storage image::tests 2>&1 | tail -15
```

Resultado esperado:
```
test image::tests::image_resize_reduces_dimensions ... ok
test image::tests::image_thumbnail_max_dimensions ... ok
test image::tests::image_to_webp_produces_bytes ... ok
```

- [ ] **Step 3: Commit**

```bash
git add crates/ag-storage/src/image.rs
git commit -m "feat(storage): ImageProcessor — resize/thumbnail/webp, 3 tests"
```

---

## Task 10: Verificacion final y documentacion

**Files:**
- Modify: `crates/ag-storage/README.md`

- [ ] **Step 1: Correr la suite completa**

```bash
cargo test -p ag-storage 2>&1 | tail -30
```

Resultado esperado: todos los tests pasan. Contar que sean al menos 17.

- [ ] **Step 2: fmt y clippy**

```bash
cargo fmt -p ag-storage
cargo clippy -p ag-storage -- -D warnings 2>&1 | grep "^error" | head -20
```

Resultado esperado: `cargo fmt` sin cambios (o cambios menores de formato). `cargo clippy` sin errores.

Si clippy reporta warnings convertidos en errores, corregirlos antes de continuar.

- [ ] **Step 3: cargo check con features**

```bash
cargo check -p ag-storage --features s3 2>&1 | grep "^error" | head -10
cargo check -p ag-storage --all-features 2>&1 | grep "^error" | head -10
```

Resultado esperado: sin errores en ambos casos.

- [ ] **Step 4: Actualizar README.md del crate**

Reemplazar el contenido de `crates/ag-storage/README.md`:

```markdown
# ag-storage

Store nativo Anti-Gravital — almacenamiento de objetos sobre filesystem,
con servidor HTTP embebido, seguridad por construccion y procesamiento de imagen.

> Estado: Fase 4 — implementado.

## Uso minimo (embebido)

```rust
use ag_storage::{AgStorage, StorageConfig};
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = AgStorage::new(StorageConfig::default()).await?;

    storage.put("docs/readme.txt", Bytes::from("hola")).await?;
    let data = storage.get("docs/readme.txt").await?;
    println!("{}", String::from_utf8_lossy(&data));
    Ok(())
}
```

## Modo servidor HTTP

```bash
STORAGE_SERVER=true STORAGE_PORT=4280 cargo run
```

```
PUT    /v1/objects/{*key}    subir
GET    /v1/objects/{*key}    descargar
DELETE /v1/objects/{*key}    borrar
HEAD   /v1/objects/{*key}    existe?
GET    /v1/objects/?prefix=  listar
POST   /v1/copy?from=&to=    copiar
GET    /v1/health            health check
```

## Variables de entorno

| Variable | Default | Descripcion |
|---|---|---|
| `STORAGE_BACKEND` | `native` | `native`, `s3` (feature), `minio` (feature) |
| `STORAGE_ROOT` | `./ag-store-data` | Directorio raiz del store |
| `STORAGE_SERVER` | `false` | Levantar servidor HTTP |
| `STORAGE_PORT` | `4280` | Puerto del servidor |
| `STORE_TOKEN` | `""` | Bearer token (vacio = sin auth) |
| `STORAGE_MAX_OBJECT_SIZE_MB` | `100` | Tamano maximo de objeto |
| `STORAGE_RATE_LIMIT_RPS` | `100` | Requests/segundo del servidor |

## Features

- `auth` — Valida JWT via `ag-auth` en el servidor HTTP.
- `s3` — Adaptadores AWS S3 y MinIO via `object_store`.

## Referencias

- Spec de diseno: `docs/superpowers/specs/2026-05-22-ag-storage-design.md`
- Arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` seccion 8.5.
- Constitucion tecnica: `CLAUDE.md`.
```

- [ ] **Step 5: Commit final**

```bash
git add crates/ag-storage/ Cargo.toml Cargo.lock
git commit -m "feat(storage): ag-storage completo — store nativo, servidor HTTP, imagen, seguridad, 17 tests"
```

---

## Notas de implementacion

**Sobre `normalize_path`:** Se define en `store/mod.rs` a nivel de modulo (no pub). Es una funcion auxiliar pura, sin I/O, usada unicamente por `resolve_path` cuando el path candidato no existe en disco todavia.

**Sobre `with_file_name` en write-then-rename:** `dest.with_file_name(...)` reemplaza solo el nombre de archivo preservando el directorio padre. El archivo temporal queda junto al destino final, garantizando que `rename` sea atomico (mismo filesystem, sin cruzar puntos de montaje).

**Sobre la ruta de copy:** Se usa `POST /v1/copy?from=...&to=...` en lugar de `POST /v1/objects/{*from}/copy` porque el wildcard de axum consumiria `copy` como parte de la clave, haciendo imposible separar la clave del sufijo.

**Sobre `tempfile` en tests:** Se usa `tempfile::TempDir` que limpia el directorio al hacer drop, garantizando aislamiento entre tests. Es una dev-dependency sin efecto en el binario final.
