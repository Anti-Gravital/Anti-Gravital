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
STORAGE_SERVER=true STORAGE_PORT=4280 cargo run
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

- `auth` -- Valida JWT via `ag-auth` en el servidor HTTP.
- `s3` -- Adaptadores AWS S3 y MinIO via `object_store`.

## Seguridad

- Validacion de clave: rechaza path traversal (`..`), bytes nulos, caracteres de control.
- Confinamiento de path: canonicalizacion + verificacion `starts_with(root)` impide escape del directorio raiz.
- Proteccion contra symlinks: `canonicalize()` resuelve symlinks antes del check.
- Escritura atomica: write-then-rename con nonce aleatorio evita lecturas parciales.
- Rate limiting: 100 req/s por defecto, configurable.
- Content-Type: lista positiva, extensiones desconocidas forzadas a `attachment`.

## Referencias

- Spec de diseno: `docs/superpowers/specs/2026-05-22-ag-storage-design.md`
- Arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` seccion 8.5.
- Constitucion tecnica: `CLAUDE.md`.
