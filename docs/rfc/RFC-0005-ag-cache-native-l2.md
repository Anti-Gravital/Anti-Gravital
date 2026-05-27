# RFC-0005: ag-cache L2 Nativo con Protocolo RESP2

**Estado:** Implemented — branch `p5-native-cache-server`, merged into `main` alongside the corrective audit.
**Fecha:** 2026-05-23
**Autor:** Gravital Labs — Nereira Technology and Business Solutions
**Crate afectado:** `ag-cache`
**Feature nueva:** `native-server`

---

## Contexto

`ag-cache` actualmente implementa una cache L1 en memoria (moka) con soporte de tags y TTL. La documentacion tecnica prevee un nivel L2 de cache distribuida. El enfoque natural seria usar Redis como backend L2.

El problema: Redis es un servicio externo gestionado por terceros. Las aplicaciones construidas con Anti-Gravital se disenan para minimizar la complejidad operacional y las dependencias de servicios. Depender de Redis como daemon externo contradice este principio.

Este RFC propone una alternativa nativa: un servidor de cache en proceso que habla el protocolo RESP2 (REdis Serialization Protocol versión 2), lo que mantiene la compatibilidad con cualquier cliente Redis existente sin depender de Redis como proceso separado.

---

## Problema

1. **Dependencia operacional.** Redis requiere instalacion, configuracion, monitorizacion, backups y disponibilidad alta separados de la aplicacion principal.
2. **Complejidad de despliegue.** Un stack Anti-Gravital basico requiere al menos dos procesos (app + Redis) para tener cache L2.
3. **Compatibilidad de herramientas.** Si Anti-Gravital implementa su propia cache con un protocolo propietario, las herramientas de observabilidad, CLI (`redis-cli`) e integraciones existentes no funcionan.

---

## Alternativas consideradas

### Alternativa A: Redis externo (descartada)

Integrar directamente el crate `redis` (MIT) o `deadpool-redis` como backend L2. El cliente habla TCP con un servidor Redis.

**Ventaja:** implementacion minima, compatibilidad total.

**Inconveniente:** dependencia operacional de Redis. Contradice el principio de simplicidad operacional de Anti-Gravital. Las aplicaciones pequenas no deberian necesitar gestionar un daemon Redis solo para tener cache L2.

### Alternativa B: Sled como store embebido (descartada)

Usar `sled` (MIT) como base de datos embebida en disco para L2.

**Ventaja:** no requiere proceso externo, persistente.

**Inconveniente:** no es cache — es una base de datos. Semantica diferente (no hay TTL nativo, no hay protocolo estandar). `sled` esta en modo mantenimiento limitado.

### Alternativa C (elegida): Servidor RESP2 nativo embebido

Implementar un listener TCP dentro del propio proceso de la aplicacion que habla RESP2. Los clientes Redis normales se conectan a `localhost:{puerto}` sin saber que no es Redis.

**Ventaja:**
- Cero dependencias externas: el servidor vive dentro del proceso.
- Compatibilidad con `redis-cli`, qualquier cliente Redis en cualquier lenguaje.
- El store subyacente es el L1 existente (moka) — sin duplicacion de logica.
- Despliegue simple: un solo proceso.

**Inconveniente:**
- Implementar un subset de RESP2 requiere trabajo inicial.
- No escalable mas alla de un proceso (un servidor L2 por instancia).
- Para arquitecturas multi-instancia, se necesita un Redis real — este RFC no reemplaza Redis para ese caso, lo hace opcional.

---

## Diseno

### 4.1 Protocolo RESP2

RESP2 (REdis Serialization Protocol version 2) es un protocolo de texto simple sobre TCP. Los comandos llegan como arrays de bulk strings:

```
*3\r\n$3\r\nSET\r\n$5\r\nhello\r\n$5\r\nworld\r\n
```

Las respuestas son:
- Simple strings: `+OK\r\n`
- Errors: `-ERR message\r\n`
- Integers: `:42\r\n`
- Bulk strings: `$5\r\nhello\r\n` / `$-1\r\n` (nil)
- Arrays: `*2\r\n...`

El parser es simple y determinista. No requiere libreria externa — se implementa directamente sobre `tokio::net::TcpListener` + `tokio::io::BufReader`.

### 4.2 Comandos soportados (minimos para caso de uso cache)

| Comando | Semantica |
|---|---|
| `GET key` | Recuperar valor. Nil si no existe o expirado. |
| `SET key value [EX secs] [PX ms] [NX] [XX]` | Almacenar valor con TTL opcional. |
| `DEL key [key ...]` | Borrar una o varias claves. Retorna numero borradas. |
| `EXISTS key [key ...]` | Contar claves existentes. |
| `EXPIRE key secs` | Establecer TTL en segundos. |
| `TTL key` | TTL restante en segundos. -2 si no existe, -1 si sin TTL. |
| `MGET key [key ...]` | Get multiple. |
| `MSET key value [key value ...]` | Set multiple. |
| `KEYS pattern` | Listar claves. Solo soporta `*` como wildcard (patron simple). |
| `PING [message]` | Health check. Retorna PONG o el mensaje. |
| `FLUSHDB` | Borrar todas las claves. |
| `DBSIZE` | Numero de claves. |
| `COMMAND` | Lista de comandos (para compatibilidad con clientes que auto-detectan capacidades). |

