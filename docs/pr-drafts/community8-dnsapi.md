# Descriptor de PR: community8/dnsapi

## Resumen

Add offline `ag dns plan|apply|status` and manifest-aware `ag domains attach` from the RFC-0033 `[domain]` contract

## Fase afectada

Additive Phase 4.8 domain-to-deploy loop (wave 8). No master roadmap document is changed; RFC-0033 is the governing derived contract.

## Tipo de cambio

- [x] Feature (new CLI command group and flags)
- [ ] Fix
- [ ] Refactor
- [x] Documentation
- [ ] CI
- [ ] Seguridad

## Documentos relacionados

- `docs/rfc/RFC-0033-domain-to-deploy-loop.md` (sections 4.1, 4.3, 4.5, 4.7, 4.8)
- `docs/rfc/RFC-0021-server-install-remote-console-and-machine-interface.md` (section 4.3 plan/apply and JSON)
- `docs/mcp/cli-json-contract-0.2.0.md`, `docs/mcp/cli-json-0.2.0.schema.json`
- `CLAUDE.md` rules 12, 16, 19, 22, 26, 28
- `ADR-0008` (English canonical), `ADR-0009` (native-first), `ADR-0011` (no third-party brand names)

## Plan de prueba

```bash
cargo test -p ag-cli
cargo test --workspace --all-features
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo audit
cargo deny --all-features check
```

Key assertions: deterministic plan id across runs and machines; `apply` refuses a missing or stale plan id before writing; no network or credential access by default; incomplete adapter configuration fails before any state write; no token, zone id or brand name in generated files; wildcard, TTL, visibility, TLS and environment alias validation; `--output json` for `dns.plan`/`dns.status` matches byte-exact snapshots and mutating commands refuse it.

## Criterios de salida que avanza

- Domain configuration can be planned and applied locally without a live provider or cluster.
- Private bindings fail closed and never publish public routing records.
- Machine consumers get a versioned read-only JSON shape for the domain loop.
- No credential or provider zone id is persisted in the repository.

## Cierre de issues

Closes: none

## Checklist

- [x] Pertenece a la fase correcta (additive Phase 4.8, wave 8)
- [x] Respeta la documentacion (RFC-0033, RFC-0021, ADR-0008/0009/0011)
- [x] No rompe arquitectura (reuses ag-domains and the existing JSON envelope)
- [x] No crea dependencias circulares (no new crate or dependency)
- [x] Compila
- [x] Pasa tests
- [x] Pasa fmt
- [x] Pasa clippy
- [x] Tiene documentacion (CHANGELOG, README EN+ES, crate and module docs, docs/mcp)
- [x] Tiene trazabilidad (descriptor, notes, RFC references)
