# ag-cache

> Capitulo de arquitectura: `docs/architecture/08-modulos-batteries-included.md`.
> README del crate: `crates/ag-cache/README.md`.
> RFC-0005: `docs/rfc/RFC-0005-ag-cache-native-l2.md`.
> Criticidad: Estandar.
> Fase de implementacion: Fase 4. Estado: L1 implementado; L2 en RFC.

## Dominio

Cache de dos niveles: L1 en memoria del proceso con invalidacion por tags,
L2 nativo RESP2 (RFC-0005, pendiente de aprobacion) o Redis externo (feature `redis`).

## Stack implementado

| Componente | Libreria | Version |
|---|---|---|
| L1 cache concurrente | `moka` | 0.12 |
| Redis L2 (feature) | `fred` | 9.x |

## Capacidades implementadas (Fase 4, L1)

- `L1Cache::new(max_capacity: u64, default_ttl: Duration)` — constructor.
- `set_bytes_tagged(key, value: Vec<u8>, tags: &[&str])` — almacenar con etiquetas.
- `get_bytes(key) -> Option<Vec<u8>>` — lectura.
- `invalidate_tag(tag: &str)` — invalida todas las entradas con ese tag.
- TTL individual o global. Eviccion LRU automatica cuando se alcanza `max_capacity`.

## API Path

El tipo principal NO esta re-exportado desde `ag_cache::`. Importar directamente:

```rust
use ag_cache::l1::L1Cache;
```

Esta decision mantiene la separacion entre L1 y L2 en la API publica y es
intencional — cambiara cuando L2 se una bajo un tipo `AgCache` unificado (Fase 5).

## RFC-0005: L2 Nativo RESP2

El RFC-0005 propone un servidor de cache en proceso compatible con el protocolo
RESP2 de Redis. Cualquier cliente Redis existente (`redis-cli`, `redis-rs`, `ioredis`)
puede conectarse sin saber que no es Redis. El store subyacente es el L1 existente.

Estado: **Propuesto**. Pendiente de revision y aprobacion tecnica.
Ver `docs/rfc/RFC-0005-ag-cache-native-l2.md` para el diseno completo.

## Dependencias internas permitidas

Depende de `ag-core`.

## Tests

Cobertura >= 80%. Tests cubren: set/get/miss, invalidacion por tag, TTL, concurrencia.

## Pendiente (criterios externos)

- Publicacion en crates.io con version 0.1.0.
- Benchmark L1 >= 1M ops/segundo.
- Aprobacion e implementacion RFC-0005 (L2 nativo RESP2).
- Cache de queries SQL declarativo desde DSL (Fase 5).
