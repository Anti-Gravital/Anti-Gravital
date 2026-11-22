# Diario Kaizen — 2026-11-22 19:20 UTC

## Tarea
Implementar RFC-0038 (alcance por proyecto y salud de componentes en ag-observe) tras la aprobacion BDFL, respetando K-1/K-7/L-6/L-7.

## Decision
- No se creo crate nuevo: todo vive en `crates/ag-observe` (rules 5/14/28, RFC-0018).
- `AG_PROJECT_ID` se modela como etiqueta logica opcional y acotada (1-64 chars, `[A-Za-z0-9_.-]`), no como limite de seguridad ni identificador multi-tenant. La semantica profunda de "proyecto" sigue abierta, pero la implementacion no la inventa.
- Un valor invalido se rechaza en `init()` con `ObserveError::InvalidScope` y el mensaje NO incluye el valor (S-1/S-8).
- La etiqueta se anade como global label de Prometheus y como atributo `project.id` del recurso OTLP. Sin feature `otlp`, no se requiere collector (ADR-0009/ADR-0019).
- `HealthRegistry` es nativo y under demanda: sin poller, scheduler ni servicio externo. Liveness no evalua dependencias; readiness devuelve 503 si algun componente falla.

## Evidencia (comandos y resultados reales)
- `cargo test -p ag-observe`: 22 passed, 0 failed (+1 doc-test).
- `cargo test -p ag-observe --features otlp`: 24 passed, 0 failed (+1 doc-test).
- `cargo fmt --check -p ag-observe`: OK.
- `cargo clippy -p ag-observe --all-targets -- -D warnings` (default y `--features otlp`): OK.
- `git diff --check`: OK.
- `scripts/kaizen check`: sigue FALLO por `tests`, `swarm`, `gobernanza` y `ratchet` (hallazgos preexistentes documentados en el outbox). No se relajo ni silencio nada.

## Entregables
- `ObserveConfig.project_id`, `is_valid_project_id`, `ObserveError::InvalidScope`.
- `health.rs`: `HealthRegistry`, `ComponentHealth`, `HealthSnapshot`, `liveness_handler`, `readiness_handler`.
- Recorder Prometheus con etiqueta `project`; recurso OTLP con atributo `project.id`.
- Tests de validacion, render de etiqueta, atributo OTLP, agregacion de readiness e independencia de liveness.
- Documentacion actualizada en el mismo cambio: README del crate, capitulo 14, seccion 14.4 del maestro (EN y ES) y CHANGELOG.

## Resultado
RFC-0038 implementada y verificada en verde con y sin OTLP. No se ejecuto `git add`, `git commit` ni `git push`. Los cambios quedan en el arbol de trabajo; el HEAD sigue en `5ff8e5d` por actividad ajena.

## Aprendizaje
Los documentos de arquitectura afirmaban capacidades (percentiles, correlation ID, exportacion a Loki) que el codigo no produce. La correccion honesta es declarar el alcance implementado en una seccion nueva (14.4) en lugar de borrar el diseno objetivo o inventar implementacion (K-1/regla 26).

## Decisiones humanas pendientes
- No hay nuevas decisiones de esta iteracion. Siguen abiertas las de iteraciones anteriores: puertas de gobernanza, `oha`/hardware para RFC-0039, evidencia de contratos de outsourcing, ADR del contrato de errores (RFC-0037) y semantica profunda de "proyecto".
