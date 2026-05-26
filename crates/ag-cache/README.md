# ag-cache

Cache de dos niveles para Anti-Gravital: L1 en memoria del proceso con
invalidacion por etiquetas (moka), L2 Redis opcional (fred).

> Status: Phase 4 — L1 implemented (in-process, tag-based invalidation, moka).
> L2 is NOT functional yet: when `redis_url` is set the cache only logs a tracing
> warning (`src/lib.rs`). RFC-0005 proposes a native Anti-Gravital L2 over RESP2
> (no Redis dependency); see `docs/DEBT.md` and `docs/rfc/RFC-0005-ag-cache-native-l2.md`.

## Uso minimo (L1)

```rust
use ag_cache::l1::L1Cache;
use bytes::Bytes;
use std::time::Duration;

fn main() {
    // max_capacity: numero maximo de entradas, default_ttl: TTL por defecto
    let cache = L1Cache::new(10_000, Duration::from_secs(300));

    // Almacenar con etiquetas para invalidacion selectiva
    cache.set_bytes_tagged("user:42:profile", Bytes::from("...json..."), &["user:42"]);

    // Leer
    if let Some(data) = cache.get_bytes("user:42:profile") {
        println!("{}", String::from_utf8_lossy(&data));
    }

    // Invalidar todas las entradas con la etiqueta "user:42"
    cache.invalidate_tag("user:42");
}
```

## Capacidades

### L1 (moka)

- Cache en memoria de alta concurrencia con `moka`.
- Invalidacion por etiquetas: `set_bytes_tagged(key, value, &["tag1", "tag2"])`.
- TTL configurable por entrada o global via `default_ttl`.
- Capacidad maxima con eviccion LRU automatica.

### L2 Redis (feature `redis`)

Activar con `features = ["redis"]` en `Cargo.toml` y definir `REDIS_URL`.
Uso como cache secundario cuando L1 expira o se reinicia el proceso.

## Variables de entorno (L2)

| Variable | Default | Descripcion |
|---|---|---|
| `REDIS_URL` | `redis://127.0.0.1:6379` | URL del servidor Redis (L2) |

## RFC-0005

El RFC-0005 (`docs/rfc/RFC-0005-ag-cache-native-l2.md`) propone un L2 nativo
Anti-Gravital compatible con el protocolo RESP2, eliminando la dependencia de
Redis como servicio externo. Estado: propuesto, pendiente de aprobacion.

## Referencias

- Spec de diseno: `docs/superpowers/specs/2026-05-23-fase4-completion-design.md`
- Arquitectura: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` seccion 8.3.
- RFC-0005: `docs/rfc/RFC-0005-ag-cache-native-l2.md`.
- Constitucion tecnica: `CLAUDE.md`.
