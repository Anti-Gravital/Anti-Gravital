# RFC-0028 - Verificacion en CI del feature opt-in `otlp` de ag-observe

- Estado: aceptada
- Autor: responsable de calidad/observabilidad (issue #149)
- Fecha de borrador: 2026-11-22
- Fase objetivo: Fase 4.7 (calidad); sin cambio de fase, alcance ni roadmap
- Modulos o crates afectados: `ag-observe`, `.github/workflows/ci.yml`
- RFC predecesora: ninguna; complementa la decision ADR-0019 (issue #149)
- Periodo de comentarios: renunciado (ver seccion 9)

## 1. Motivacion

La issue #149 quedo implementada con `Refs #149` (ADR-0019): el feature
opt-in `ag-observe/otlp` existe, el default nativo se conserva y ambos
marcadores `TECH-DEBT` se eliminaron. La issue sigue abierta porque el
criterio de cierre exige una ejecucion verde de CI con el feature activado.
El workspace compila `ag-observe` solo con features por defecto, asi que
ningun job existente ejerce la configuracion `otlp`. Sin cobertura en CI,
la evidencia de cierre depende de una ejecucion manual no reproducible.

## 2. Problema

- Los jobs `ci / build` usan `--workspace` con features por defecto; el
  feature `otlp` no se activa en ningun job.
- El job `quality / deny` usa `--all-features`, que comprueba la resolucion
  de dependencias pero no ejecuta los tests del feature.
- La evidencia inicial de #149 es local y no se reproduce automaticamente
  al fusionar, por lo que el cierre de la issue no puede automatizarse.

Datos: `cargo test -p ag-observe --features otlp` ejecuta 14 tests (1
doc-test ignorado) sin collector ni `protoc`; la compilacion exito con
`PROTOC=/nonexistent/protoc` en un target-dir limpio (2026-11-22).

## 3. Alternativas consideradas

- **No hacer nada:** mantiene #149 abierta sin evidencia reproducible.
  Rechazada; incumple el criterio de cierre de la issue.
- **Anadir `otlp` a la matriz por defecto (`--all-features` en `build`):**
  Rechazada; cambiaria la matriz de features por defecto y aumentaria el
  costo de los cuatro runners sin una senal de fallo mas clara.
- **Anadir un job dedicado de feature-on (elegida):** senal aislada,
  matriz por defecto intacta, misma forma que el job existente
  `ci / ag-mail mta + api features` (RFC-0009 seccion 4.8).

## 4. Diseno propuesto

Anadir un job `otlp` a `.github/workflows/ci.yml`, modelado sobre el job
`mail-mta`:

- `runs-on: ubuntu-latest` con toolchain stable y componente clippy.
- Cache de cargo con clave propia (`${{ runner.os }}-cargo-otlp-...`).
- Pasos:
  - `cargo check --locked -p ag-observe --features otlp --all-targets`
  - `cargo test --locked -p ag-observe --features otlp`
  - `cargo clippy --locked -p ag-observe --features otlp --all-targets -- -D warnings`

Restricciones explicitas de esta RFC:

1. **El job NO hace obligatorio `protoc`.** No se anade ningun paso de
   aprovisionamiento de protobuf compiler en este job ni en los demas.
   `opentelemetry-otlp` 0.33.x usa `http-proto` con codigo protobuf
   pregenerado en `opentelemetry-proto`; no hay `build.rs` que invoque
   `protoc` (ADR-0019).
2. **El job NO hace obligatorio un collector externo.** Los tests del
   feature construyen el exportador HTTP contra un endpoint sintactico
   (`http://127.0.0.1:4318/v1/traces`) sin contactarlo; no se inicia
   ningun servicio ni se requiere `OTEL_EXPORTER_OTLP_ENDPOINT` real.
3. El feature `otlp` sigue desactivado por defecto (ADR-0009 regla 2 y
   ADR-0019 decision 1). La matriz de features por defecto no cambia.
4. No hay cambios en APIs publicas, dependencias, DSL ni documentos
   maestros. Solo se anade un job de verificacion y se actualizan el
   indice de RFC, el CHANGELOG y una nota de trazabilidad en ADR-0019.

Nota de coherencia documental: ADR-0019 decision 4 afirmo que no hacia
falta un cambio de CI porque no se necesita `protoc`. Esta RFC no
contradice ese hallazgo: anade cobertura de verificacion del feature, sin
introducir `protoc` ni collector. ADR-0019 queda ampliado con una nota
que apunta a esta RFC.

## 5. Plan de implementacion

Unico PR en la rama `community2/otlp`: RFC + job de CI + indice de RFC +
nota en ADR-0019 + CHANGELOG + descriptor de PR. Sin PRs posteriores.

## 6. Riesgos

- **Falso positivo por cache:** bajo. Mitigacion: clave de cache propia y
  ejecucion local con target-dir limpio y `PROTOC=/nonexistent/protoc`.
- **El job pase pero el default se rompa:** bajo. Mitigacion: la matriz
  por defecto no se toca y los jobs existentes siguen activos.
- **Confusion con ADR-0019 decision 4:** bajo. Mitigacion: nota explicita
  de que no se introduce `protoc` ni collector.

## 7. Impacto

- Alcance del producto: ninguno.
- Roadmap: ninguno.
- Complejidad operacional: un job adicional en `ci`, sin servicios nuevos.
- APIs publicas: ninguno.
- Documentacion: indice de RFC, ADR-0019 y CHANGELOG.

## 8. Rollback

Revertir el commit que anade el job en `ci.yml`. Indicadores de rollback:
fallo sostenido del job por dependencias externas o necesidad de
aprovisionar `protoc`/collector (lo que invalidaria la premisa ADR-0019).

## 9. Decision

- Decisor: BDFL (responsable del proyecto, resolucion de la issue #149).
- Fecha de decision: 2026-11-22.
- Resultado: aceptada.
- Periodo de comentarios: renunciado. Justificacion: cambio aditivo de
  verificacion sobre una decision ya aceptada (ADR-0019); sin impacto en
  API publica, alcance, roadmap ni features por defecto.
- Justificacion: hace reproducible y automatica la evidencia de cierre de
  #149 sin debilitar el default nativo ni introducir requisitos de build.

## 10. Referencias

- Issue #149 (ag-observe: OTLP exporter deferred).
- ADR-0019 (ag-observe OTLP transport without protoc).
- ADR-0009 regla 2 (native default preserved).
- RFC-0009 seccion 4.8 (precedente del job `mail-mta` en `ci.yml`).
- Reglas CLAUDE.md 28 (RFC para cambios de CI) y 35 (ADR).
