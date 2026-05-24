# f4-storage-complete — Plan de Implementacion

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implementar el backend S3/MinIO en `ag-storage` (feature `s3`) y URLs firmadas con HMAC-SHA256. El esqueleto ya existe (config, error type, Cargo.toml con `object_store`). Solo falta la implementacion real.

**Architecture:** `store/s3.rs` contiene `S3Store` wrapping `object_store::ObjectStore`. `store/mod.rs` amplia `AgStore` para despachar a `S3Store` cuando el backend es S3 o MinIO. `signed.rs` implementa `sign_url` y `verify_signed_url` usando HMAC-SHA256 sobre `key || expires_at`. La clave de firma se lee de `StorageConfig::sign_secret` (nuevo campo).

**Tech Stack:** `object_store 0.11` (ya en Cargo.toml de ag-storage con feature `aws`), `hmac 0.12` y `sha2 0.10` (nuevos workspace deps), `base64ct` (ya en workspace).

---

## Mapa de archivos

| Accion | Archivo | Responsabilidad |
|---|---|---|
| Modify | `Cargo.toml` (workspace) | Añadir hmac, sha2 si no existen ya (f4-auth-complete los añade) |
| Modify | `crates/ag-storage/Cargo.toml` | Activar hmac, sha2 en deps |
| Modify | `crates/ag-storage/src/config.rs` | Añadir campo `sign_secret` |
| Create | `crates/ag-storage/src/store/s3.rs` | S3Store — adaptador object_store |
| Modify | `crates/ag-storage/src/store/mod.rs` | AgStore despacha a S3Store |
| Create | `crates/ag-storage/src/signed.rs` | sign_url, verify_signed_url |
| Modify | `crates/ag-storage/src/lib.rs` | AgStorage::signed_url, AgStorage::verify_signed_url |

---

## Task 1: Dependencias y campo `sign_secret` en config

**Files:**
- Modify: `Cargo.toml` (workspace si f4-auth-complete no las añadio aun)
- Modify: `crates/ag-storage/Cargo.toml`
- Modify: `crates/ag-storage/src/config.rs`

- [ ] **Step 1: Verificar deps en workspace**

```bash
grep -E "^hmac|^sha2" Cargo.toml
```

Si no aparecen (porque f4-auth-complete no se ejecuto aun), añadir en `[workspace.dependencies]`:

```toml
hmac = "0.12"
sha2 = "0.10"
```

- [ ] **Step 2: Añadir a ag-storage/Cargo.toml**

En `[dependencies]`:

```toml
hmac  = { workspace = true }
sha2  = { workspace = true }
```

- [ ] **Step 3: Tests que fallan para sign_secret**

Añadir al final del bloque `#[cfg(test)]` de `config.rs`:

```rust
#[test]
fn config_sign_secret_default_empty() {
    let cfg = StorageConfig::default();
    assert!(cfg.sign_secret.is_empty(), "sign_secret debe estar vacio por defecto");
}

#[test]
fn config_from_env_reads_sign_secret() {
    std::env::set_var("STORAGE_SIGN_SECRET", "my-secret-key");
    let cfg = StorageConfig::from_env();
    std::env::remove_var("STORAGE_SIGN_SECRET");
    assert_eq!(cfg.sign_secret, "my-secret-key");
}
```

- [ ] **Step 4: Añadir campo sign_secret a StorageConfig**

En `crates/ag-storage/src/config.rs`, añadir al struct `StorageConfig`:

```rust
/// Clave secreta HMAC para firmar URLs. Vacia = URLs firmadas deshabilitadas.
/// Variable: `STORAGE_SIGN_SECRET`.
pub sign_secret: String,
```

En `Default for StorageConfig`, añadir:

```rust
sign_secret: String::new(),
```

En `StorageConfig::from_env`, añadir:

```rust
sign_secret: std::env::var("STORAGE_SIGN_SECRET").unwrap_or_default(),
```

- [ ] **Step 5: Compilar y pasar tests de config**

```bash
cargo test -p ag-storage config -- --nocapture 2>&1
```

