# P5 — ag-cache L2 nativo RESP2 (COMPUERTA: RFC-0005)

> **For agentic workers:** Plan hijo de `2026-05-26-corrective-before-fase5-MASTER.md`.
> **BLOQUEADO** hasta que RFC-0005 pase a estado "Aceptado" (ver Task 0). NO escribir
> codigo de implementacion antes de esa aprobacion (CLAUDE.md secciones 5 y 22).
> Ejecutar con superpowers:subagent-driven-development o executing-plans. TDD estricto.
> Comentarios en ingles (ADR-0008). Leer cada archivo antes de editar.

**Goal:** Reemplazar el L2 de `ag-cache` (hoy un stub que solo emite un warning) por un
servidor de cache nativo en proceso que habla un subconjunto de RESP2, eliminando la
dependencia obligatoria de Redis (filosofia de autosuficiencia, CLAUDE.md secciones 12-13).

**Architecture:** Tal y como especifica `docs/rfc/RFC-0005-ag-cache-native-l2.md`: un
`tokio::net::TcpListener` en `127.0.0.1:{port}` que parsea RESP2 a mano (sin libreria),
despacha cada comando contra el `L1Cache` existente (moka, ya thread-safe), bajo la
feature `native-server`. Cero cambios al API publico de `ag-cache`. Single-node, sin
persistencia, sin AUTH, sin TLS (limitaciones documentadas en la RFC).

**Tech Stack:** Rust, tokio (net, io), moka (L1 existente). Cliente de prueba: `redis` crate.

**Cierra:** DEBT-004 (L2 nativo) de `docs/DEBT.md`.

---

## Estado actual (verificado)

- `docs/rfc/RFC-0005-ag-cache-native-l2.md`: estado **Propuesto**. Diseno COMPLETO
  (protocolo RESP2 seccion 4.1, tabla de comandos 4.2, esbozo de `NativeCacheServer`
  4.3, `CacheConfig` con `native_server_enabled`/`native_server_port`, limitaciones 4.4,
  riesgos, rollback). Dice literal: "Este RFC NO esta autorizado para implementacion
  hasta su aprobacion explicita."
- `crates/ag-cache/src/lib.rs`: `AgCache` con `l1: L1Cache`; en `new`, si `redis_url`
  es `Some` solo hace `tracing::warn!` (L2 no funcional). `CacheError::Redis(String)`.
- `crates/ag-cache/src/{config.rs,l1.rs,tags.rs}`. `L1Cache` ya thread-safe.

El diseno NO necesita redactarse: ya existe. Esta compuerta es solo de APROBACION.

---

## Task 0: Aprobar y finalizar RFC-0005 (BLOQUEANTE)

**Files:**
- Modify: `docs/rfc/RFC-0005-ag-cache-native-l2.md`

- [ ] **Step 1: Revision tecnica del diseno (humano)**

El mantenedor (BDFL, ADR-0003) revisa la seccion 4 de RFC-0005. Confirmar: subset de
comandos suficiente, bind solo loopback por defecto, store = L1 existente, feature
aislada. Si hay cambios, editarlos en la RFC ANTES de implementar.

- [ ] **Step 2: Marcar la RFC como Aceptada**

Cambiar `**Estado:** Propuesto` por `**Estado:** Aceptado (2026-..., revision por <revisor>)`.
Actualizar la seccion "Estado y siguientes pasos" para reflejar la aprobacion.

```bash
git add docs/rfc/RFC-0005-ag-cache-native-l2.md
git commit -m "docs(rfc-0005): accept native RESP2 L2 design for implementation"
```

> **Si RFC-0005 NO se aprueba:** este plan se detiene aqui. DEBT-004 permanece abierta
> y P5 sale del alcance del ciclo correctivo. El resto del master (P1-P4, P6) no depende de P5.

---

## Mapa de archivos (tras aprobacion)

- Modify: `crates/ag-cache/Cargo.toml` (feature `native-server`; dev-dep `redis`)
- Create: `crates/ag-cache/src/server/mod.rs` (listener + dispatch)
- Create: `crates/ag-cache/src/server/resp2.rs` (parser/encoder RESP2)
- Modify: `crates/ag-cache/src/config.rs` (`native_server_enabled`, `native_server_port`)
- Modify: `crates/ag-cache/src/lib.rs` (arranque del server bajo feature; quitar warning)
- Modify: `crates/ag-cache/README.md` (seccion L2 nativo)
- Test: `crates/ag-cache/tests/resp2_compat.rs`

---

## Task 1: Parser/encoder RESP2 (TDD puro, sin red)

**Files:**
- Create: `crates/ag-cache/src/server/resp2.rs`

- [ ] **Step 1: Declarar la feature y el modulo**

