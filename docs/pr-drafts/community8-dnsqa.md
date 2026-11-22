# Resumen

Offline harness and real evidence for the wave-8 domain-to-deploy loop

# Fase afectada

Fase 4.8, circuito dominio-despliegue. Additive validation tooling and audit;
no roadmap box is advanced and no master document is modified.

# Tipo de cambio

- [ ] Codigo
- [ ] CLI
- [x] Documentacion (audit, CHANGELOG)
- [ ] Infraestructura
- [x] Tests (validation harness with self-test)

# Documentos relacionados

- `scripts/validate-domain-loop.sh` (new harness)
- `docs/audits/domain-loop-validation.md` (new real evidence)
- `CHANGELOG.md`
- Contract under test: `docs/rfc/RFC-0033-domain-to-deploy-loop.md` (dnsarch),
  implemented by `community8/dnsapi`. No file of another role was edited.

# Plan de prueba

- `bash -n scripts/validate-domain-loop.sh` (syntax)
- `git diff --check` (whitespace)
- `./scripts/validate-domain-loop.sh --self-test` (harness reachability: 12 PASS)
- Baseline run against the pre-dns binary: 4 PASS, 7 FAIL, exit 1 (missing
  commands reported as FAIL, never SKIP)
- Build from the dnsapi worktree:
  `CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 cargo build --locked --offline -p ag-cli --bin ag`
- Authoritative run: 12 PASS, 0 FAIL, 0 SKIP, exit 0. Verbatim output, host,
  tool versions, commit, date and exact commands are in the audit.
- Content scans: no emoji, no AI-tool attribution, no third-party brand used as
  a component name, no credential value in output.

# Criterios de salida que avanza

- The domain loop is proven testable with no live provider: plan, apply gates,
  status, versioned JSON and visibility rules all verified offline in a
  no-network sandbox with a hosts-file mock.
- plan/apply gates are shown to be non-bypassable: apply refuses without a plan
  id and with a bogus id without writing state; mutations refuse `--output json`.
- Honest evidence is recorded: baseline FAILs are preserved, no SKIP is used
  for a missing command, and the self-test is labelled as harness-only.

## Cierre de issues

Closes: none

# Checklist

- [x] Pertenece a la fase correcta (wave 8 validation, additive)
- [x] Respeta la documentacion y no introduce features fuera de alcance
- [x] No rompe arquitectura ni anade dependencias o crates
- [x] No anade complejidad innecesaria ni credenciales
- [x] No crea dependencias circulares
- [x] No edita archivos de otros roles (solo scripts/, docs/audits/, CHANGELOG y este descriptor)
- [x] Pasa bash -n y git diff --check
- [x] Tiene documentacion (audit con evidencia real)
- [x] Sin emojis, sin evidencia de herramientas IA, titulo <= 256 caracteres
