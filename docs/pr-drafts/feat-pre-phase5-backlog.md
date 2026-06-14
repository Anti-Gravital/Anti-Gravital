# Descriptor de PR

## Resumen

Backlog pre-Fase-5: features opt-in tras feature Cargo con el camino nativo por defecto (ag-cache Redis L2)

## Fase afectada

Endurecimiento pre-Fase-5 (Fase 4.x). Trabajo aditivo; cada feature es opt-in y
conserva el comportamiento nativo por defecto (ADR-0009 regla 2). No avanza una
fase de la Hoja de Ruta.

## Tipo de cambio

- [ ] Correccion de bug
- [x] Nueva feature (adaptadores opt-in tras feature Cargo)
- [x] Documentacion (cabeceras, README EN+ES, READMEs de crate)
- [ ] CI / seguridad
- [ ] Cambio que rompe compatibilidad

## Contexto

Esta rama acumula la resolucion de varias issues del backlog pre-Fase-5, cada
una como un commit enfocado. Todas siguen el mismo patron honesto: el mecanismo
agnostico del backend se verifica en proceso (tests que pasan aqui) y el backend
externo real va tras una feature Cargo con su test live `#[ignore]` (el sandbox
no tiene Docker ni servicios externos; CLAUDE.md section 29.2 admite
`#[ignore]` + verificacion manual documentada para dependencias de servicio).

### Issue #144 — ag-cache: capa L2 Redis externa

`ag-cache` tenia solo L1 (moka) y el servidor RESP2 nativo; el L2 Redis externo
estaba diferido (solo avisaba con `REDIS_URL`). RFC-0005 eligio el servidor RESP2
nativo como L2 *por defecto*; ADR-0009 regla 2 permite Redis externo como
adaptador opt-in con el camino nativo por defecto. Esta rama lo anade:

- `crates/ag-cache/src/l2.rs` (nuevo): trait `L2Cache`, `InMemoryL2` (referencia
  + tests) y `RedisL2` tras feature `redis-l2` (cliente `redis` con
  `ConnectionManager`; tags como Redis sets).
- `crates/ag-cache/src/lib.rs`: `AgCache` gana `l2: Option<Arc<dyn L2Cache>>`;
  `get` hace read-through y puebla L1; `set`/`delete`/`invalidate_tag` escriben
  write-through; `with_l2` para inyectar backing; `connect_l2` conecta Redis
  cuando la feature esta activa y hay `redis_url`. Marcador TECH-DEBT eliminado.
- Modelo de consistencia: el L2 es la fuente de verdad compartida; la
  invalidacion cross-instancia se propaga por el read-through; el staleness de L1
  esta acotado por `l1_ttl_secs` (documentado).
- `crates/ag-cache/Cargo.toml`: feature `redis-l2 = ["dep:redis"]`, deps
  `async-trait` y `redis` (opcional), descripcion sin la marca retirada.
- `crates/ag-cache/tests/redis_l2.rs` (nuevo): test live `#[ignore]` con `REDIS_URL`.
- README (raiz EN+ES) y `crates/ag-cache/README.md` sincronizados.

## Plan de prueba

```sh
cargo fmt --check -p ag-cache
cargo clippy -p ag-cache --tests -- -D warnings
cargo clippy -p ag-cache --features redis-l2 --tests -- -D warnings
cargo build  -p ag-cache --features redis-l2
cargo test   -p ag-cache            # 24 tests, incluye los 4 de L2
# Test live (requiere Redis):
REDIS_URL=redis://localhost:6379 \
  cargo test -p ag-cache --features redis-l2 --test redis_l2 -- --ignored
```

## Alcance de verificacion (honesto)

- Verificado aqui: read-through + write-through + invalidacion distribuida (via
  `InMemoryL2` compartido entre instancias) con `fmt`/`clippy`/24 tests en verde,
  con y sin la feature; compila con `redis-l2`.
- NO ejecutado aqui: el round-trip live contra Redis (sin Docker en el sandbox);
  queda como test `#[ignore]` + `REDIS_URL` documentado.

## Criterios de salida que avanza

- L2 distribuido tras feature con el camino nativo por defecto (ADR-0009).
- `REDIS_URL` + feature activa la L2; sin feature solo avisa.
- Invalidacion distribuida cubierta por test (live `#[ignore]`).
- Marcadores TECH-DEBT de `ag-cache` eliminados.

## Cierre de issues

Closes #144

## Checklist final

- [x] Pertenece a la fase correcta y respeta RFC-0005 + ADR-0009 regla 2.
- [x] No rompe arquitectura; el camino L1-only por defecto es identico.
- [x] No crea dependencias circulares; nueva dep `redis` justificada y tras feature.
- [x] Compila; pasa fmt, clippy y tests con y sin features.
- [x] Documentacion sincronizada (modulo, README raiz EN+ES, README del crate).
- [x] Sin evidencia de herramientas IA; atribuido a la persona autora.
