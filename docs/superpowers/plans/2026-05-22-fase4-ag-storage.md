# ag-storage — Plan de Implementacion

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Almacenamiento de objetos con tres backends intercambiables (S3, MinIO, filesystem local), URLs firmadas y procesamiento de imagenes en upload.

**Architecture:** Trait `StorageBackend` + tres implementaciones. `AgStorage` elige el backend por configuracion. El codigo de aplicacion no se entera del backend. Rama: `fase-4/ag-storage`.

**Precondicion:** `fase-4/ag-observe` mergeado a `fase-4`.

**Tech Stack:** `aws-sdk-s3` (S3 + MinIO), `image` (procesamiento), `hmac`/`sha2` (URLs firmadas para filesystem), `testcontainers` + `testcontainers-modules` (MinIO).

**Spec:** `docs/superpowers/specs/2026-05-22-fase4-modulos-estandar-design.md` seccion ag-storage.
**Arquitectura:** `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` seccion 8.5.

---

## Mapa de archivos

### Crear
- `crates/ag-storage/Cargo.toml`
- `crates/ag-storage/src/lib.rs` — `AgStorage`, `StorageConfig`, re-exports
- `crates/ag-storage/src/config.rs` — `StorageConfig::from_env()`, `StorageBackendType`
- `crates/ag-storage/src/backend.rs` — trait `StorageBackend`
- `crates/ag-storage/src/s3.rs` — implementacion S3/MinIO via aws-sdk-s3
- `crates/ag-storage/src/local.rs` — implementacion filesystem local
- `crates/ag-storage/src/image.rs` — procesamiento de imagenes con thumbnails

### Modificar
- `Cargo.toml` (root) — dependencias nuevas + miembro `crates/ag-storage`

---

## Task 1: Cargo.toml y trait StorageBackend

- [ ] Agregar a `[workspace.members]`: `"crates/ag-storage"`
- [ ] Agregar a `[workspace.dependencies]`:
  ```toml
  aws-sdk-s3  = "1"
  aws-config  = "1"
  image       = { version = "0.25", default-features = false, features = ["jpeg", "png", "webp"] }
  hmac        = "0.12"
  sha2        = "0.10"
  hex         = "0.4"
  bytes.workspace = true  # ya existe
  ```
- [ ] Crear `crates/ag-storage/Cargo.toml` con deps anteriores + `tokio`, `serde`, `tracing`, `ag-core`; dev-deps: `testcontainers`, `testcontainers-modules` (feature `minio`)
- [ ] Definir trait en `crates/ag-storage/src/backend.rs`:
  ```rust
  #[async_trait::async_trait]
  pub trait StorageBackend: Send + Sync {
      async fn put(&self, key: &str, data: bytes::Bytes, content_type: &str) -> Result<(), StorageError>;
      async fn get(&self, key: &str) -> Result<bytes::Bytes, StorageError>;
      async fn delete(&self, key: &str) -> Result<(), StorageError>;
      async fn signed_url(&self, key: &str, ttl: std::time::Duration, writable: bool) -> Result<String, StorageError>;
      async fn exists(&self, key: &str) -> Result<bool, StorageError>;
  }
  ```
- [ ] Definir `StorageError` con variantes: `NotFound`, `Unauthorized`, `Backend(String)`, `InvalidKey(String)`
- [ ] Verificar: `cargo check -p ag-storage 2>&1 | grep "^error" | head -5`
- [ ] Commit: `chore(storage): Cargo.toml, deps workspace, trait StorageBackend`

---

## Task 2: StorageConfig

**Files:** `crates/ag-storage/src/config.rs`

- [ ] TDD: `storage_config_local_by_default` — sin `STORAGE_BACKEND` en env, `config.backend` es `Local`
- [ ] TDD: `storage_config_s3_reads_bucket` — con `STORAGE_BACKEND=s3` y `STORAGE_BUCKET=my-bucket`, `config.bucket` es `Some("my-bucket")`
- [ ] Implementar:
  ```rust
  pub enum StorageBackendType { Local, S3, Minio }
  pub struct StorageConfig {
      pub backend: StorageBackendType,
      pub bucket: Option<String>,        // STORAGE_BUCKET
      pub endpoint: Option<String>,      // STORAGE_ENDPOINT (para MinIO)
      pub region: String,               // AWS_REGION, default "us-east-1"
      pub local_path: PathBuf,          // STORAGE_LOCAL_PATH, default "./storage"
      pub signing_secret: String,       // STORAGE_SIGNING_SECRET (para URLs locales)
  }
  ```
- [ ] Correr tests, commit: `feat(storage): StorageConfig from_env`

---

## Task 3: Backend filesystem local

**Files:** `crates/ag-storage/src/local.rs`

- [ ] TDD (unitarios, sin infra):
  - `local_put_and_get_roundtrip` — guardar bytes, recuperarlos, iguales
  - `local_delete_removes_file` — guardar, borrar, `exists()` retorna `false`
  - `local_get_not_found_returns_error` — clave inexistente retorna `StorageError::NotFound`
  - `local_signed_url_contains_expiry` — URL generada contiene el timestamp de expiracion
  - `local_signed_url_expired_is_detectable` — URL expirada puede detectarse (verificar HMAC + timestamp)
