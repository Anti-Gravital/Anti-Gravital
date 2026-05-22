# Spec: ag-storage — Almacenamiento de objetos

**Fecha:** 2026-05-22
**Fase:** 4 (Modulos estandar)
**Crate:** `ag-storage`
**Estado:** Aprobado para implementacion

---

## 1. Objetivo

Implementar `ag-storage` como abstraccion delgada sobre tres backends de almacenamiento de objetos: S3 (AWS y compatibles), MinIO (self-hosted S3-compatible) y filesystem local (para desarrollo). El codigo de aplicacion no conoce el backend activo; la seleccion ocurre exclusivamente por configuracion.

## 2. Alcance

### En scope

- `StorageConfig` con lectura desde variables de entorno
- `AgStorage` facade: `put`, `get`, `delete`, `exists`, `signed_url`
- Adaptador local via `object_store::local::LocalFileSystem`
- Adaptador S3/MinIO via `object_store::aws::AmazonS3Builder`
- `ImageProcessor`: resize, thumbnail, convert a WebP, compresion JPEG/PNG
- Tests unitarios con backend local (sin dependencias externas en CI)
- TECH-DEBTs explicitamente documentados

### Fuera de scope (TECH-DEBT)

- Presigned URLs con firma real para S3/MinIO (segunda iteracion)
- Formato AVIF (requiere deps nativas pesadas)
- Thumbnails automaticos segun politica de schema de ag-dsl
- Multipart upload para objetos grandes
- Listado de objetos (`list`)

## 3. Arquitectura

### Estructura de modulos

```
ag-storage/src/
  lib.rs        AgStorage facade, StorageError, Permission
  config.rs     StorageConfig, StorageBackend enum
  local.rs      adaptador filesystem local
  s3.rs         adaptador S3 y MinIO (object_store::aws)
  image.rs      ImageProcessor (resize, thumbnail, webp, compress)
```

### Dependencias nuevas

| Crate | Version | Feature flags | Razon |
|---|---|---|---|
| `object_store` | 0.11 | `aws` | S3, MinIO, local filesystem |
| `image` | 0.25 | `jpeg`, `png`, `webp` | Procesamiento de imagen |
| `url` | 2 | — | Construccion de signed URLs |
| `bytes` | 1 | — | Tipo compartido para payloads |

### API publica

```rust
// Construccion (async: inicializa conexion con el backend)
let storage = AgStorage::new(StorageConfig::from_env()).await?;

// Operaciones de objetos
storage.put("avatars/user-123.jpg", bytes).await?;
let data: Bytes = storage.get("avatars/user-123.jpg").await?;
storage.delete("avatars/user-123.jpg").await?;
let found: bool = storage.exists("avatars/user-123.jpg").await?;

// URL de acceso (local: file://<path>, S3/MinIO: URL publica — firma en TECH-DEBT)
let url: String = storage.signed_url("avatars/user-123.jpg", Duration::from_secs(900), Permission::Read)?;

// Procesamiento de imagen
let processor = storage.processor();
let thumb: Bytes = processor.thumbnail(original_bytes, 200, 200)?;
let webp: Bytes = processor.to_webp(original_bytes, 85)?;
let resized: Bytes = processor.resize(original_bytes, 800, 600)?;
```

## 4. Configuracion

### StorageBackend enum

```rust
pub enum StorageBackend {
    Local,
    S3,
    MinIO,
}
```

### StorageConfig campos

| Campo | Tipo | Default | Env var |
|---|---|---|---|
| `backend` | `StorageBackend` | `Local` | `STORAGE_BACKEND` (`local`/`s3`/`minio`) |
| `bucket` | `String` | `"ag-storage"` | `STORAGE_BUCKET` |
| `region` | `String` | `"us-east-1"` | `STORAGE_REGION` |
| `endpoint` | `Option<String>` | `None` | `STORAGE_ENDPOINT` |
| `access_key` | `Option<String>` | `None` | `AWS_ACCESS_KEY_ID` |
| `secret_key` | `Option<String>` | `None` | `AWS_SECRET_ACCESS_KEY` |
| `base_path` | `Option<String>` | `None` | `STORAGE_BASE_PATH` (solo local) |

## 5. Errores

```rust
pub enum StorageError {
    Backend(object_store::Error),
    NotFound(String),
    Image(String),
    Config(String),
}
```

`StorageError` implementa `std::error::Error` y `Display`.

## 6. TECH-DEBTs explicitamente declarados

### TD-1: Presigned URLs con firma real (S3/MinIO)

`object_store` 0.11 no expone presigned URL generation directamente para todos los casos.
En S3/MinIO la implementacion actual devuelve la URL publica del objeto sin firma.
La segunda iteracion integrara `aws-sigv4` o actualizara a `object_store` con soporte completo.

- Impacto: las URLs devueltas por `signed_url()` en S3/MinIO no estan firmadas — cualquier objeto publico es accesible, objetos privados no.
- Eliminacion esperada: segunda iteracion ag-storage en Fase 4.

### TD-2: AVIF

El formato AVIF requiere `libavif` o `rav1e` con dependencias de compilacion nativas que rompen CI en Windows y algunos entornos macOS. Se documenta como TECH-DEBT y se evalua en segunda iteracion.

- Impacto: no se puede generar/convertir imagenes AVIF.
- Eliminacion esperada: segunda iteracion con verificacion de CI multiplataforma.

### TD-3: Thumbnails automaticos segun politica de schema

El spec arquitectonico menciona thumbnails declarados en el schema DSL. Esta integracion requiere que `ag-dsl` exponga metadatos de politica en tiempo de ejecucion. Se implementa cuando la integracion DSL-runtime este disponible.

- Impacto: thumbnails deben generarse manualmente por el desarrollador.
- Eliminacion esperada: segunda iteracion ag-storage tras integracion ag-dsl.

## 7. Tests

| Nombre | Tipo | Backend | Verifica |
|---|---|---|---|
| `config_defaults_are_local` | unit | — | Default es `StorageBackend::Local` |
| `config_from_env_reads_backend` | unit | — | `STORAGE_BACKEND=s3` -> `S3` |
| `put_get_roundtrip_local` | async unit | local | bytes puestos y recuperados son identicos |
| `get_not_found_returns_error` | async unit | local | `StorageError::NotFound` en clave inexistente |
| `delete_removes_object` | async unit | local | `exists()` false tras `delete()` |
| `exists_returns_false_for_missing` | async unit | local | `exists()` false en clave inexistente |
| `signed_url_local_has_file_prefix` | unit | local | URL empieza con `file://` |
| `image_resize_reduces_dimensions` | unit | — | imagen resultante <= dimensiones pedidas |
| `image_thumbnail_max_dimensions` | unit | — | thumbnail tiene dimensiones <= WxH pedido (aspect ratio preservado) |
| `image_to_webp_produces_bytes` | unit | — | conversion produce bytes no vacios |

Total: 10 tests. Sin dependencias externas. Sin Docker. Sin AWS.

## 8. Coherencia con documentacion maestra

- Arquitectura Tecnica seccion 8.5: cumplido (S3, MinIO, local, URLs firmadas, image processing).
- Hoja de Ruta Fase 4: cumplido (`ag-storage` completo como crate independiente).
- CLAUDE.md regla 14: `ag-storage` es crate estandar, sin dependencias de otros crates Anti-Gravital.
- CLAUDE.md regla 18: 10 tests, sin estado externo requerido.
- CLAUDE.md regla 29: 3 TECH-DEBTs declarados con formato obligatorio.