`crates/ag-cache/Cargo.toml`:
```toml
[features]
native-server = []
```
Crear `crates/ag-cache/src/server/mod.rs` minimo con `pub mod resp2;` y exponer
`#[cfg(feature = "native-server")] pub mod server;` en `lib.rs`.

- [ ] **Step 2: Escribir tests del parser primero**

En `resp2.rs`:

```rust
//! Minimal RESP2 codec for the native cache server. See RFC-0005 section 4.1.
//!
//! Commands arrive as arrays of bulk strings:
//!   *3\r\n$3\r\nSET\r\n$5\r\nhello\r\n$5\r\nworld\r\n
//! Replies are simple strings, errors, integers, bulk strings or arrays.

/// A parsed client command: the array of argument byte-strings.
#[derive(Debug, PartialEq, Eq)]
pub struct Command(pub Vec<Vec<u8>>);

/// Parse error for malformed RESP2 input.
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    Incomplete,
    Malformed,
}

/// Parses one command array from `buf`. Returns the command and the number of
/// bytes consumed, or `Incomplete` if more data is needed.
pub fn parse_command(buf: &[u8]) -> Result<(Command, usize), ParseError> {
    // ... implemented in Step 4 ...
    # unimplemented!()
}

/// Encodes a simple string reply: +OK\r\n
pub fn simple(s: &str) -> Vec<u8> { format!("+{s}\r\n").into_bytes() }
/// Encodes an error reply: -ERR ...\r\n
pub fn error(s: &str) -> Vec<u8> { format!("-ERR {s}\r\n").into_bytes() }
/// Encodes an integer reply: :42\r\n
pub fn integer(n: i64) -> Vec<u8> { format!(":{n}\r\n").into_bytes() }
/// Encodes a bulk string, or nil ($-1) when `None`.
pub fn bulk(v: Option<&[u8]>) -> Vec<u8> {
    match v {
        None => b"$-1\r\n".to_vec(),
        Some(b) => {
            let mut out = format!("${}\r\n", b.len()).into_bytes();
            out.extend_from_slice(b);
            out.extend_from_slice(b"\r\n");
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_set_command() {
        let input = b"*3\r\n$3\r\nSET\r\n$5\r\nhello\r\n$5\r\nworld\r\n";
        let (cmd, n) = parse_command(input).unwrap();
        assert_eq!(n, input.len());
        assert_eq!(cmd.0[0], b"SET");
        assert_eq!(cmd.0[1], b"hello");
        assert_eq!(cmd.0[2], b"world");
    }

    #[test]
    fn incomplete_returns_incomplete() {
        let input = b"*3\r\n$3\r\nSET\r\n";
        assert_eq!(parse_command(input), Err(ParseError::Incomplete));
    }

    #[test]
    fn encoders_match_resp2() {
        assert_eq!(simple("OK"), b"+OK\r\n");
        assert_eq!(integer(42), b":42\r\n");
        assert_eq!(bulk(None), b"$-1\r\n");
        assert_eq!(bulk(Some(b"hi")), b"$2\r\nhi\r\n");
    }
}
```

- [ ] **Step 3: Ejecutar para ver fallar**

Run: `cargo test -p ag-cache --features native-server resp2`
Expected: FAIL (`parse_command` unimplemented / panics).

- [ ] **Step 4: Implementar `parse_command`**

Reemplazar el cuerpo `unimplemented!()` por un parser que: lee `*<n>\r\n`, luego `n`
bulk strings (`$<len>\r\n<bytes>\r\n`), devolviendo `Incomplete` si faltan bytes y
`Malformed` si los prefijos no cuadran. Devuelve `(Command, bytes_consumed)`.

- [ ] **Step 5: Ejecutar para ver pasar**

Run: `cargo test -p ag-cache --features native-server resp2`
Expected: PASS (3 tests).

- [ ] **Step 6: Commit**

```bash
git add crates/ag-cache/Cargo.toml crates/ag-cache/src/server/
git commit -m "feat(ag-cache): RESP2 codec for native L2 server (RFC-0005)"
```

---

## Task 2: Dispatch de comandos contra L1Cache (TDD)

**Files:**
- Modify: `crates/ag-cache/src/server/mod.rs`

- [ ] **Step 1: Test del dispatcher (sin TCP)**

Escribir un test que construya un `L1Cache`, llame `dispatch(&cache, Command(...))` para
`SET k v` y luego `GET k`, verificando las respuestas RESP2:

```rust
#[tokio::test]
async fn set_then_get_via_dispatch() {
    use crate::l1::L1Cache;
    use std::time::Duration;
    let cache = std::sync::Arc::new(L1Cache::new(100, Duration::from_secs(60)));

    let set = dispatch(&cache, Command(vec![b"SET".to_vec(), b"k".to_vec(), b"v".to_vec()])).await;
    assert_eq!(set, super::resp2::simple("OK"));

    let get = dispatch(&cache, Command(vec![b"GET".to_vec(), b"k".to_vec()])).await;
    assert_eq!(get, super::resp2::bulk(Some(b"v")));

    let miss = dispatch(&cache, Command(vec![b"GET".to_vec(), b"missing".to_vec()])).await;
    assert_eq!(miss, super::resp2::bulk(None));
}
```

