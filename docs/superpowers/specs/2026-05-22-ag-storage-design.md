# Spec: ag-storage — Store nativo Anti-Gravital

**Fecha:** 2026-05-22
**Fase:** 4 (Modulos estandar)
**Crate:** `ag-storage`
**Estado:** Aprobado para implementacion

---

## 1. Objetivo

Implementar `ag-storage` como sistema de almacenamiento de objetos nativo de Anti-Gravital. El store nativo (`AgStore`) almacena archivos en disco usando rutas de filesystem estandar como claves. Puede usarse embebido en una aplicacion o levantarse como servidor HTTP independiente. Opcionalmente soporta backends S3/MinIO via feature flag. Incluye procesamiento de imagen.

## 2. Concepto central

`AgStore` es un object storage cuya representacion en disco es un filesystem comun. La clave de un objeto es su ruta:

```
"avatars/user-123.jpg"   ->  {root}/avatars/user-123.jpg
"docs/reports/q1.pdf"    ->  {root}/docs/reports/q1.pdf
"config/settings.json"   ->  {root}/config/settings.json
```

No hay indice separado, base de datos auxiliar ni formato propietario. Es un directorio de Linux. Cualquier persona puede inspeccionarlo con `ls`, copiarlo con `rsync` o moverlo con `mv`. Esta legibilidad es una propiedad del diseno, no un compromiso.

## 3. Alcance

### En scope

- `AgStore`: operaciones nativas sobre disco (put, get, delete, exists, list, copy)
- `StorageConfig` con lectura desde variables de entorno
- Modo servidor: Axum HTTP levantado en proceso cuando `server_mode = true`
- HTTP API REST v1 para todas las operaciones del store
- Autenticacion del servidor: Bearer token estatico (default) o JWT via ag-auth (feature `auth`)
- `ImageProcessor`: resize, thumbnail, convert a WebP
- Adaptadores S3/MinIO via `object_store` (feature `s3`)
- 12 tests sin dependencias externas (backend local en todos)

### Fuera de scope (TECH-DEBT)

- Cliente HTTP para hablar con un AgStore remoto desde otra aplicacion
- Presigned URLs con firma real para S3/MinIO
- Formato AVIF
- Thumbnails automaticos segun politica de schema ag-dsl
- Multipart upload para objetos grandes (streaming chunked)
- Replicacion entre instancias AgStore

## 4. Arquitectura

### Estructura de modulos

```
ag-storage/src/
  lib.rs          AgStorage facade, StorageError, Permission
  config.rs       StorageConfig, StorageBackend enum
  store/
    mod.rs        AgStore — operaciones nativas sobre disco
    server.rs     servidor HTTP Axum (activado por config)
    auth.rs       middleware Bearer token + feature "auth" ag-auth JWT
  image.rs        ImageProcessor (resize, thumbnail, webp, compress)
  s3.rs           adaptador S3/MinIO via object_store (feature "s3")
```

### Features de Cargo

```toml
[features]
default = []
auth = ["dep:ag-auth"]
s3   = ["dep:object_store"]
```

- `auth`: activa validacion JWT via `ag-auth::AgAuth` en el servidor HTTP. Sin este feature, la autenticacion usa Bearer token estatico.
- `s3`: activa los adaptadores S3 y MinIO. Sin este feature, el unico backend disponible es `native`.

### Dependencias base (sin features)

| Crate | Version | Razon |
|---|---|---|
| `tokio` | 1 | async runtime |
| `axum` | 0.7 | servidor HTTP |
| `tower-http` | 0.6 | middlewares HTTP (logging, cors) |
| `image` | 0.25 | procesamiento de imagen |
| `serde` | 1 | serializacion de respuestas JSON |
| `serde_json` | 1 | JSON |
| `bytes` | 1 | tipo de payload |
| `thiserror` | 2 | derivar Error |
| `tracing` | 0.1 | logging estructurado |

### Dependencias condicionales

| Crate | Feature | Razon |
|---|---|---|
| `object_store` | `s3` | backends S3/MinIO |
| `ag-auth` | `auth` | validacion JWT en servidor |

