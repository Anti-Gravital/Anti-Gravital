#PR descriptor

## Resumen

Add test-only release signing tooling and a human key ceremony procedure for RFC-0030; no production key is created and #141 stays open.

## Fase afectada

Release infrastructure (pre-1.0 alpha). Release workflow behavior and product scope are unchanged. Docs: `docs/security/`, `tools/`, `CHANGELOG.md`.

## Tipo de cambio

- [ ] Feature de producto
- [x] Infraestructura de release / tooling de soporte
- [x] Documentacion de seguridad
- [ ] Cambio de DSL o API publica
- [ ] Cambio de CI que afecta al estado de una gate

`.github/workflows/release.yml` is deliberately NOT modified: no signing claim.

## Documentos relacionados

- `docs/rfc/RFC-0030-signed-release-manifest-and-trust-anchor.md` (draft design)
- `docs/rfc/RFC-0032-alpha-release-process.md` (unsigned alpha decision)
- `docs/security/RELEASE_KEY_CEREMONY.md` (new)
- `docs/security/RELEASE_SIGNING_TEST_EVIDENCE.md` (new)
- `docs/security/INSTALLATION_INTEGRITY.md`, `SECURITY.md`, `ADR-0009`, `ADR-0011`

## Plan de prueba

Commands run from the worktree root on commit `9a95e795dc5e15eff9946d5c438df76905f13913`:

```bash
bash tools/release-signing/release-signing.sh selftest
AG_SIGN_STATE_DIR="$PWD/.ag-sign-should-not-exist" \
  bash tools/release-signing/release-signing.sh genkey   # must exit 1
export AG_SIGN_STATE_DIR=/tmp/opencode/ag-sign-test-state
bash tools/release-signing/release-signing.sh genkey
bash tools/release-signing/release-signing.sh sign /tmp/opencode/ag-sign-installers-test.sha256
bash tools/release-signing/release-signing.sh verify /tmp/opencode/ag-sign-installers-test.sha256
sed -i '1s/^./0/' /tmp/opencode/ag-sign-installers-test.sha256
bash tools/release-signing/release-signing.sh verify /tmp/opencode/ag-sign-installers-test.sha256  # must exit 1
git diff --check
```

Verbatim output, exit codes and host metadata are in
`docs/security/RELEASE_SIGNING_TEST_EVIDENCE.md`. The private key stays in a
`0700` temp directory with mode `0600` and is never printed or committed.

## Criterios de salida que avanza

- Advances the signed-release tooling design of RFC-0030 section 4.4 and the
  ceremony definition of sections 4.2, 4.3 and 4.6. It does NOT mark any
  release gate as passed.
- Does not satisfy RFC-0030 section 11: no production key, no two-channel
  publication, no signed asset, no clean-machine run. Issue #141 stays open.

## Cierre de issues

Refs #141
Closes: none

Rationale: #141 requires a real trust anchor, an offline-generated published
key, signed release assets and a clean-machine end-to-end verification. None
of those exists. This PR only adds rehearsal tooling and the human procedure.

## Checklist

- [x] Pertenece a la fase correcta (release infrastructure, no product code).
- [x] Respeta la documentacion (RFC-0030 design; no RFC change needed).
- [x] No anade dependencias ni codigo criptografico propio; usa OpenSSL CLI.
- [x] No modifica `.github/workflows/release.yml` ni anade secretos a CI.
- [x] Verificacion real registrada; #141 permanece abierto con evidencia.
