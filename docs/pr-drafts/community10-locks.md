# Audit lockfile reproducibility and silent-mutation paths in local automation

- Fase afectada: wave 10 real-terminal QA (release reproducibility). No roadmap
  phase advance and no release-gate row changed.
- Tipo de cambio: [x] auditoria y documentacion; [ ] codigo de producto;
  [ ] decision arquitectonica; [ ] cambio de fase.

## Resumen

Enumerate all four Cargo.lock files, verify each resolves with `--locked`,
reproduce silent lock mutation in local automation scripts and record exact
commands, hashes, timings and remediation in a new audit document.

## Documentos relacionados

- `docs/audits/lockfile-user-journey.md` (new, full evidence and remediation).
- `docs/fuzz/wave9-window-evidence.md` (wave-9 stale fuzz lock precedent).
- `docs/rfc/RFC-0022-maintainability-guardrails.md` (CI `--locked` policy).
- `CHANGELOG.md` (new Unreleased entry).
- `CLAUDE.md` sections 17, 26, 29, 33 and descriptor lifecycle rules.

## Plan de prueba

- `find . -name Cargo.lock -not -path '*/target/*' | sort` -> four lockfiles.
- `cargo metadata --manifest-path <workspace>/Cargo.toml --locked --offline
  --format-version 1` for root, fuzz, examples/domain-link and
  benchmarks/neon-real -> rc=0 for all four after warming the registry cache.
- Drift reproduction with a bumped manifest version: `--locked` returns rc=101
  with "cannot update the lock file"; plain cargo returns rc=0 and rewrites the
  lock (SHA-256 recorded before and after in the audit).
- `cargo install --path crates/ag-cli --locked` from a clean checkout at the
  audited HEAD, with a private target dir and temporary install root.
- `git diff --check` and the content scans from `.github/workflows/docs.yml`.

## Criterios de salida que avanza

- Establishes the definitive state of every committed lockfile at commit
  `72fa9d8`: none stale.
- Provides a reproducible evidence trail for the missing `--locked` guard in
  `scripts/run-fuzz-window.sh`, `scripts/run-benchmarks.sh` and `install.sh`.
- Does not claim any fuzz or benchmark gate result; issues #156 and #157 stay
  open and unchanged.

## Cierre de issues

Refs #199

Closes: none

Issue #199 was opened by this audit for the reproducible silent-mutation defect.
It is not closed here: the defect is reported, not fixed, and wave 10 never
closes issues.

## Checklist

- [x] Cambio minimo y dentro de la fase correcta.
- [x] Documentacion actualizada en la misma rama.
- [x] Sin evidencia falsificada: comandos, hashes y tiempos tomados de corridas reales.
- [x] `git diff --check` ejecutado.
- [x] Escaneos de contenido ejecutados.
- [x] Sin push ni merge; sin prefijos de herramientas IA ni emojis.
