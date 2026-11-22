# Descriptor de Pull Request

## Resumen

Wave 10 real-terminal evidence for the full non-interactive scaffold-to-HTTP journey, with timings and one README defect reported as issue #205.

## Fase afectada

Alpha release verification (wave 10, real terminal QA). No roadmap phase is
advanced: this change adds audit evidence and a defect report only.

## Tipo de cambio

- [ ] Code
- [ ] RFC / ADR
- [x] Documentation / audit evidence
- [x] Issue report (no issue closed)
- [ ] CI / tooling

## Documentos relacionados

- `docs/audits/scaffold-user-journey.md` (new evidence document)
- `CHANGELOG.md` (Unreleased entry)
- `docs/examples/alpha-prototype.md` (offline prototype guide, correct bind behaviour)
- Issue #191 (CLOSED), issue #205 (new, documentation)

## Plan de prueba

The audit document records every command with its exact cwd, environment,
exit code, elapsed time and verbatim output. Reproducer summary:

```
ag --version
ag new demo --template rest --no-interactive
cd demo && cargo check --offline
ag schema lint
ag generate
ag build
ag dev --bind 127.0.0.1:8080 & curl http://127.0.0.1:8080/
ag new website --template static-site --no-interactive
ag new hooks --template webhook-gateway --no-interactive
ag new layeredmod --template layered --with auth --with cache --no-interactive
cd website && cargo check --offline
cd hooks && cargo check --offline
cd layeredmod && cargo check --offline
git diff --check
```

The `ag` binary was built with a private `CARGO_TARGET_DIR` and
`CARGO_INCREMENTAL=0` to avoid overwriting other agents' binaries. Generated
projects used per-project private target directories; `--offline` resolution
and dependency sources were unchanged.

## Criterios de salida que avanza

- Real-user terminal verification for the alpha scaffold path: executed.
- Evidence of a running HTTP prototype from a fresh binary: HTTP 200 captured.
- Honest issue board: stale README claim reported, nothing falsely closed.

## Cierre de issues

Closes: none
Refs #205

## Checklist

- [x] La rama y el mensaje de commit respetan el limite de 256 caracteres.
- [x] No hay evidencia de herramientas IA, emojis ni marcas prohibidas.
- [x] La documentacion afectada se actualiza en el mismo cambio.
- [x] `git diff --check` y los escaneos de contenido pasan.
- [x] No se cierra ningun issue sin verificacion.
