# ag-storage

Store nativo Anti-Gravital -- almacenamiento de objetos sobre filesystem,
con servidor HTTP embebido, seguridad por construccion y procesamiento de imagen.

> Estado: Fase 4 -- implementado.

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
STORAGE_SERVER=true STORAGE_BIND=127.0.0.1 STORAGE_PORT=4280 cargo run
```

```
PUT    /v1/objects/*key     subir
GET    /v1/objects/*key     descargar
DELETE /v1/objects/*key     borrar
HEAD   /v1/objects/*key     existe?
GET    /v1/objects/?prefix= listar
POST   /v1/copy?from=&to=   copiar
GET    /v1/health            health check
```

## Modo S3 / MinIO

```bash
STORAGE_BACKEND=s3 \
AWS_REGION=us-east-1 \
AWS_ACCESS_KEY_ID=minioadmin \
AWS_SECRET_ACCESS_KEY=minioadmin \
S3_ENDPOINT=http://localhost:9000 \
S3_BUCKET=my-bucket \
cargo run
```

El feature `s3` usa `object_store 0.11` (Apache-2.0). Para MinIO se configura
`S3_ENDPOINT` con la URL del servidor local y se permite HTTP via
`with_allow_http(true)`.

## URLs firmadas

```rust
use ag_storage::AgStorage;
use std::time::{SystemTime, UNIX_EPOCH};

let storage = AgStorage::new(config).await?;

// Generar token valido 3600 segundos
let expires_at = SystemTime::now()
    .duration_since(UNIX_EPOCH)?.as_secs() + 3600;
let token = storage.signed_url("docs/readme.txt", expires_at)?;

// Verificar
storage.verify_signed_url("docs/readme.txt", &token)?;
```

Token HMAC-SHA256 en formato `{base64url_hmac}_{expires_at}`.
Comparacion en tiempo constante; expiracion verificada.
Variable de entorno: `STORAGE_SIGN_SECRET` (string arbitrario).

## Variables de entorno

| Variable | Default | Descripcion |
|---|---|---|
| `STORAGE_BACKEND` | `native` | `native`, `s3` (feature) |
| `STORAGE_ROOT` | `./ag-store-data` | Directorio raiz (native) |
| `STORAGE_SERVER` | `false` | Levantar servidor HTTP |
| `STORAGE_BIND` | `127.0.0.1` | Direccion de escucha; loopback por defecto |
| `STORAGE_PORT` | `4280` | Puerto del servidor |
| `STORE_TOKEN` | `""` | Bearer token; vacio solo se permite en loopback |
| `STORAGE_ALLOW_INSECURE_PUBLIC` | `false` | Permite explicitamente bind publico sin token |
| `STORAGE_MAX_OBJECT_SIZE_MB` | `100` | Tamano maximo de objeto |
| `STORAGE_RATE_LIMIT_RPS` | `100` | Requests/segundo del servidor |
| `STORAGE_SIGN_SECRET` | `""` | Secreto para URLs firmadas |
| `S3_ENDPOINT` | `""` | URL endpoint S3/MinIO (feature `s3`) |
| `S3_BUCKET` | `""` | Nombre del bucket S3 (feature `s3`) |
| `AWS_REGION` | `""` | Region AWS (feature `s3`) |
| `AWS_ACCESS_KEY_ID` | `""` | Credenciales AWS/MinIO (feature `s3`) |
| `AWS_SECRET_ACCESS_KEY` | `""` | Credenciales AWS/MinIO (feature `s3`) |

## Features

- `auth` -- Valida JWT via `ag-auth` en el servidor HTTP.
- `s3` -- Adaptadores AWS S3 y MinIO via `object_store 0.11`.

## Seguridad

A non-loopback `STORAGE_BIND` requires `STORE_TOKEN`. Starting with a
public bind and an empty token returns a configuration error before the server
task starts. `STORAGE_ALLOW_INSECURE_PUBLIC=true` is an explicit unsafe escape
hatch for controlled environments.

Native filesystem operations are relative to an opened directory capability.
Parent symlinks cannot escape the configured root, and temporary writes plus
their final rename remain inside that same capability.

- Validacion de clave: rechaza path traversal (`..`), bytes nulos, caracteres de control.
- Confinamiento de path: canonicalizacion + verificacion `starts_with(root)` impide escape del directorio raiz.
- Proteccion contra symlinks: `canonicalize()` resuelve symlinks antes del check.
- Escritura atomica: write-then-rename con nonce aleatorio evita lecturas parciales.
- Rate limiting: 100 req/s por defecto, configurable.
- Content-Type: lista positiva, extensiones desconocidas forzadas a `attachment`.

Uploaded HTML, HTM, and SVG retain their media type but are always returned with `Content-Disposition: attachment` to prevent active content from executing on the storage origin.

## Referencias

- Spec de diseno: `docs/superpowers/specs/2026-05-22-ag-storage-design.md`
- Arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` seccion 8.5.
- Constitucion tecnica: `CLAUDE.md`.
