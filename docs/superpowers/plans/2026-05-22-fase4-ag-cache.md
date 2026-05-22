# ag-cache — Plan de Implementacion

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Cache multinivel: moka (L1, en memoria, siempre disponible) + Redis via fred (L2, opcional). Invalidacion por tags en ambos niveles. Objetivo: >= 1M ops/seg en L1.

**Architecture:** Trait `CacheBackend` en ag-core. `AgCache` implementa el trait y orquesta L1+L2. Si Redis no esta configurado, solo L1 activo. Rama: `fase-4/ag-cache`.

**Precondicion:** `fase-4/ag-observe` mergeado a `fase-4`.

**Tech Stack:** `moka` (TinyLFU), `fred` (Redis async), `testcontainers` + `testcontainers-modules` (Redis).

**Spec:** `docs/superpowers/specs/2026-05-22-fase4-modulos-estandar-design.md` seccion ag-cache.
**Arquitectura:** `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` seccion 8.4.

---

## Mapa de archivos

### Crear
- `crates/ag-cache/Cargo.toml`
- `crates/ag-cache/src/lib.rs` — `AgCache`, `CacheConfig`, re-exports
- `crates/ag-cache/src/config.rs` — `CacheConfig::from_env()`
- `crates/ag-cache/src/l1.rs` — wrapper moka con soporte de tags
- `crates/ag-cache/src/l2.rs` — wrapper Redis/fred con soporte de tags
- `crates/ag-cache/src/tags.rs` — indice de tags (tag -> Set<key>)

### Modificar
- `crates/ag-core/src/lib.rs` o nuevo `crates/ag-core/src/cache.rs` — trait `CacheBackend`
- `Cargo.toml` (root) — dependencias nuevas + miembro `crates/ag-cache`

---

## Task 1: Cargo.toml y trait CacheBackend en ag-core

- [ ] Agregar a `[workspace.members]`: `"crates/ag-cache"`
- [ ] Agregar a `[workspace.dependencies]`:
  ```toml
  moka  = { version = "0.12", features = ["future"] }
  fred  = { version = "10", features = ["tokio-runtime", "codec"] }
  ```
- [ ] Crear `crates/ag-cache/Cargo.toml` con deps: `moka`, `fred`, `tokio`, `serde`, `tracing`, `ag-core`, `testcontainers`/`testcontainers-modules` en `[dev-dependencies]`
- [ ] Definir trait `CacheBackend` en `crates/ag-core/src/cache.rs`:
  ```rust
  #[async_trait::async_trait]
  pub trait CacheBackend: Send + Sync {
      async fn get_bytes(&self, key: &str) -> Option<Vec<u8>>;
      async fn set_bytes(&self, key: &str, value: Vec<u8>, ttl: std::time::Duration);
      async fn delete(&self, key: &str);
      async fn invalidate_tag(&self, tag: &str);
  }
  ```
- [ ] Re-exportar desde `ag_core::cache`
- [ ] Verificar: `cargo check --workspace 2>&1 | grep "^error" | head -5`
- [ ] Commit: `chore(cache): Cargo.toml, deps workspace, trait CacheBackend en ag-core`

---

## Task 2: CacheConfig

**Files:** `crates/ag-cache/src/config.rs`

- [ ] TDD: `cache_config_l2_disabled_when_no_redis_url` — sin `REDIS_URL` en env, `config.redis_url` es `None`
- [ ] Implementar `CacheConfig` con: `l1_max_capacity: u64` (default 10_000), `l1_ttl_secs: u64` (default 300), `redis_url: Option<String>` (de `REDIS_URL`), `redis_pool_size: u32` (default 10)
- [ ] Correr test, commit: `feat(cache): CacheConfig from_env`

---

## Task 3: L1 — moka con tags

**Files:** `crates/ag-cache/src/l1.rs`, `crates/ag-cache/src/tags.rs`