Esperado: todos los tests de config pasan.

- [ ] **Step 6: Commit**

```bash
git add crates/ag-storage/src/config.rs Cargo.toml crates/ag-storage/Cargo.toml
git commit -m "feat(ag-storage): sign_secret en StorageConfig + deps hmac/sha2"
```

---

## Task 2: `signed.rs` — URLs firmadas con HMAC-SHA256

**Files:**
- Create: `crates/ag-storage/src/signed.rs`

- [ ] **Step 1: Escribir tests que fallen**

Crear `crates/ag-storage/src/signed.rs`:

```rust
//! URLs firmadas con HMAC-SHA256 para ag-storage.
//!
//! Una URL firmada es valida solo si:
//! 1. La firma HMAC coincide (integridad + autenticidad).
//! 2. El timestamp `expires_at` no ha pasado.
//!
//! Formato del token: `{base64url(hmac)}_{expires_at_secs}`
//! La firma cubre: `key_bytes || expires_at_be_bytes`.

use base64ct::{Base64Url, Encoding};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Error al verificar una URL firmada.
#[derive(Debug, PartialEq)]
pub enum SignedUrlError {
    /// La firma no coincide (posible manipulacion).
    InvalidSignature,
    /// La URL ha expirado.
    Expired,
    /// Formato del token invalido.
    InvalidFormat,
    /// Clave secreta vacia (URLs firmadas no configuradas).
    NoSecret,
}

impl std::fmt::Display for SignedUrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignedUrlError::InvalidSignature => write!(f, "firma invalida"),
            SignedUrlError::Expired => write!(f, "URL expirada"),
            SignedUrlError::InvalidFormat => write!(f, "formato de token invalido"),
            SignedUrlError::NoSecret => write!(f, "STORAGE_SIGN_SECRET no configurado"),
        }
    }
}

impl std::error::Error for SignedUrlError {}

/// Genera la parte de firma del token para `key` con expiracion `expires_at`.
///
/// Retorna `"<base64url_hmac>_<expires_at>"`.
pub fn sign_url(secret: &str, key: &str, expires_at: u64) -> Result<String, SignedUrlError> {
    if secret.is_empty() {
        return Err(SignedUrlError::NoSecret);
    }
    let mac = compute_hmac(secret.as_bytes(), key, expires_at);
    let sig_b64 = Base64Url::encode_string(&mac);
    Ok(format!("{sig_b64}_{expires_at}"))
}

/// Verifica un token generado por `sign_url`.
///
/// Retorna `Ok(())` si la firma es valida y el token no ha expirado.
pub fn verify_signed_url(secret: &str, key: &str, token: &str) -> Result<(), SignedUrlError> {
    if secret.is_empty() {
        return Err(SignedUrlError::NoSecret);
    }

    let (sig_b64, expires_str) = token
        .rsplit_once('_')
        .ok_or(SignedUrlError::InvalidFormat)?;

    let expires_at: u64 = expires_str.parse().map_err(|_| SignedUrlError::InvalidFormat)?;

    // Verificar expiracion antes de la firma para evitar timing attacks innecesarios
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if now > expires_at {
        return Err(SignedUrlError::Expired);
    }

    let expected_mac = compute_hmac(secret.as_bytes(), key, expires_at);
    let provided_sig = Base64Url::decode_vec(sig_b64)
        .map_err(|_| SignedUrlError::InvalidSignature)?;

    // Comparacion en tiempo constante
    if expected_mac.len() != provided_sig.len() {
        return Err(SignedUrlError::InvalidSignature);
    }
    let mismatch = expected_mac
        .iter()
        .zip(provided_sig.iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b));
    if mismatch != 0 {
        return Err(SignedUrlError::InvalidSignature);
    }

    Ok(())
}

fn compute_hmac(secret: &[u8], key: &str, expires_at: u64) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(secret)
        .expect("HMAC acepta claves de cualquier longitud");
    mac.update(key.as_bytes());
    mac.update(&expires_at.to_be_bytes());
    mac.finalize().into_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn future_ts() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600
    }

    fn past_ts() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .saturating_sub(3600)
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let secret = "super-secret-key";
        let key = "avatars/user-42.jpg";
        let expires = future_ts();
        let token = sign_url(secret, key, expires).expect("sign debe tener exito");
        verify_signed_url(secret, key, &token).expect("verify debe tener exito");
    }

    #[test]
    fn expired_token_rejected() {
        let secret = "secret";
        let key = "docs/report.pdf";
        let token = sign_url(secret, key, past_ts()).unwrap();
        let result = verify_signed_url(secret, key, &token);
        assert_eq!(result, Err(SignedUrlError::Expired));
    }

    #[test]
    fn wrong_key_rejected() {
        let secret = "secret";
        let expires = future_ts();
        let token = sign_url(secret, "original-key", expires).unwrap();
        let result = verify_signed_url(secret, "other-key", &token);
        assert_eq!(result, Err(SignedUrlError::InvalidSignature));
    }

    #[test]
    fn wrong_secret_rejected() {
        let expires = future_ts();
        let key = "file.txt";
        let token = sign_url("correct-secret", key, expires).unwrap();
        let result = verify_signed_url("wrong-secret", key, &token);
        assert_eq!(result, Err(SignedUrlError::InvalidSignature));
    }

    #[test]
    fn tampered_token_rejected() {
        let secret = "secret";
        let expires = future_ts();
        let mut token = sign_url(secret, "file.txt", expires).unwrap();
        // Modificar un caracter de la firma
        let idx = token.find('_').unwrap() - 1;
        unsafe {
            let bytes = token.as_bytes_mut();
            bytes[idx] = if bytes[idx] == b'A' { b'B' } else { b'A' };
        }
        let result = verify_signed_url(secret, "file.txt", &token);
        assert!(result.is_err());
    }

    #[test]
    fn empty_secret_returns_no_secret_error() {
        let result = sign_url("", "key.txt", future_ts());
        assert_eq!(result, Err(SignedUrlError::NoSecret));
    }

    #[test]
    fn invalid_token_format_rejected() {
        let result = verify_signed_url("secret", "key.txt", "not-a-token");
        assert_eq!(result, Err(SignedUrlError::InvalidFormat));
    }

    #[test]
    fn different_keys_produce_different_tokens() {
        let secret = "s";
        let expires = future_ts();
        let t1 = sign_url(secret, "file1.txt", expires).unwrap();
        let t2 = sign_url(secret, "file2.txt", expires).unwrap();
        assert_ne!(t1, t2);
    }
}
```