## 5. API publica

### AgStorage facade

```rust
// Construccion
let storage = AgStorage::new(StorageConfig::from_env()).await?;

// Operaciones de objetos
storage.put("avatars/user-123.jpg", bytes).await?;
let data: Bytes = storage.get("avatars/user-123.jpg").await?;
storage.delete("avatars/user-123.jpg").await?;
let found: bool = storage.exists("avatars/user-123.jpg").await?;
storage.copy("avatars/user-123.jpg", "avatars/backup/user-123.jpg").await?;
let keys: Vec<String> = storage.list(Some("avatars/")).await?;

// URL de acceso
// Native: "file://{root}/avatars/user-123.jpg"
// Servidor activo: "http://localhost:4280/v1/objects/avatars/user-123.jpg"
// S3/MinIO: URL publica del objeto (firma en TECH-DEBT)
let url: String = storage.object_url("avatars/user-123.jpg")?;

// Procesamiento de imagen
let processor = storage.processor();
let resized: Bytes = processor.resize(bytes, 800, 600)?;
let thumb: Bytes = processor.thumbnail(bytes, 200, 200)?;
let webp: Bytes = processor.to_webp(bytes, 85)?;
```

### HTTP API del servidor (modo servidor activo)

| Metodo | Ruta | Descripcion |
|---|---|---|
| `PUT` | `/v1/objects/{*key}` | Subir o reemplazar un objeto |
| `GET` | `/v1/objects/{*key}` | Descargar un objeto |
| `DELETE` | `/v1/objects/{*key}` | Borrar un objeto |
| `HEAD` | `/v1/objects/{*key}` | Verificar existencia y metadata |
| `GET` | `/v1/objects/` | Listar objetos (`?prefix=avatars/`) |
| `POST` | `/v1/objects/{*from}/copy` | Copiar (`?to=avatars/backup/user.jpg`) |
| `GET` | `/v1/health` | Health check (sin auth requerida) |

Respuestas de lista (`GET /v1/objects/`):

```json
{
  "keys": ["avatars/user-123.jpg", "avatars/user-456.jpg"],
  "prefix": "avatars/",
  "count": 2
}
```

Cabeceras de respuesta en GET:
- `Content-Type`: inferido de la extension del objeto
- `Content-Length`: tamano en bytes
- `X-AG-Store-Key`: clave del objeto
- `ETag`: hash SHA256 truncado del contenido

### Autenticacion del servidor

Sin feature `auth`:
- Si `STORE_TOKEN` esta vacio o no definido: todas las rutas son publicas (modo dev).
- Si `STORE_TOKEN` tiene valor: todas las rutas (excepto `/v1/health`) requieren `Authorization: Bearer {token}`.

Con feature `auth`:
- El servidor valida el Bearer token como JWT via `ag-auth::AgAuth`.
- Si el token no es JWT valido, rechaza con 401.

## 6. Configuracion

### StorageConfig campos

| Campo | Tipo | Default | Env var |
|---|---|---|---|
| `backend` | `StorageBackend` | `Native` | `STORAGE_BACKEND` (`native`/`s3`/`minio`) |
| `root_path` | `PathBuf` | `./ag-store-data` | `STORAGE_ROOT` |
| `server_mode` | `bool` | `false` | `STORAGE_SERVER` (`true`/`false`) |
| `server_port` | `u16` | `4280` | `STORAGE_PORT` |
| `store_token` | `String` | `""` | `STORE_TOKEN` |
| `region` | `String` | `"us-east-1"` | `STORAGE_REGION` (solo s3) |
| `endpoint` | `Option<String>` | `None` | `STORAGE_ENDPOINT` (solo minio) |
| `access_key` | `Option<String>` | `None` | `AWS_ACCESS_KEY_ID` (solo s3) |
| `secret_key` | `Option<String>` | `None` | `AWS_SECRET_ACCESS_KEY` (solo s3) |
| `bucket` | `String` | `"ag-storage"` | `STORAGE_BUCKET` (solo s3/minio) |