- [ ] Implementar `LocalBackend { base_path: PathBuf, signing_secret: String }`:
  - `put`: crea directorios intermedios, escribe bytes en `base_path/key`
  - `get`: lee el archivo
  - `delete`: elimina el archivo
  - `signed_url`: genera `{base_url}/{key}?expires={unix_ts}&sig={hmac_hex}` con HMAC-SHA256 sobre `{key}:{expires}`
  - `exists`: `tokio::fs::metadata`
- [ ] Correr tests
- [ ] Commit: `feat(storage): backend filesystem local con URLs firmadas HMAC`

---

## Task 4: Backend S3/MinIO

**Files:** `crates/ag-storage/src/s3.rs`

- [ ] TDD (integracion con testcontainers MinIO):
  - `s3_put_and_get_roundtrip` — subir objeto, descargarlo, contenido igual
  - `s3_delete_removes_object` — subir, borrar, `exists()` retorna `false`
  - `s3_signed_url_is_accessible` — URL pre-firmada tiene formato correcto (contiene el host de MinIO)
  - `s3_not_found_returns_storage_error` — clave inexistente retorna `StorageError::NotFound`
- [ ] Correr para verificar que fallan
- [ ] Implementar `S3Backend { client: aws_sdk_s3::Client, bucket: String }`:
  - `new(config: &StorageConfig) -> Result<Self>` — configura el cliente S3 con region y endpoint opcional (para MinIO)
  - `put`: `client.put_object().bucket(&self.bucket).key(key).body(data).content_type(ct).send().await`
  - `get`: `client.get_object()...` + leer el body como bytes
  - `delete`: `client.delete_object()...`
  - `signed_url`: `client.presigned_url(PutObject | GetObject, expiry)`
  - `exists`: HEAD request, `Ok(true)` si 200, `Ok(false)` si 404
- [ ] Correr tests (levantan contenedor MinIO automaticamente)
- [ ] Commit: `feat(storage): backend S3/MinIO via aws-sdk-s3`

---

## Task 5: Procesamiento de imagenes

**Files:** `crates/ag-storage/src/image.rs`

- [ ] TDD (unitarios, sin infra):
  - `image_resize_produces_correct_dimensions` — imagen 1000x1000 PNG, resize a 128x128, resultado tiene 128x128
  - `image_convert_to_webp_produces_webp_bytes` — imagen JPEG de entrada, salida es WebP valido
  - `image_policy_generates_thumbnails` — con `ImagePolicy { formats: &[WebP], sizes: &[(128,128), (512,512)] }`, `process_image(bytes)` retorna 2 variantes
- [ ] Correr para verificar que fallan
- [ ] Implementar:
  ```rust
  pub struct ImagePolicy {
      pub formats: Vec<ImageFormat>,   // WebP, Jpeg, Png, Avif
      pub sizes: Vec<(u32, u32)>,      // (ancho, alto)
  }
  pub struct ImageVariant {
      pub key_suffix: String,          // p.ej. "-128x128.webp"
      pub data: bytes::Bytes,
      pub content_type: &'static str,
  }
  pub fn process_image(data: &[u8], policy: &ImagePolicy) -> Result<Vec<ImageVariant>>
  ```
  Usar `image::load_from_memory(data)`, `.resize(w, h, FilterType::Lanczos3)`, codificar en formato solicitado
- [ ] Correr tests
- [ ] Commit: `feat(storage): procesamiento de imagenes — resize, conversion, thumbnails`

---

## Task 6: AgStorage facade y verificacion final

**Files:** `crates/ag-storage/src/lib.rs`

- [ ] Implementar `AgStorage`:
  ```rust
  pub struct AgStorage { backend: Arc<dyn StorageBackend> }
  impl AgStorage {
      pub async fn new(config: StorageConfig) -> Result<Self>  // elige backend segun config
      pub async fn put(&self, key: &str, data: Bytes, content_type: &str) -> Result<()>
      pub async fn get(&self, key: &str) -> Result<Bytes>
      pub async fn delete(&self, key: &str) -> Result<()>
      pub async fn signed_url(&self, key: &str, ttl: Duration) -> Result<String>
      pub async fn put_image(&self, key_prefix: &str, data: Bytes, policy: ImagePolicy) -> Result<Vec<String>>
      // put_image: procesa variantes y las sube, retorna lista de keys generados
  }
  ```
- [ ] Integrar con ag-observe: `tracing::info!` en upload/download, counter `ag_storage_operations_total{op, backend}`
- [ ] `cargo fmt --all && cargo clippy -p ag-storage -- -D warnings`
- [ ] `cargo test -p ag-storage 2>&1 | tail -10`
- [ ] `cargo doc -p ag-storage --no-deps 2>&1 | grep "^error" | head -5`
- [ ] Commit: `feat(storage): AgStorage facade con observabilidad`
- [ ] Merge: `git checkout fase-4 && git merge --no-ff fase-4/ag-storage -m "feat(storage): ag-storage completo — S3/MinIO/local + imagenes + URLs firmadas"`
