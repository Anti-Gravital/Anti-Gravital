# Resumen

Enable the scheduled 24h fuzz gate workflow on a dedicated self-hosted runner with least privilege and no pass claim.

- Fase afectada: cambio de CI aditivo (ola 11, cierre de puertas); no altera fases del roadmap.
- Tipo de cambio: [x] documentacion y gobernanza (RFC)  [x] CI/workflow  [ ] codigo de crate  [ ] DSL  [ ] API publica.
- Documentos relacionados: RFC-0035, `docs/fuzz/24h-gate-verification.md`,
  `docs/fuzz/scheduled-24h-gate-draft.md`, `docs/fuzz/windows-jsonl-schema.md`,
  `docs/fuzz/self-hosted-runner.md`, `docs/INDEX.md`, `CHANGELOG.md`,
  CLAUDE.md reglas 16, 22, 28 y 36.

## Cambios

- `docs/rfc/RFC-0035-scheduled-fuzz-24h-gate.md`: RFC aceptada con renuncia
  al periodo de comentarios registrada por el BDFL, requerida por la regla 28
  antes de cualquier cambio de CI.
- `.github/workflows/fuzz-24h.yml`: disparadores `schedule` (mensual) y
  `workflow_dispatch`; `runs-on: [self-hosted, linux, x64, fuzz-dedicated]` con
  guarda adicional sobre `runner.environment`; `permissions: {}` en lo alto y
  `contents: read` / `issues: write` solo donde hacen falta; acciones fijadas
  por SHA; matriz de siete jobs con un target por job y `max-parallel: 1`
  (un job no puede alojar 168 h); invocacion de `scripts/run-fuzz-window.sh`
  con `DURATION=86400` y `FUZZ_TARGETS` por job; subida de evidencia con
  `if: always()`; aviso de fallo idempotente en #156 marcado por `github.run_id`,
  sin cerrar la issue ni mover la fila de puerta. Unico secreto: `GITHUB_TOKEN`.
- `scripts/run-fuzz-window.sh`: entrada opcional `FUZZ_TARGETS` para seleccionar
  un subconjunto de los siete targets canonicos; nombres desconocidos fallan
  cerrados y el comportamiento por defecto no cambia.
- `docs/fuzz/self-hosted-runner.md`: requisitos de host, cuenta de servicio,
  aislamiento de red, dimensionamiento de disco, toolchain fechada, etiquetas
  del runner y comandos de verificacion.
- `docs/fuzz/README.md`, `docs/INDEX.md`, `docs/rfc/README.md`, `CHANGELOG.md`:
  navegacion y registro del cambio.

## Plan de prueba

- `python3 -c 'import yaml,sys; yaml.safe_load(open(sys.argv[1]))' .github/workflows/fuzz-24h.yml`
- `python3 -c 'import yaml,sys; [yaml.safe_load(open(f)) for f in sys.argv[1:]]' .github/workflows/*.yml`
- `bash -n scripts/run-fuzz-window.sh`
- `FUZZ_TARGETS=target_inexistente bash scripts/run-fuzz-window.sh 1` debe
  salir 2 sin ejecutar fuzzing (fallo cerrado).
- `git diff --check`
- Escaneos de contenido prohibido: emojis, evidencia de herramientas IA y
  marcas comerciales retiradas (ADR-0011), segun `.github/workflows/docs.yml`.
- No se ejecuto ningun target durante 86400 s: esto es capacidad, no evidencia.

## Criterios de salida que avanza

- Hace ejecutable el procedimiento documentado de la issue #156 sin afirmar
  que la puerta paso. La fila `Fuzz 24h` sigue `pending` y la issue abierta.
- Cumple la regla 28 (RFC antes de cambiar CI) y la regla 16 (menos privilegios,
  sin secretos adicionales).

## Cierre de issues

Refs #156
Closes: none

## Checklist

- [x] Pertenece a la fase correcta (CI aditivo, ola 11).
- [x] Respeta la documentacion y no anade features no contempladas.
- [x] No rompe arquitectura ni anade dependencias circulares.
- [x] No anade complejidad innecesaria: reutiliza el harness existente.
- [x] Compila / no aplica (sin codigo Rust): YAML y shell validados.
- [x] Pasa `git diff --check` y los escaneos de contenido.
- [x] Tiene documentacion (RFC, guia de runner, indice y changelog).
- [x] No inventa resultados de ejecucion ni cierra la puerta.
