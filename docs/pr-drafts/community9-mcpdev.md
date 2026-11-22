# Descriptor de PR: community9/mcpdev

## Resumen

Define additive versioned config.get, env.list and env.check JSON shapes and repair the 0.2.0 contract data

## Fase afectada

Additive Phase 4.8 machine-interface substrate (wave 9). No master roadmap document is changed; RFC-0021 section 4.3, RFC-0024 section 9 and RFC-0018 are the governing contracts.

## Tipo de cambio

- [x] Feature (new versioned read shape, defined as a contract fixture)
- [ ] Fix (internal behaviour)
- [ ] Refactor
- [x] Documentation (contract, schema, snapshots, README, CHANGELOG)
- [ ] CI
- [ ] Seguridad

## Documentos relacionados

- `docs/rfc/RFC-0024-mcp-shared-machine-interface.md` (sections 4.2, 9, Q4)
- `docs/rfc/RFC-0021-server-install-remote-console-and-machine-interface.md` (section 4.3)
- `docs/rfc/RFC-0018` / `crates/ag-cli/src/config.rs` (closed config key set)
- `docs/audits/mcp-readiness-review.md` (sections 12.2, 12.3)
- `docs/mcp/cli-json-contract-0.2.0.md`, `docs/mcp/cli-json-0.2.0.schema.json`,
  `docs/mcp/snapshots/{config.get,env.list,env.check}.json`, `docs/mcp/README.md`
- `CLAUDE.md` rules 4, 19, 22, 26, 28, 29; ADR-0008, ADR-0011

## Plan de prueba

```bash
cargo test -p ag-cli
cargo test --workspace --all-features
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo audit
cargo deny --all-features check
```

Real results:

- `cargo test -p ag-cli`: 144 unit + 25 json_output integration tests pass,
  together with all other ag-cli suites. New tests cover schema/version
  agreement, exact serialization, closed field sets and secret redaction for
  config.get, env.list and env.check.
- `cargo test --workspace --all-features`: passes.
- `cargo fmt --all -- --check`: passes.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passes.
- `cargo audit`: FAILS on pre-existing `url 2.5.8`, RUSTSEC-2026-0592
  (published 2026-11-20, solution upgrade to >=2.5.9). This change adds no
  dependency and does not touch Cargo.lock. NOT claimed as passing.
- `cargo deny --all-features check`: same pre-existing advisory failure;
  bans, licenses and sources all pass. NOT claimed as passing.

Key assertions: `config.get` has exactly four fields (`key`, `kind`, `value`,
`source`); `additionalProperties` is false; `key` and `kind` come from closed
sets so command input cannot become a field name; no token, password or PEM
body is part of the RFC-0018 key set; mutating commands still refuse JSON
before dispatch; prose output is byte-identical.

## Criterios de salida que avanza

- Typed, versioned shapes exist for the `config.get`, `env.list` and
  `env.check` read command groups.
- The 0.2.0 schema, contract, snapshot fixture and tests agree on one version.
- The duplicated trailing fragments in the 0.2.0 contract and MCP README are
  removed; dns.plan and dns.status are documented in the main body.
- No prose output, mutating behaviour or dependency set changed.

## Cierre de issues

Closes: none

## Checklist

- [x] Pertenece a la fase correcta (additive Phase 4.8, wave 9)
- [x] Respeta la documentacion (RFC-0021, RFC-0024, RFC-0018, ADR-0008/0011)
- [x] No rompe arquitectura (reuses the existing versioned envelope)
- [x] No crea dependencias circulares (no new crate or dependency)
- [x] Compila
- [x] Pasa tests
- [x] Pasa fmt
- [x] Pasa clippy
- [ ] Pasa audit (pre-existing RUSTSEC-2026-0592 in url 2.5.8; Cargo.lock is outside this role's ownership)
- [ ] Pasa deny (same pre-existing advisory; bans/licenses/sources pass)
- [x] Tiene documentacion (contract, schema, snapshots, MCP README, CHANGELOG)
- [x] Tiene trazabilidad (descriptor, notes, RFC references)

## Known boundary

`config.get`, `env.list` and `env.check` are defined fixtures, not emitted
output. `json_command_name()` in `cli.rs` and `run_json_command()` in `main.rs`
are owned by `clidev` in this wave; that wiring is required before these shapes
are live. The contract document states this explicitly and the refusal message
lists emitted commands only. Clidev's `env.list`/`env.check` structs must emit
the pinned field order byte-for-byte.
`project.list` and `project.describe` do not exist as CLI commands, so no shape
was invented for them (RFC-0024 Q4, audit section 12.3).