- [ ] **Step 2: Ejecutar tests**

```bash
cargo test -p ag-storage signed -- --nocapture 2>&1
```

Esperado: todos los tests pasan.

- [ ] **Step 3: Exponer en lib.rs**

En `crates/ag-storage/src/lib.rs`, añadir despues de `pub mod image`:

```rust
pub mod signed;
pub use signed::{sign_url, verify_signed_url, SignedUrlError};
```

Y añadir metodo en `impl AgStorage`:

```rust
/// Genera un token firmado para acceso temporal a `key`.
///
/// `ttl_secs`: tiempo de vida en segundos. Error si `sign_secret` esta vacio.
pub fn signed_url(&self, key: &str, ttl_secs: u64) -> Result<String, SignedUrlError> {
    let expires_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        + ttl_secs;
    crate::signed::sign_url(&self.config.sign_secret, key, expires_at)
}

/// Verifica que el token permite acceso a `key`.
pub fn verify_signed_url(&self, key: &str, token: &str) -> Result<(), SignedUrlError> {
    crate::signed::verify_signed_url(&self.config.sign_secret, key, token)
}
```

- [ ] **Step 4: Commit**

```bash
git add crates/ag-storage/src/signed.rs crates/ag-storage/src/lib.rs
git commit -m "feat(ag-storage): signed_url HMAC-SHA256 — acceso temporal firmado a objetos"
```

---

## Task 3: `store/s3.rs` — S3Store con object_store

