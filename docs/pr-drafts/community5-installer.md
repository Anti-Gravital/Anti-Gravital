# Descriptor de PR: community5/installer

## Resumen

Add opt-in alpha installer channel with checksum-verified prebuilt binary download

## Fase afectada

Alpha release infrastructure (wave 5). No `crates/` code, DSL, public API or
master document changes. Fase 0 release tooling only; no roadmap phase moved.

## Tipo de cambio

- [x] Feature (herramienta de instalacion, no producto)
- [x] Documentacion tecnica (`docs/security/`, `CHANGELOG.md`)
- [ ] Cambio de DSL / API / crates
- [ ] Cambio de arquitectura o roadmap

## Documentos relacionados

- RFC-0030 (signed release manifest and trust anchor): design only, still draft.
- Issue #141: advanced, not closed.
- ADR-0009 rule 4 (auditable installers, no privileged default, native fallback).
- `docs/security/INSTALLATION_INTEGRITY.md`; `SECURITY.md`.

## Plan de prueba

Commands executed on branch `community5/installer`:

```bash
bash -n install.sh
bash -n tools/check-installers.sh
bash -n tools/test-alpha-installer.sh
bash tools/check-installers.sh
sha256sum --check checksums/installers.sha256
bash tests/hooks/test-hooks.sh
git diff --check
```

`tools/test-alpha-installer.sh` builds a local `file://` fake release and checks:
valid manifest installs; mismatched checksum, missing manifest, missing binary,
missing source and unknown channel all fail closed with no file installed. It
also checks the optional out-of-band binary manifest path (pass and missing
file). A local HTTP server run exercised the real `curl` download path end to
end. The release workflow's `checksums.txt` uses the same bare `ag-<target>`
names (RFC-0032 section 4.3).

## Criterios de salida que avanza

- Alpha installation path is usable for evaluation without a signing key.
- Checksum verification remains fail closed; no network execution before verify.
- No privileged default install; scripts stay auditable and LF-only.
- Integrity guide and CHANGELOG describe the channel and its limits honestly.

## Cierre de issues

Refs #141
Closes: none

Issue #141 stays open: no signing key, no detached signature and no clean-machine
signed-manifest flow exist yet, so RFC-0030 conditions 1-6 are not met.

## Checklist

- [x] Resumen <= 256 caracteres, sin emojis ni evidencia de herramientas IA.
- [x] Cambio minimo y auditable; sin claves inventadas ni claims de firma.
- [x] Documentacion tecnica y CHANGELOG actualizados en la misma rama.
- [x] Verificacion local ejecutada y registrada en notas.
- [x] No se hizo push ni merge.
