## Resumen

Add `ag deploy plan|apply|status` as a capability-scoped local artifact renderer

## Fase afectada

Wave 7 community deployment design (additive, before Phase 5). Does not open
Phase 5 `ag-cloud` or claim cluster deployment.

## Tipo de cambio

- [x] New CLI command group and executor module (`ag-cli`).
- [x] Additive versioned JSON contract 0.2.0 (two read shapes).
- [x] RFC, documentation, CHANGELOG and bilingual README update.
- [x] Tests (unit, integration, snapshots, no-network sentinel).
- [ ] CI changes: none.
- [x] Dependency change: `ag-cli` uses the existing `cap-std` workspace
      dependency for symlink-safe writes (RFC-0013, RFC-0033). Cargo.lock only
      records that direct dependency; no new transitive crate was added.

## Documentos relacionados

- `docs/rfc/RFC-0033-local-deploy-artifact-rendering.md` (new, scope and plan
  identifier contract).
- `docs/rfc/RFC-0021-server-install-remote-console-and-machine-interface.md`
  (plan/apply section 4.3).
- `docs/rfc/RFC-0024-mcp-shared-machine-interface.md` (section 9 prerequisites).
- `docs/mcp/cli-json-contract-0.2.0.md`, `cli-json-0.2.0.schema.json` and
  `docs/mcp/snapshots/deploy.{plan,status}.json`.
- `docs/modules/ag-cli/README.md`, RFC index, MCP readiness review update note,
  CHANGELOG and README (EN + ES).

## Plan de prueba

```sh
cargo fmt --all -- --check
cargo test -p ag-cli
cargo test --workspace --all-features
cargo clippy -p ag-cli --all-targets --all-features -- -D warnings
cargo audit
cargo deny check
```

Every command above was executed on the branch with the shared Cargo target
directory. The integration suite includes plan/apply parity for both targets,
stale-plan and missing-plan refusal, JSON snapshot parity, and a sentinel
`PATH` test that fails if `docker`, `kubectl`, `podman`, `helm` or `curl` is
invoked.

## Criterios de salida que avanza

- RFC-0021 section 4.3: a mutating command (`ag deploy apply`) requires an
  explicit plan identifier and refuses a stale one.
- Wave 7 deployment goal: authored Docker/Kubernetes artifacts become a
  reviewable, deterministic local artifact set without a cluster or runtime.
- Honest alpha documentation: README and module docs state that cluster
  deployment remains a Phase 5 capability.

## Cierre de issues

Closes: none
Refs #193

## Checklist

- [x] The change belongs to the wave 7 coordination scope and RFC-0033.
- [x] Documentation was written before and with the code.
- [x] No circular dependency, new crate, new dependency or CI change.
- [x] `cargo fmt`, `cargo test`, `cargo clippy -D warnings`, `cargo audit` and
      `cargo deny check` pass (audit reports pre-existing allowed warnings).
- [x] Public behaviour is documented in README (EN + ES) and CHANGELOG.
- [x] No AI-tool evidence, no emojis, commit subjects within 256 characters.