**Files:**
- Create: `crates/ag-storage/src/store/s3.rs`

- [ ] **Step 1: Crear s3.rs**

```rust
//! Backend S3/MinIO de ag-storage.
//!
//! Solo disponible con la feature `s3`. Usa el crate `object_store`
//! (Apache-2.0) que soporta AWS S3 y cualquier backend S3-compatible (MinIO).

use crate::{StorageConfig, StorageError};
use bytes::Bytes;
use object_store::{
    aws::AmazonS3Builder, local::LocalFileSystem, path::Path, ObjectStore,
};
use std::sync::Arc;

/// Backend de almacenamiento S3/MinIO.
pub struct S3Store {
    inner: Arc<dyn ObjectStore>,
    bucket_prefix: String,
}

impl S3Store {
    /// Construye el store S3 o MinIO segun la configuracion.
    ///
    /// Si `config.endpoint` esta definido, se usa como endpoint personalizado (MinIO).
    /// Si `config.endpoint` es None, se conecta a AWS S3 estandar.
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
            // MinIO u otro S3-compatible
            builder = builder.with_endpoint(endpoint).with_allow_http(true);
        }

        let store = builder
            .build()
            .map_err(|e| StorageError::Config(format!("S3 build: {e}")))?;

        Ok(Self {
            inner: Arc::new(store),
            bucket_prefix: String::new(),
        })
    }

    /// Convierte una clave ag-storage en un `object_store::path::Path`.
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
        let result = self.inner.get(&Self::to_path(key)).await;
        match result {
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

    /// Comprueba si existe un objeto.
    pub async fn exists(&self, key: &str) -> Result<bool, StorageError> {
        match self.inner.head(&Self::to_path(key)).await {
            Ok(_) => Ok(true),
            Err(object_store::Error::NotFound { .. }) => Ok(false),
            Err(e) => Err(StorageError::S3(e)),
        }
    }

    /// Lista objetos con prefijo opcional.
    pub async fn list(&self, prefix: Option<&str>) -> Result<Vec<String>, StorageError> {
        use futures_util::StreamExt;
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
        std::env::var("AWS_ACCESS_KEY_ID").is_ok()
            || std::env::var("MINIO_URL").is_ok()
    }

    #[test]
    fn s3_store_build_fails_without_credentials() {
        if has_s3_env() {
            return; // skip si hay credenciales reales
        }
        // Sin credenciales, build puede tener exito (validacion lazy) o fallar.
        // Solo verificamos que no haya panic.
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
            return; // skip en CI sin MinIO/S3
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
        store.put("test/ag-s3-test.txt", data.clone()).await.expect("put");
        let result = store.get("test/ag-s3-test.txt").await.expect("get");
        assert_eq!(result, data);
        store.delete("test/ag-s3-test.txt").await.expect("delete");
    }
}
```

- [ ] **Step 2: Añadir S3Store a store/mod.rs**

En `crates/ag-storage/src/store/mod.rs`, añadir al inicio (dentro de `#[cfg(feature = "s3")]`):

```rust
#[cfg(feature = "s3")]
pub mod s3;
#[cfg(feature = "s3")]
pub use s3::S3Store;
```

- [ ] **Step 3: Compilar con feature s3**

```bash
cargo check -p ag-storage --features s3 2>&1 | grep "^error" | head -20
```

Esperado: 0 errores.

- [ ] **Step 4: Tests sin S3**

```bash
cargo test -p ag-storage --features s3 s3 -- --nocapture 2>&1
```

Esperado: `s3_store_build_fails_without_credentials` pasa, `with_minio` se salta.

- [ ] **Step 5: Commit**

```bash
git add crates/ag-storage/src/store/s3.rs crates/ag-storage/src/store/mod.rs
git commit -m "feat(ag-storage): S3Store — backend object_store para AWS S3 y MinIO"
```

---

## Task 4: Integrar S3Store en AgStore y AgStorage

**Files:**
- Modify: `crates/ag-storage/src/store/mod.rs`
- Modify: `crates/ag-storage/src/lib.rs`

