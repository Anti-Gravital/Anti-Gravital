# Resumen

RFC-0027: evaluar chumsky 0.9 frente a la linea 0.13/0.14 y recomendar diferir la migracion del parser DSL

## Fase afectada

Fase 3 (contrato de librerias del compilador, enmienda de RFC-0003). Sin cambio de fase ni de roadmap.

## Tipo de cambio

- [ ] Codigo
- [ ] Dependencias / manifiestos
- [x] Documentacion tecnica (RFC)
- [x] Propuesta de decision arquitectonica
- [ ] CI / tooling
- [ ] DSL / gramatica

## Documentos relacionados

- `docs/rfc/RFC-0027-chumsky-parser-upgrade.md` (nuevo)
- `docs/rfc/RFC-0003-librerias-compilador-ag-dsl.md` (enmienda propuesta en secciones 3.2, 4.1, 6 y 9)
- `docs/rfc/README.md`, `CHANGELOG.md`
- Issues #180 (evaluacion), #156 (gate fuzz 24h), #157 (evidencia benchmarks)
- `docs/audits/pre-fase5-fuzzing.md`, `crates/ag-dsl/src/parser.rs`

## Plan de prueba

Verificacion de esta entrega (solo documentacion):

1. `git diff --check` en la rama.
2. `grep -n '0027' docs/rfc/README.md` y lectura del indice para confirmar numero libre (0025 y 0026 reclamados por otros roles).
3. Revision del spike de API compilado fuera del repo: `/tmp/opencode/spike` con `chumsky 0.13.0` (no versionado).
4. Revision de la medicion local con metadata Rule 17: `/tmp/opencode/spike-bench` (no versionado).
5. Si el comite decide migrar, ejecutar primero los PR 0 y 1 del plan de la RFC antes de tocar `parser.rs`.

No se ejecuta `cargo test` ni `cargo clippy`: no se cambio ningun archivo Rust ni manifiesto.

## Criterios de salida que avanza

- Issue #180: resolution de la RFC que pide el issue (recomendacion registrada). Queda abierto hasta decision del comite y cierre del coordinador.
- RFC-0003: se resuelve el placeholder "RFC futura por asignar" de las secciones 6, 8 y 9.

## Cierre de issues

Refs #180
Closes: none

## Checklist

- [x] Pertenece a la fase correcta (Fase 3, amendment).
- [x] Respeta la documentacion (ADR-0008: RFC tecnica en ingles; sin atribucion de herramientas IA).
- [x] No rompe arquitectura ni anade dependencias.
- [x] No anade complejidad innecesaria.
- [x] Sin codigo nuevo; no aplica fmt/clippy/test/audit/deny (audit consultado como evidencia, sin cambios).
