# ag-domains control plane + ag-edge data plane (ADR-0010 / RFC-0009, phase A)

## Summary

Extends `ag-domains` from a declarative DNS+TLS library into a native domain
attachment and serving control plane, and adds the `ag-edge` data-plane crate,
per ADR-0010 / RFC-0009 (phase A). Additive and non-regressive: the existing
declarative library, CLI commands and routing are unchanged.

Changes:

- `crates/ag-domains/src/hostname.rs`: hostname normalization, IDN/Punycode
  identity, apex/subdomain/wildcard classification and validation.
- `crates/ag-domains/src/attachment.rs`: attachment state machine
  (ownership/dns/tls/routing dimensions + derived lifecycle + activation rule).
- `crates/ag-domains/src/store.rs`: `AttachmentStore` trait, native
  `InMemoryStore` and `JsonFileStore`, tombstones.
- `crates/ag-domains/src/ownership.rs`: TXT token generation + verification.
- `crates/ag-domains/src/instructions.rs`: DNS instruction engine + BIND export.
- `crates/ag-domains/src/caa.rs`: CAA preflight (pure decision + resolver query).
- `crates/ag-domains/src/diagnostics.rs`: expected vs observed comparison.
- `crates/ag-domains/Cargo.toml`, workspace `Cargo.toml`: `idna`, `uuid` deps.
- `crates/ag-edge/*`: new crate — `router::resolve_hostname`, `tls::SniCertStore`
  + `allow_on_demand`, `redirect::CanonicalPolicy`.
- `crates/ag-cli/src/main.rs`: `ag domains attach|instructions|export-zone|status|list|verify|detach`.
- Governance/docs: `RFC-0009`, `ADR-0010`, `docs/ag-domains/**`,
  `openapi/ag-domains.v1.yaml` (phase-C draft), README/CHANGELOG/DEBT updates.

## Phase affected

Phase 5 (extends Phase 4.5 `ag-domains`). RFC-0009 phase A.

## Type of change

- Feature implementation (additive; new crate `ag-edge`; native-first defaults)
- Documentation
- Test

## Related documents

- `docs/rfc/RFC-0009-ag-domains-control-plane.md`
- `docs/adr/0010-ag-domains-control-plane.md`
- `docs/DEBT.md` — DEBT-017 (eTLD+1/PSL), DEBT-018 (deferred phases B-F)

## Test plan

- [x] `cargo test -p ag-domains --all-features` — 80 tests + doctests pass
- [x] `cargo test -p ag-edge` — 17 tests pass
- [x] `cargo build -p ag-cli` — compiles; manual flow smoke-tested
      (attach apex+subdomain, list, status, export-zone, detach, tombstone block)
- [x] `cargo fmt --check -p ag-domains -p ag-edge -p ag-cli` — clean
- [x] `cargo clippy -p ag-domains --all-features --all-targets -- -D warnings` — clean
- [x] `cargo clippy -p ag-edge --all-targets -- -D warnings` — clean
- [x] `cargo clippy -p ag-cli --all-targets -- -D warnings` — clean
- [x] `cargo build --workspace` — no errors

## Exit criteria advanced

- RFC-0009 phase A delivered (control plane library, ag-edge, manual CLI flow).
- Deferred phases B-F tracked in DEBT-018.

## Final checklist

- [x] Belongs to correct phase
- [x] Respects documentation (RFC-0009 + ADR-0010 authorize the scope)
- [x] Does not break architecture (additive; existing API/CLI unchanged)
- [x] No unnecessary complexity added
- [x] No circular dependencies (ag-edge depends on ag-domains only)
- [x] Compiles
- [x] Tests pass
- [x] `cargo fmt` passes
- [x] `cargo clippy` passes
- [x] Documentation updated in same PR
- [x] No emojis
- [x] No AI attribution
- [x] Commit messages under 256 characters
- [x] PR descriptor present
