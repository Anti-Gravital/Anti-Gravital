# ag-storage

> Capitulo de arquitectura: `docs/architecture/08-modulos-batteries-included.md`.
> README del crate: `crates/ag-storage/README.md`.
> Spec de diseno: `docs/superpowers/specs/2026-05-22-ag-storage-design.md`.
> Criticidad: Estandar.
> Fase de implementacion: Fase 4. Estado: implementado.

## Dominio

Almacenamiento de objetos: store filesystem nativo con servidor HTTP Axum embebido,
backend S3/MinIO via `object_store`, URLs firmadas HMAC-SHA256, procesamiento de imagen.

## Stack implementado

| Componente | Libreria | Version |
|---|---|---|
| Store filesystem | std::fs + tokio::fs | (std/tokio 1.x) |
| HTTP server embebido | `axum` | 0.7 |
| Backend S3/MinIO (feature) | `object_store` | 0.11 |
| URLs firmadas | `hmac` + `sha2` | 0.12 |
| Encoding URL | `base64ct` | 1.x |
| Procesamiento imagen | `image` | 0.25 |

## Capacidades implementadas (Fase 4)

### AgStore enum

```rust
pub enum AgStore {
    Native(NativeStore),        // siempre disponible
    #[cfg(feature = "s3")]
    S3(S3Store),                // objeto_store 0.11, AWS S3 y MinIO
}
```

### AgStorage (fachada)

- `AgStorage::new(config) -> Result<Self>` — asincrono.
- `put(key, bytes)`, `get(key)`, `delete(key)`, `exists(key)`, `list(prefix)`, `copy(from, to)`.
- `signed_url(key, expires_at: u64) -> Result<String>` — HMAC-SHA256.
- `verify_signed_url(key, token: &str) -> Result<()>` — comparacion en tiempo constante.

### Servidor HTTP embebido

Activado con `STORAGE_SERVER=true`. Endpoints:

```
PUT    /v1/objects/*key
GET    /v1/objects/*key
DELETE /v1/objects/*key
HEAD   /v1/objects/*key
GET    /v1/objects/?prefix=
POST   /v1/copy?from=&to=
GET    /v1/health
```

Bearer token configurable via `STORE_TOKEN`. Rate limiting 100 req/s.

### URLs firmadas

Token HMAC-SHA256 en formato `{base64url_hmac}_{expires_at_unix}`.
HMAC cubre `key_bytes || expires_at.to_be_bytes()`. Comparacion en tiempo
constante via fold XOR. Variable de entorno: `STORAGE_SIGN_SECRET`.

### Seguridad de path

Native filesystem operations use an opened root-directory capability. This
prevents parent symlinks and check/open races from escaping the configured root.

- Validacion: rechaza `..`, bytes nulos, caracteres de control.
- Confinamiento: `canonicalize()` + `starts_with(root)` tras resolucion de symlinks.
- Escritura atomica: write-then-rename con nonce aleatorio.

### Procesamiento de imagen (ImageProcessor)

- `resize(width, height)`, `thumbnail(size)`, `to_webp(quality)`.
- Soporta JPEG, PNG, WebP, GIF de entrada.

## Dependencias internas permitidas

Depende de `ag-core`. Puede depender de `ag-auth` para URLs firmadas autenticadas
(planificado Fase 5 — actualmente el firmado usa `STORAGE_SIGN_SECRET` directo).

## Tests

Cobertura >= 80%. Tests para store, servidor HTTP, URLs firmadas, procesamiento imagen,
y S3Store (unit, sin servidor AWS real).

## Pendiente (criterios externos)

- Publicacion en crates.io con version 0.1.0.
- Integracion completa ag-auth <-> ag-storage para URLs firmadas con JWT.