### StorageBackend enum

```rust
pub enum StorageBackend {
    Native,
    #[cfg(feature = "s3")]
    S3,
    #[cfg(feature = "s3")]
    MinIO,
}
```

## 7. Errores

```rust
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("objeto no encontrado: {0}")]
    NotFound(String),
    #[error("error de backend: {0}")]
    Io(#[from] std::io::Error),
    #[error("error de imagen: {0}")]
    Image(String),
    #[error("configuracion invalida: {0}")]
    Config(String),
    #[cfg(feature = "s3")]
    #[error("error S3: {0}")]
    S3(#[from] object_store::Error),
}
```

## 8. TECH-DEBTs explicitamente declarados

### TD-1: Cliente HTTP remoto

Sin un cliente HTTP que hable con un servidor AgStore remoto, las aplicaciones solo pueden usar el backend en modo embebido o construir sus propias llamadas HTTP. El cliente se implementa en la segunda iteracion.

- Impacto: acceso remoto requiere HTTP manual o mode embebido.
- Eliminacion esperada: segunda iteracion ag-storage en Fase 4.

### TD-2: Presigned URLs con firma real (S3/MinIO)

En S3/MinIO la implementacion actual devuelve la URL publica del objeto sin firma criptografica. La firma real requiere `aws-sigv4`.

- Impacto: objetos privados en S3 no son accesibles por URL generada.
- Eliminacion esperada: segunda iteracion ag-storage en Fase 4.

### TD-3: AVIF

Requiere `libavif` o `rav1e` con deps nativas que rompen CI en Windows.

- Impacto: no se puede generar ni convertir imagenes AVIF.
- Eliminacion esperada: segunda iteracion con verificacion CI multiplataforma.

### TD-4: Thumbnails automaticos por schema DSL

Requiere integracion entre ag-dsl y ag-storage en runtime.

- Impacto: thumbnails deben generarse manualmente.
- Eliminacion esperada: tras integracion ag-dsl runtime, segunda mitad Fase 4.

## 9. Tests

| Nombre | Tipo | Verifica |
|---|---|---|
| `config_defaults_are_native` | unit | Default backend es `Native` |
| `config_server_mode_off_by_default` | unit | `server_mode = false` por default |
| `config_from_env_reads_port` | unit | `STORAGE_PORT` se lee correctamente |
| `put_get_roundtrip` | async unit | bytes puestos y recuperados son identicos |
| `get_not_found_returns_error` | async unit | `StorageError::NotFound` en clave inexistente |
| `delete_removes_object` | async unit | `exists()` false tras `delete()` |
| `exists_returns_false_for_missing` | async unit | `exists()` false en clave no existente |
| `list_returns_keys_by_prefix` | async unit | `list(Some("avatars/"))` retorna solo claves con ese prefijo |
| `copy_duplicates_object` | async unit | objeto copiado existe en destino, original intacto |
| `image_resize_reduces_dimensions` | unit | imagen resultante tiene dimensiones <= pedidas |
| `image_thumbnail_max_dimensions` | unit | thumbnail <= WxH con aspect ratio preservado |
| `image_to_webp_produces_bytes` | unit | conversion produce bytes no vacios |

Total: 12 tests. Sin dependencias externas. Sin Docker. Sin credenciales AWS.

## 10. Coherencia con documentacion maestra

- Arquitectura Tecnica 8.5: cumplido (storage nativo + S3/MinIO opcional + image processing).
- Hoja de Ruta Fase 4: cumplido (`ag-storage` completo como crate independiente).
- CLAUDE.md regla 14: `ag-storage` es crate estandar. Dependencia de `ag-auth` es opt-in via feature.
- CLAUDE.md regla 15: sin dependencias circulares (ag-auth no depende de ag-storage).
- CLAUDE.md regla 18: 12 tests, sin estado externo requerido.
- CLAUDE.md regla 21: no hay magia — el store es un directorio de Linux legible.
- CLAUDE.md regla 29: 4 TECH-DEBTs con formato obligatorio.