(Verificar el API real de `L1Cache` con `grep -n "pub async fn\|pub fn" crates/ag-cache/src/l1.rs` y mapear GET/SET/DEL/EXISTS/EXPIRE/TTL/PING/FLUSHDB/DBSIZE segun la tabla 4.2 de RFC-0005.)

- [ ] **Step 2: Implementar `dispatch`**

```rust
use std::sync::Arc;
use crate::l1::L1Cache;
use crate::server::resp2::{self, Command};

/// Dispatches one parsed command against the L1 cache and returns a RESP2 reply.
/// Supports the subset in RFC-0005 section 4.2; unknown commands return an error reply.
pub async fn dispatch(cache: &Arc<L1Cache>, cmd: Command) -> Vec<u8> {
    let name = cmd.0.first().map(|b| b.to_ascii_uppercase()).unwrap_or_default();
    match name.as_slice() {
        b"PING" => resp2::simple("PONG"),
        b"SET" => { /* parse key/value [EX ...]; cache.set_bytes; */ resp2::simple("OK") }
        b"GET" => { /* cache.get_bytes -> bulk(Some/None) */ resp2::bulk(None) }
        // DEL, EXISTS, EXPIRE, TTL, MGET, MSET, KEYS, FLUSHDB, DBSIZE, COMMAND ...
        _ => resp2::error("unsupported command in ag-cache native server"),
    }
}
```

Implementar cada brazo segun el API de `L1Cache`. Para TTL/EXPIRE, usar la semantica de
moka disponible; si moka no expone TTL por clave, documentar la limitacion en la RFC y
el README (no inventar).

- [ ] **Step 3: Ejecutar y commit**

Run: `cargo test -p ag-cache --features native-server dispatch`
Expected: PASS.

```bash
git add crates/ag-cache/src/server/mod.rs
git commit -m "feat(ag-cache): dispatch RESP2 commands against L1 cache"
```

---

## Task 3: Servidor TCP `NativeCacheServer`

**Files:**
- Modify: `crates/ag-cache/src/server/mod.rs`

- [ ] **Step 1: Implementar el listener**

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// In-process RESP2 cache server (RFC-0005). Binds loopback by default.
pub struct NativeCacheServer {
    listener: TcpListener,
    cache: Arc<L1Cache>,
}