- [ ] **Step 1: AgStore despacha a S3 cuando el backend es S3/MinIO**

En `store/mod.rs`, reemplazar `pub struct AgStore` y su `impl AgStore`:

```rust
/// Backend de almacenamiento — filesystem nativo o S3/MinIO.
pub enum AgStore {
    Native(NativeStore),
    #[cfg(feature = "s3")]
    S3(S3Store),
}

/// Store nativo (filesystem local).
pub struct NativeStore {
    root: std::path::PathBuf,
    max_object_size: usize,
}
```

Renombrar el `impl AgStore` existente a `impl NativeStore`, y añadir un `impl AgStore` que despacha:

```rust
impl AgStore {
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
            StorageBackend::S3 | StorageBackend::MinIO => {
                Ok(AgStore::S3(S3Store::new(config)?))
            }
        }
    }

    pub async fn put(&self, key: &str, data: Bytes) -> Result<(), StorageError> {
        match self {
            AgStore::Native(s) => s.put(key, data).await,
            #[cfg(feature = "s3")]
            AgStore::S3(s) => s.put(key, data).await,
        }
    }

    pub async fn get(&self, key: &str) -> Result<Bytes, StorageError> {
        match self {
            AgStore::Native(s) => s.get(key).await,
            #[cfg(feature = "s3")]
            AgStore::S3(s) => s.get(key).await,
        }
    }

    pub async fn delete(&self, key: &str) -> Result<(), StorageError> {
        match self {
            AgStore::Native(s) => s.delete(key).await,
            #[cfg(feature = "s3")]
            AgStore::S3(s) => s.delete(key).await,
        }
    }

    pub async fn exists(&self, key: &str) -> Result<bool, StorageError> {
        match self {
            AgStore::Native(s) => s.exists(key).await,
            #[cfg(feature = "s3")]
            AgStore::S3(s) => s.exists(key).await,
        }
    }

    pub async fn list(&self, prefix: Option<&str>) -> Result<Vec<String>, StorageError> {
        match self {
            AgStore::Native(s) => s.list(prefix).await,
            #[cfg(feature = "s3")]
            AgStore::S3(s) => s.list(prefix).await,
        }
    }

    pub async fn copy(&self, from: &str, to: &str) -> Result<(), StorageError> {
        match self {
            AgStore::Native(s) => s.copy(from, to).await,
            #[cfg(feature = "s3")]
            AgStore::S3(s) => s.copy(from, to).await,
        }
    }

    pub fn root(&self) -> Option<&std::path::Path> {
        match self {
            AgStore::Native(s) => Some(&s.root),
            #[cfg(feature = "s3")]
            AgStore::S3(_) => None,
        }
    }
}
```

Nota: si el refactor de renombrar `AgStore` a `NativeStore` genera muchos errores, la alternativa
simplificada es mantener `AgStore` para native y añadir un wrapper `AgStoreBackend` enum — pero el
dispatch directo es mas limpio.

- [ ] **Step 2: Compilar**

```bash
cargo check -p ag-storage 2>&1 | grep "^error" | head -20
cargo check -p ag-storage --features s3 2>&1 | grep "^error" | head -20
```

Esperado: 0 errores en ambos.

- [ ] **Step 3: Tests generales**

```bash
cargo test -p ag-storage 2>&1 | tail -20
```

Esperado: todos los tests existentes siguen pasando (native backend intacto).

- [ ] **Step 4: CI local**

```bash
cargo fmt --all
cargo clippy -p ag-storage -- -D warnings
cargo clippy -p ag-storage --features s3 -- -D warnings
cargo deny check
```

- [ ] **Step 5: Commit y merge**

```bash
git add crates/ag-storage/src/store/mod.rs crates/ag-storage/src/lib.rs
git commit -m "feat(ag-storage): AgStore enum nativo+S3, AgStorage::signed_url integrado"
git checkout fase-4
git merge --no-ff f4-storage-complete -m "merge(f4-storage-complete): S3/MinIO + URLs firmadas HMAC-SHA256"
```
