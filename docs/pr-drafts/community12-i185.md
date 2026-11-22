# Resumen

Add an opt-in drift monitor for the last chrono edge with baseline evidence and a live upstream re-check; issue #185 stays blocked and open.

- Fase afectada: mantenimiento / higiene de dependencias (ola 12); no altera fases ni puertas del roadmap.
- Tipo de cambio: [x] documentacion y auditoria  [ ] CI/workflow  [ ] codigo de crate  [ ] DSL  [ ] API publica  [x] script local opt-in.
- Documentos relacionados: `docs/audits/chrono-unblock-analysis.md`, `docs/audits/chrono-edge-monitor.md`, `docs/audits/chrono-edge-baseline.sha256`, RFC-0026, RFC-0029, `deny.toml`, `CHANGELOG.md`, CLAUDE.md reglas 16, 19, 22, 24 y 26.

## Cambios

- `scripts/check-chrono-edge.sh`: check local opt-in (no cableado a CI) que ejecuta
  `cargo tree --workspace --all-features -i chrono` y `cargo deny --all-features check`,
  normaliza rutas/ANSI, calcula SHA-256 de la senal compacta y la compara con la
  baseline versionada. Estados: PASS (sin deriva), FAIL (deriva), SKIP (herramienta
  ausente, nunca PASS), ERROR (baseline ausente o malformada). PASS no significa cierre.
- `docs/audits/chrono-edge-baseline.sha256`: hash de referencia y forma de regenerarlo.
- `docs/audits/chrono-edge-monitor.md`: re-verificacion real de upstream el 2026-11-22
  (object_store sigue en 0.14.2, issue #909 abierta sin PR de fix, chrono sin version
  parcheada), evidencia de la ejecucion y semantica de resultados. No repite el analisis
  definitivo; enlaza su seccion 4.
- `CHANGELOG.md`: una entrada bajo [Unreleased] con Refs #185.

## Plan de prueba

- `bash -n scripts/check-chrono-edge.sh`
- `bash scripts/check-chrono-edge.sh --print-baseline` dos veces: mismo hash.
- `bash scripts/check-chrono-edge.sh`: `RESULT: PASS`, exit 0, hash == baseline.
- Baseline sustituida temporalmente por un hash incorrecto: `RESULT: FAIL`, exit 1;
  baseline restaurada y reverificada en PASS / exit 0.
- `git diff --check`.
- Escaneos de contenido: emojis, evidencia de herramientas IA, marcas retiradas
  (ADR-0011) y seccion de cierre de descriptor, segun `.github/workflows/docs.yml`.
- No se ejecutaron los comandos de cierre de #185 con el ignore eliminado: la issue
  sigue bloqueada y no se afirma que la puerta paso.

## Criterios de salida que avanza

- Ninguna casilla de cierre se marca completada; no se cierra #185 ni se mueve fila de puerta.
- Anade deteccion automatica de deriva: si upstream publica un object_store sin chrono
  o cambia el resultado de deny, el check pasa a FAIL y exige re-examinar y actualizar
  la baseline solo tras verificacion completa.
- Cumple la regla 26 (codigo/documentacion juntos) y la regla 24 (sin evidencia falsa).

## Cierre de issues

Refs #185
Closes: none

## Checklist

- [x] Pertenece a la fase correcta (mantenimiento, ola 12).
- [x] Respeta la documentacion y no anade features no contempladas.
- [x] No rompe arquitectura ni anade dependencias circulares.
- [x] No anade complejidad innecesaria: un script y una baseline.
- [x] No aplica compilacion Rust: bash validado con `bash -n`.
- [x] Pasa `git diff --check` y los escaneos de contenido.
- [x] Tiene documentacion (auditoria, baseline, CHANGELOG).
- [x] No inventa resultados ni cierra la issue; #185 conserva su estado bloqueado.