- [ ] TDD (unitarios, sin infra):
  - `l1_set_and_get_roundtrip` — serializar valor, guardarlo, recuperarlo, deserializar
  - `l1_tag_invalidation_removes_tagged_entries` — insertar dos entradas con tag "users", invalidar tag, verificar que ambas desaparecen
  - `l1_ttl_expiry` — insertar con TTL de 1s, esperar 2s con `tokio::time::sleep`, verificar `None`
  - `l1_ops_per_second_exceeds_1m` — insertar y leer 1_000_000 entradas, verificar duracion < 1s (benchmark basico en test)
- [ ] Implementar `L1Cache { inner: moka::future::Cache<String, Vec<u8>>, tags: Arc<Mutex<TagIndex>> }`
- [ ] `TagIndex` en `tags.rs`: `HashMap<String, HashSet<String>>` (tag -> keys). `insert(key, tags)`, `remove(key)`, `keys_for_tag(tag) -> Vec<String>`
- [ ] Correr tests (el test de throughput puede ser `#[ignore]` si es lento en CI pero debe existir)
- [ ] Commit: `feat(cache): L1 moka con invalidacion por tags`

---

## Task 4: L2 — Redis via fred con tags

**Files:** `crates/ag-cache/src/l2.rs`

- [ ] TDD (integracion con testcontainers Redis):
  - `l2_set_and_get_roundtrip` — SET y GET contra Redis real
  - `l2_tag_invalidation_removes_tagged_entries` — usar SET de Redis para el indice de tags (`SADD`, `SMEMBERS`, `DEL`)
  - `l2_ttl_is_set_in_redis` — verificar con `TTL key` que el TTL esta configurado
- [ ] Correr para verificar que fallan
- [ ] Implementar `L2Cache { client: fred::clients::RedisPool }` con `new(redis_url, pool_size) -> Result<Self>`
- [ ] Para tags en Redis: usar key `tag:{nombre}` como Redis SET que almacena los keys con ese tag. En `invalidate_tag`: `SMEMBERS tag:{tag}` -> `DEL` todos los keys -> `DEL tag:{tag}`
- [ ] Correr tests (levantan contenedor Redis automaticamente)
- [ ] Commit: `feat(cache): L2 Redis/fred con invalidacion por tags`

---

## Task 5: AgCache facade

**Files:** `crates/ag-cache/src/lib.rs`

- [ ] TDD (integracion con testcontainers Redis para el test con L2):
  - `ag_cache_uses_only_l1_without_redis_config` — sin `redis_url`, operaciones funcionan via L1
  - `ag_cache_l2_fallthrough_on_l1_miss` — borrar de L1, verificar que se lee de L2
- [ ] Implementar `AgCache`:
  ```rust
  pub struct AgCache { l1: L1Cache, l2: Option<L2Cache> }
  impl AgCache {
      pub async fn new(config: CacheConfig) -> Result<Self, CacheError>
      pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T>
      pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Duration)
      pub async fn set_tagged<T: Serialize>(&self, key: &str, value: &T, ttl: Duration, tags: &[&str])
      pub async fn invalidate_tag(&self, tag: &str)
      pub async fn delete(&self, key: &str)
  }
  ```
- [ ] Logica: `get` busca en L1 primero, si miss busca en L2 y repuebla L1; `set` escribe en ambos niveles
- [ ] Integrar con ag-observe: `tracing::debug!` en hits/misses, counter `ag_cache_hits_total{level}` y `ag_cache_misses_total`
- [ ] Correr todos los tests del crate
- [ ] Commit: `feat(cache): AgCache facade L1+L2 con metricas`

---

## Task 6: Verificacion final

- [ ] `cargo fmt --all && cargo clippy -p ag-cache -- -D warnings`
- [ ] `cargo test -p ag-cache 2>&1 | tail -10`
- [ ] Ejecutar el benchmark de throughput L1 manualmente: `cargo bench -p ag-cache` (crear suite criterion basica con `get`/`set` de 1000 entradas)
- [ ] `cargo doc -p ag-cache --no-deps 2>&1 | grep "^error" | head -5`
- [ ] Commit: `docs(cache): docstrings y benchmark criterion L1`
- [ ] Merge: `git checkout fase-4 && git merge --no-ff fase-4/ag-cache -m "feat(cache): ag-cache completo — L1 moka + L2 Redis + tags"`