Comandos explicitamente NO soportados (fuera de scope):
- Listas (`LPUSH`, `LRANGE`, etc.)
- Sets (`SADD`, `SMEMBERS`, etc.)
- Hashes (`HSET`, `HGET`, etc.)
- Sorted sets (`ZADD`, `ZRANGE`, etc.)
- Pub/Sub (`PUBLISH`, `SUBSCRIBE`) — para esto existe ag-realtime
- Scripting (`EVAL`)
- Clustering (`CLUSTER`)
- Replication (`REPLICAOF`)
- Persistence (`BGSAVE`, `BGREWRITEAOF`)

Intentar un comando no soportado retorna `-ERR comando no soportado en ag-cache native server\r\n`.

### 4.3 Implementacion

**Nuevo archivo: `crates/ag-cache/src/server/mod.rs`**

```rust
pub struct NativeCacheServer {
    listener: TcpListener,
    cache: Arc<L1Cache>,
}

impl NativeCacheServer {
    pub async fn bind(port: u16, cache: Arc<L1Cache>) -> Result<Self, io::Error>;
    pub async fn serve(self) -> Result<(), io::Error>;
}
```

Cada conexion TCP es una `tokio::spawn`. El parser de RESP2 lee linea a linea usando `BufReader`. Cada comando se despacha al `L1Cache` existente. Sin estado adicional — L1Cache ya es thread-safe.

**Configuracion nueva en `CacheConfig`:**

```rust
pub native_server_enabled: bool,   // default: false
pub native_server_port: u16,       // default: 6379 (compatible Redis)
```

**Activacion:**

```toml
[dependencies]
ag-cache = { version = "0.0.0", features = ["native-server"] }
```

El servidor se inicia en background automaticamente si `native_server_enabled = true`.

### 4.4 Limitaciones conocidas

1. **Single-node.** No hay replicacion ni clustering. Para arquitecturas multi-instancia con cache compartida, seguir usando Redis real.
2. **No persistencia.** Los datos se pierden al reiniciar el proceso. Es cache, no base de datos.
3. **No AUTH.** RESP2 soporta AUTH pero esta implementacion no la incluye — el servidor escucha solo en loopback (`127.0.0.1`) por defecto.
4. **No TLS.** Las conexiones al servidor nativo son TCP plano. Usar Redis real para conexiones TLS.

---

## Impacto

- **ag-cache:** nuevo modulo `server/` bajo feature `native-server`. Zero impacto en el API existente.
- **Tests:** server escucha en puerto aleatorio en tests para no conflictar con Redis local.
- **Documentacion:** README de ag-cache actualizado con seccion "L2 nativo".
- **Otros crates:** ninguno. La API de ag-cache no cambia.

---

## Riesgos

| Riesgo | Mitigacion |
|---|---|
| RESP2 incompatibilidad con clientes modernos | Testear con `redis-cli`, `redis-rs`, y `ioredis` |
| Confusion con Redis real | Documentar claramente los comandos no soportados |
| Puerto 6379 en conflicto con Redis instalado | Configurable; advertencia al inicio si puerto ocupado |
| Regresion en L1 existente | Tests de L1 no se tocan; feature aislada |

---

## Rollback

La feature es completamente opcional (`#[cfg(feature = "native-server")]`). Desactivar la feature vuelve al comportamiento anterior sin ningun cambio de codigo.

---

## Estado y siguientes pasos

Este RFC esta en estado **Propuesto**. Requiere:

1. Revision tecnica del diseno RESP2 por parte del equipo.
2. Aprobacion via PR con al menos una revision.
3. Creacion de rama `f4-cache-native-server` desde `fase-4`.
4. Implementacion segun el diseno de la seccion 4.3.
5. Tests de compatibilidad con `redis-rs` como cliente.
6. Actualizacion de README y CHANGELOG.

**Este RFC NO esta autorizado para implementacion hasta su aprobacion explícita.**

---

## Implementation

- Feature: `native-server` (Cargo).
- Supported commands: GET, SET (EX/PX/NX/XX), DEL, EXISTS, MGET, MSET, EXPIRE, TTL, KEYS (*), PING, FLUSHDB, DBSIZE, COMMAND.
- Per-key TTL via `DashMap<String, Option<Instant>>` (internal expiry map in the server).
- Tests: 13 cases in `crates/ag-cache/tests/resp2_compat.rs` via raw `TcpStream`.
- Validated against redis-cli (inline and array format).
