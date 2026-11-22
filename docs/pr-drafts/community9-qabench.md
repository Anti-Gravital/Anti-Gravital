# community9/qabench

## Resumen

Informative criterion benchmark evidence on the wave-9 host for issue #157; gate row stays pending.

## Fase afectada

Stage 6 (benchmarks) of the pre-Phase-5 release gate. No roadmap phase advances:
this is supporting evidence while the reference-hardware procedure remains
unexecuted. No source crate or CLI behaviour changed.

## Tipo de cambio

- [ ] Code
- [ ] RFC / ADR
- [x] Documentation / measurement evidence
- [x] Tooling (reproducible script)
- [ ] Dependency change
- [ ] Performance target change

## Documentos relacionados

- `docs/benchmarks/verification-package.md` (Stage 6 reproducible procedure).
- `docs/benchmarks/wave9-host-results.md` (new: real informative-host results).
- `docs/benchmarks/raw/wave9-2026-11-22/` (metadata and estimates summary).
- `docs/audits/PRE_FASE5_RELEASE_GATE.md` (Benchmarks row still pending).
- `docs/benchmarks/smoke-2026-11-22.md` (prior reduced-sample smoke, not evidence).
- `CLAUDE.md` rule 17 (benchmark metadata) and ADR-0008 (English technical docs).

## Plan de prueba

Reviewer commands, from the repository root:

```sh
bash -n scripts/run-benchmarks.sh
git diff --check
SAMPLE_SIZE=30 WARMUP_TIME=2 MEASUREMENT_TIME=3 ./scripts/run-benchmarks.sh
```

The script must exit 0, dump rule-17 metadata and print standard deviation
values from `target/criterion/<bench>/new/estimates.json`. No number may be
published as official: the host is mobile-class, uses the `powersave` governor
and has no load generator installed.

## Criterios de salida que avanza

None is marked complete. The Stage 6 Benchmarks exit criterion (issue #157)
gains a reproducible informative dataset but stays pending; the gate row keeps
`Blocks Phase 5 = yes`.

## Cierre de issues

Closes: none
Refs #157

## Checklist

- [x] El resumen tiene 256 caracteres o menos y es usable como titulo del PR.
- [x] La fase afectada y el tipo de cambio estan identificados.
- [x] Los documentos relacionados estan enumerados con rutas concretas.
- [x] El plan de prueba contiene comandos ejecutables y criterios de exito.
- [x] La seccion de cierre de issues incluye una palabra clave de cierre.
