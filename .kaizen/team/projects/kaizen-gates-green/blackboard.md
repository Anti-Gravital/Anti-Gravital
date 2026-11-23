# Blackboard: kaizen-gates-green

Solo hechos con `source_ids` verificados o salida de comando reproducible.


## F1 platform-engineer — tests/run.sh (2026-11-22)

- Que hace: `bash tests/run.sh` corre 29 selftests deterministicos (sin red, secretos ni Rust): archivos canonicos, `bash -n` de scripts del gate, parseo de `kaizen-insights.py`, `ag-check --list` de 8 compuertas y honestidad de ag-check (false->1, true->0, --bogus->2, SKIP nunca PASS).
- `bash tests/run.sh` -> exit 0; "29 pruebas pasadas, 0 falladas".
- `scripts/kaizen check --only tests` -> exit 2: modo no soportado por el CLI (ruta protegida). Gate real validado con `scripts/kaizen check` -> "ok: tests" (exit global 1 por fallos preexistentes swarm/gob/deuda, otras tareas).
- `bash scripts/ag-check` -> FALLO preexistente en `masters` (hash de ANTI-GRAVITAL-Arquitectura-Tecnica.md != docs/master/VERSION.md); no causado por este cambio.
- `git diff --check` -> exit 0. `scripts/kaizen check --only descriptors` -> exit 0.
- Commit en rama community14/f1-runsh; sin push/PR/merge.
