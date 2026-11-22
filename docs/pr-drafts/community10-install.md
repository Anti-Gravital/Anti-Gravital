# Installer user journey - wave 10 real terminal QA

## Resumen

Ran the documented installer as a real user in an isolated clean HOME and a
local `file://` fake release directory. Recorded verbatim output, exit codes,
timings, installed files and `ag --version` for the source channel and for the
alpha channel with valid and invalid checksums. Added the evidence document
`docs/audits/install-user-journey.md` and a CHANGELOG entry. No product code
was changed. No push or merge is performed from this branch.

## Fase afectada

No architecture phase is advanced. Wave 10 terminal QA over the existing alpha
installer (Fase 4.8 / alpha release path).

## Tipo de cambio

Documentation and QA evidence only. No code, no CLI behavior, no architecture
contract.

## Documentos relacionados

- `docs/audits/install-user-journey.md` (new evidence)
- `docs/manual/04-instalacion-y-onboarding.md` (source read; stale version found)
- `docs/security/INSTALLATION_INTEGRITY.md` (source read)
- `README.md` (install section read)
- `CHANGELOG.md` (entry added)

## Plan de prueba

Already executed on 2026-11-22:

1. Copy the checkout into a clean temp working dir; run `bash install.sh` with a
   clean temp HOME and an isolated `CARGO_INSTALL_ROOT`; capture stdout/stderr
   with `time`, exit code, installed files and `ag --version`.
2. Build a fake `file://` release with a SHA-256 manifest; run the alpha channel
   with a valid checksum and verify the installed binary runs.
3. Repeat with an all-zero checksum, a missing manifest entry, a missing binary,
   a missing base URL, an unknown channel and a tampered out-of-band manifest.
4. Run the installer self-integrity checks with a correct manifest, a mismatched
   manifest and a missing manifest.
5. Exercise the Rust minimum-version gate with a fake old `rustc`.

## Criterios de salida que avanza

- Real evidence that both install channels behave as documented.
- Real evidence that checksum verification fails closed before installing.
- Concrete issues for reproducible defects found during the journey.
- Honest record of timings, environment and confusing behavior.

## Checklist final

- [x] Evidence doc with exact commands, outputs and exit codes
- [x] Issues only for reproducible real defects
- [x] `git diff --check`
- [x] Content scans (no AI-tool evidence, no emojis, no retired brands)
- [x] CHANGELOG updated
- [x] No shared code modified
- [x] No push or merge

## Cierre de issues

Refs #203
Refs #204
Refs #206
