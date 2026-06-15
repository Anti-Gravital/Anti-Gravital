# Descriptor de PR

## Resumen

ag-mail: spool durable opt-in para la cola programada del MTA nativo (PostgreSQL tras feature `queue-postgres`)

## Fase afectada

Fase 4.6-B (motor MTA nativo de `ag-mail`). Trabajo aditivo gobernado por
RFC-0009 section 4.2; no avanza una fase de la Hoja de Ruta.

## Tipo de cambio

- [ ] Correccion de bug
- [x] Nueva feature (spool durable del MTA tras feature Cargo)
- [x] Documentacion (cabeceras de modulo, README, DEBT-023)
- [ ] CI / seguridad
- [ ] Cambio que rompe compatibilidad

## Contexto

La cola de dos niveles del MTA nativo (`sender::mta::queue::MtaQueue`) era solo
en memoria: un reinicio perdia los jobs programados (no entregados todavia)
(DEBT-023 / issue #151). Esta PR anade un spool durable opt-in.

El mecanismo de durabilidad es agnostico del backend: la cola en memoria sigue
siendo la fuente de verdad en runtime y espeja (write-through) su conjunto
programado a un `Spool` (`upsert` al encolar y en cada reprogramacion, `remove`
al salir de la cola, `load_all` para recuperar al arrancar). El nivel en memoria
se mantiene como default (ADR-0009 regla 2): sin spool adjunto, el comportamiento
es identico al anterior.

## Cambios

- `crates/ag-mail/src/sender/mta/spool.rs` (nuevo): trait `Spool`, tipo
  `PersistedJob` (timestamps wall-clock en vez de `Instant`), `InMemorySpool`
  (referencia + tests) y `PostgresSpool` tras feature `queue-postgres`. Usa la
  API `sqlx` en runtime (no las macros), asi el crate compila sin `DATABASE_URL`.
  `recipients` mapea a `TEXT[]` y el mensaje a `BYTEA` (sin serializacion extra).
- `crates/ag-mail/src/sender/mta/queue.rs`: `DeliveryJob` gana un `id` estable
  (128 bits aleatorios) para indexar el spool; `MtaQueue` gana un
  `Option<Arc<dyn Spool>>` y un par base `Instant`/`SystemTime` para convertir
  monotonic a wall-clock; metodos `with_spool`, `enqueue_persistent`, `recover`
  y el espejado best-effort en `process_due`. Cuatro tests del mecanismo.
- `crates/ag-mail/Cargo.toml`: feature `queue-postgres = ["mta", "dep:sqlx"]`;
  `getrandom` opcional anadido a `mta`.
- `crates/ag-mail/tests/mta_spool_postgres.rs` (nuevo): test live `#[ignore]`
  contra PostgreSQL via `DATABASE_URL`, autolimpiante.
- `README.md` (EN + mirror ES) y `docs/DEBT.md` DEBT-023 sincronizados.

## Plan de prueba

```sh
cargo fmt --check -p ag-mail
cargo clippy -p ag-mail -- -D warnings
cargo clippy -p ag-mail --features mta -- -D warnings
cargo clippy -p ag-mail --features queue-postgres --tests -- -D warnings
cargo build  -p ag-mail --features queue-postgres   # sqlx runtime: sin DB en build
cargo test   -p ag-mail --features mta              # incluye los 4 tests del spool
# Test live (requiere PostgreSQL):
DATABASE_URL=postgres://... \
  cargo test -p ag-mail --features queue-postgres --test mta_spool_postgres -- --ignored
```

## Alcance de verificacion (honesto)

- Verificado en este entorno: el mecanismo de durabilidad
  (`scheduled_jobs_survive_restart_via_spool`, `delivered_job_is_removed_from_spool`,
  `retried_job_is_updated_in_spool`, `enqueue_persistent_without_spool_is_inmemory`)
  pasa en proceso con `InMemorySpool`; `fmt`/`clippy`/`build`/109 tests en verde
  con y sin features.
- NO ejecutado aqui: el round-trip live contra PostgreSQL. El sandbox no tiene
  daemon Docker ni servidor PostgreSQL local, asi que ese test queda `#[ignore]`
  + `DATABASE_URL` documentado (CLAUDE.md section 29.2 lo admite para
  dependencias de servicio externo). Debe correrse en CI/entorno con credenciales.

## Criterios de salida que avanza

- Spool durable tras feature con el nivel en memoria como default (ADR-0009).
- Jobs programados sobreviven a un reinicio (cubierto por test; live `#[ignore]`).
- DEBT-023 referencia y describe la implementacion.

## Cierre de issues

Closes #151

## Checklist final

- [x] Pertenece a la fase correcta (4.6-B) y respeta RFC-0009 section 4.2.
- [x] No rompe arquitectura; el camino en memoria por defecto es identico.
- [x] No crea dependencias circulares; `getrandom`/`sqlx` ya estaban en el workspace.
- [x] Compila; pasa fmt, clippy y tests con y sin features.
- [x] Documentacion sincronizada (modulo, README EN+ES, DEBT-023).
- [x] Sin evidencia de herramientas IA; atribuido a la persona autora.
