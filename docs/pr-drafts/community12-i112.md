# Resumen

Add a dependency-free producer port to ag-edge so the edge can emit deferred-work enqueue requests without depending on ag-workers.

- Fase afectada: Fase 4.6-D, etapa S6 (RFC-0012); avance parcial, no cierra la fase.
- Tipo de cambio: [x] codigo de crate  [x] API publica (feature-gated)  [x] ejemplo  [x] documentacion  [ ] DSL  [ ] CI.
- Documentos relacionados: RFC-0012 (secciones 4, 5 S6), ADR-0013,
  `docs/audits/issue-board-reconciliation.md`,
  `docs/audits/infrastructure-review-112-149-141.md`,
  `crates/ag-edge/README.md`, `crates/ag-workers/README.md`,
  `docs/roadmap/STATUS.md`, `CHANGELOG.md`, CLAUDE.md reglas 5, 14, 15, 30.

## Cambios

- `crates/ag-edge/Cargo.toml`: feature `producer` (desactivada por defecto) que
  habilita solo `dep:async-trait`, dependencia ya presente en el workspace y en los
  dev-dependencies del crate. No se anade dependencia `ag-edge -> ag-workers` ni
  inversa.
- `crates/ag-edge/src/producer.rs`: puerto `EnqueueRequest`, `DeferredJobProducer`,
  `ProducerError` y `emit`. `ag-edge` posee solo el contrato; el adaptador lo
  aporta el crate consumidor, como `DnsProvider`. Test unitario del contrato.
- `examples/workers-producer-edge/src/bridge.rs`: adaptador que traduce el puerto
  a `ag-workers` (`NewJob` + `QueueBackend::enqueue`) y test con `MemoryQueue`
  que verifica enqueue por el puerto y consumo en un camino de lease separado.
- `examples/workers-producer-edge/src/main.rs`: el ejemplo usa el puerto de extremo
  a extremo, manteniendo el modelo productor-only sin worker loop.
- Documentacion y CHANGELOG actualizados en la misma PR.

## Plan de prueba

- `cargo fmt --all -- --check`
- `cargo test -p ag-edge --features producer` (24 tests, incluido
  `producer::tests::emit_forwards_request_to_adapter`).
- `cargo test -p ag-workers` (verde; sin cambios de codigo).
- `cargo test -p workers-producer-edge` (1 test: enqueue por puerto + lease/ack
  en backend en memoria).
- `cargo run -p workers-producer-edge`: 4 jobs enqueueados por el puerto y 4
  entregados por la flota separada.
- `cargo clippy -p ag-edge --all-features --all-targets -- -D warnings` y
  `cargo clippy -p workers-producer-edge --all-targets -- -D warnings`: limpios.
- `cargo tree -p ag-edge -e normal` con y sin `--features producer`: sin
  `ag-workers` en ninguno de los dos arboles (verificado).
- `git diff --check`; escaneos de emojis, evidencia de herramientas IA y marcas
  retiradas (ADR-0011) sobre los archivos cambiados.

## Criterios de salida que avanza

- RFC-0012 S6 deja de ser solo un patron documentado: existe un puerto real en
  `ag-edge` y un adaptador probado contra el backend en memoria de `ag-workers`.
- Direccion de dependencias aciclica y sin acoplamiento especulativo: `ag-edge`
  no conoce `ag-workers` y `ag-workers` sigue sin conocer `ag-edge`.
- La puerta pre-Fase 5 no se mueve: no se afirma produccion/GA.

## Cierre de issues

Refs #112
Closes: none

## Checklist

- [x] Pertenece a la fase correcta (4.6-D S6).
- [x] Respeta la documentacion; la RFC ya contempla el wiring producer/edge.
- [x] No rompe arquitectura ni anade dependencias circulares.
- [x] No anade complejidad innecesaria: un trait, un tipo de dato y un adaptador.
- [x] Compila y pasa tests (`fmt`, `test`, `clippy`).
- [x] Tiene documentacion y trazabilidad (CHANGELOG, README, STATUS).
- [x] No cierra #112: falta un evento concreto dentro de `ag-edge` que emita por
  el puerto (la renovacion ACME vive en `ag-domains`). Requiere ademas la
  verificacion completa descrita en la reconciliacion antes de `Closes #112`.