impl NativeCacheServer {
    /// Binds the server to `127.0.0.1:{port}` backed by the given L1 cache.
    pub async fn bind(port: u16, cache: Arc<L1Cache>) -> std::io::Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", port)).await?;
        Ok(Self { listener, cache })
    }

    /// Returns the actual bound port (useful when binding port 0 in tests).
    pub fn local_port(&self) -> std::io::Result<u16> {
        Ok(self.listener.local_addr()?.port())
    }

    /// Serves connections until the listener is dropped. One task per connection.
    pub async fn serve(self) -> std::io::Result<()> {
        loop {
            let (mut socket, _) = self.listener.accept().await?;
            let cache = self.cache.clone();
            tokio::spawn(async move {
                let mut buf = Vec::with_capacity(1024);
                let mut tmp = [0u8; 1024];
                loop {
                    match socket.read(&mut tmp).await {
                        Ok(0) => break,
                        Ok(n) => {
                            buf.extend_from_slice(&tmp[..n]);
                            while let Ok((cmd, consumed)) = resp2::parse_command(&buf) {
                                let reply = dispatch(&cache, cmd).await;
                                if socket.write_all(&reply).await.is_err() { return; }
                                buf.drain(..consumed);
                            }
                        }
                        Err(_) => break,
                    }
                }
            });
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add crates/ag-cache/src/server/mod.rs
git commit -m "feat(ag-cache): native RESP2 TCP server with per-connection tasks"
```

---

## Task 4: Test de compatibilidad con cliente Redis real

**Files:**
- Create: `crates/ag-cache/tests/resp2_compat.rs`
- Modify: `crates/ag-cache/Cargo.toml` (`redis` en dev-deps)

- [ ] **Step 1: Test end-to-end con el crate `redis`**

```rust
//! Verifies the native server speaks enough RESP2 for a real Redis client.
#![cfg(feature = "native-server")]

use std::sync::Arc;
use std::time::Duration;

use ag_cache::l1::L1Cache;
use ag_cache::server::NativeCacheServer;

#[tokio::test]
async fn redis_client_can_set_and_get() {
    let cache = Arc::new(L1Cache::new(100, Duration::from_secs(60)));
    let server = NativeCacheServer::bind(0, cache).await.unwrap();
    let port = server.local_port().unwrap();
    tokio::spawn(async move { let _ = server.serve().await; });

    let client = redis::Client::open(format!("redis://127.0.0.1:{port}/")).unwrap();
    let mut conn = client.get_multiplexed_async_connection().await.unwrap();

    let _: () = redis::cmd("SET").arg("k").arg("v").query_async(&mut conn).await.unwrap();
    let got: String = redis::cmd("GET").arg("k").query_async(&mut conn).await.unwrap();
    assert_eq!(got, "v");

    let pong: String = redis::cmd("PING").query_async(&mut conn).await.unwrap();
    assert_eq!(pong, "PONG");
}
```

`Cargo.toml` dev-deps: `redis = { version = "0.27", features = ["tokio-comp"] }`
(verificar version compatible; ajustar `get_multiplexed_async_connection` al API real).

- [ ] **Step 2: Ejecutar**

Run: `cargo test -p ag-cache --features native-server redis_client_can_set_and_get`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/ag-cache/tests/resp2_compat.rs crates/ag-cache/Cargo.toml
git commit -m "test(ag-cache): RESP2 compatibility with redis client"
```

---

## Task 5: Integrar en `AgCache` + quitar el warning de L2

**Files:**
- Modify: `crates/ag-cache/src/config.rs`, `crates/ag-cache/src/lib.rs`

- [ ] **Step 1: Anadir config**

En `config.rs` anadir a `CacheConfig`:
```rust
/// Start the in-process native RESP2 cache server (RFC-0005). Off by default.
#[serde(default)]
pub native_server_enabled: bool,
/// Port for the native server. Default 6379 (Redis-compatible).
#[serde(default = "CacheConfig::default_native_port")]
pub native_server_port: u16,
```
con `fn default_native_port() -> u16 { 6379 }` y los defaults en `Default`.

- [ ] **Step 2: Arrancar el server y reemplazar el warning**

En `lib.rs` `AgCache::new`, reemplazar el bloque actual del `tracing::warn!` por: si
`native_server_enabled`, `bind` + `tokio::spawn(serve)` (bajo `#[cfg(feature = "native-server")]`);
mantener un mensaje claro si la feature no esta compilada. El `redis_url` deja de ser un
warning: documentar que el L2 distribuido real (Redis) sigue siendo opcional y separado.

- [ ] **Step 3: Verificar ambos modos**

Run: `cargo build -p ag-cache` y `cargo build -p ag-cache --features native-server`
Expected: ambos compilan; sin warning de "L2 no activo" cuando se usa native-server.

- [ ] **Step 4: Commit**

```bash
git add crates/ag-cache/src/config.rs crates/ag-cache/src/lib.rs
git commit -m "feat(ag-cache): wire native RESP2 server into AgCache config"
```

---

## Task 6: README, cierre de deuda y verificacion final

- [ ] **Step 1: README L2 nativo**

Actualizar `crates/ag-cache/README.md`: seccion "L2 nativo (RESP2)" explicando L1 vs L2,
como habilitar (`features = ["native-server"]`, `native_server_enabled`), comandos
soportados (enlazar RFC-0005 tabla 4.2) y limitaciones (single-node, sin persistencia,
loopback). Quitar la advertencia previa de "L2 no funcional".

- [ ] **Step 2: Cerrar DEBT-004**

En `docs/DEBT.md`, DEBT-004 -> `closed (P5, 2026-...)`.

- [ ] **Step 3: Verificacion global**

Run:
```bash
cargo fmt -p ag-cache -- --check
cargo clippy -p ag-cache --features native-server -- -D warnings
cargo test -p ag-cache --features native-server
cargo build --workspace
```
Expected: limpio y verde.

- [ ] **Step 4: Commit**

```bash
git add crates/ag-cache/README.md docs/DEBT.md
git commit -m "docs(ag-cache): document native L2, close DEBT-004"
```

---

## Self-review

- Compuerta RFC-0005 -> Task 0 (aprobacion explicita antes de codigo; respeta CLAUDE.md s5/s22).
- Diseno ya existe en la RFC -> P5 implementa, no inventa arquitectura.
- RESP2 codec -> Task 1 (TDD). Dispatch -> Task 2. Servidor TCP -> Task 3. Compat Redis -> Task 4.
- Integracion + quitar warning -> Task 5. Docs + deuda -> Task 6.
- Tipos consistentes: `resp2::{parse_command, Command, simple, error, integer, bulk}`,
  `dispatch`, `NativeCacheServer::{bind, local_port, serve}`, `CacheConfig.native_server_*`.
- Pendiente de verificar al ejecutar: API real de `L1Cache` (TTL por clave en moka),
  version del crate `redis` y su API async.
