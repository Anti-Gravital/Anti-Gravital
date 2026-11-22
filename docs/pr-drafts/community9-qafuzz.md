# Add reproducible fuzz-window automation and real partial evidence (issue #156)

- Fase afectada: pre-Phase 5 release gate, stage 4 (fuzzing). No roadmap phase
  advance; the `Fuzz 24h` gate row stays `pending`.
- Tipo de cambio: [x] documentacion y automatizacion de evidencia;
  [x] correccion de `fuzz/Cargo.lock`; [ ] codigo de producto;
  [ ] decision arquitectonica.

## Resumen

Automate seven-target fuzz windows with schema-versioned JSONL evidence, record
a real 60 s per-target run, and keep issue #156 open until the 24h gate is run.

## Documentos relacionados

- `docs/fuzz/24h-gate-verification.md` (procedure and acceptance criteria).
- `docs/fuzz/windows-jsonl-schema.md` (new, schema v1).
- `docs/fuzz/wave9-window-evidence.md` (new, real window evidence).
- `docs/fuzz/scheduled-24h-gate-draft.md` (new, runner design note, not enabled).
- `docs/audits/PRE_FASE5_RELEASE_GATE.md` (row unchanged, still `pending`).
- `CLAUDE.md` sections 17, 18, 26, 29 and descriptor lifecycle rules.

## Plan de prueba

- `bash -n scripts/run-fuzz-window.sh`.
- `git diff --check` on all changed files.
- Content scans from `.github/workflows/docs.yml`: prohibited AI attribution,
  AI session URLs, descriptor prefixes, emojis and retired third-party brands.
- JSONL parse: `python3 -c 'import json; [json.loads(l) for l in open("docs/fuzz/results/windows.jsonl")]'`.
- 1-second self-test of the script writing outside the repository: 7 target
  records plus 1 run record, all exit 0, no crashes.
- Real 60 s per-target window: 7 targets, all exit 0, 0 crash artifacts,
  76,367,567 executions, logs with final `DONE` and termination lines.
- Lockfile stability re-check: SHA-256 unchanged across a full seven-target
  invocation after committing the regenerated `fuzz/Cargo.lock`.

## Criterios de salida que avanza

- Provides reproducible automation and honest partial evidence for issue #156;
  does not close any gate criterion that requires an 86400 s run.
- Keeps `Fuzz 24h` at `pending` and issue #156 open with `blocked`.

## Cierre de issues

Closes: none
Refs #156

No issue is closed: the 24-hour gate has not been executed and issue #156
remains open with `blocked`.

## Checklist

- [x] Cambio minimo y dentro de la fase correcta.
- [x] Documentacion actualizada en la misma rama.
- [x] Sin evidencia falsificada: numeros, rutas y hashes tomados de la corrida real.
- [x] `bash -n` y `git diff --check` ejecutados.
- [x] Escaneos de contenido ejecutados.
- [x] Sin push ni merge; sin prefijos de herramientas IA ni emojis.
