# ag-domains hardening: issuance limits, provider capabilities, diagnose (RFC-0011)

## Summary

Completes several blueprint items for `ag-domains`, executed together with their
documentation (no doc references to unimplemented features). Additive; native
defaults preserved.

- Issuance rate-limit protections (blueprint 13.4): SAN-set dedup +
  per-registered-domain counters + injected ACME issuer seam.
- Provider capability registry + matrix + `GET /v1/domains/provider-capabilities`.
- `ag domains diagnose`: expected-vs-observed DNS comparison wired to live
  resolver lookups (closes the doc/command gap).
- Provider how-to guides, troubleshooting, capability matrix, OpenAPI update.

Changes:

- `crates/ag-domains/src/issuance.rs`: `CertIssuer` trait (seam for the ACME
  issuer), `IssuedCertificate`, `san_key` (canonical SAN-set dedup key),
  `IssuanceLimiter` (in-flight dedup + per-registered-domain issuance counters),
  and `issue_certificate` (reserve -> issue -> count successes -> release).
- `crates/ag-domains/src/lib.rs`: export `issuance`.
- `crates/ag-edge/tests/issuance_to_edge.rs`: E2E — issue via the orchestration
  (rcgen issuer), load the PEM into `ag_edge::cert::CertStore`, select by SNI,
  build a rustls server config.
- `crates/ag-edge/Cargo.toml`: `async-trait` dev-dependency for the test issuer.
- `crates/ag-domains/src/provider/capabilities.rs`: provider capability registry
  (`ProviderCapabilities`, `known_provider_capabilities`, `capabilities_for`).
- `crates/ag-domains/src/api.rs`: `GET /v1/domains/provider-capabilities`.
- `crates/ag-domains/src/propagation.rs`: `lookup_observed` (A/AAAA/CNAME/TXT).
- `crates/ag-cli/src/main.rs`: `ag domains diagnose` command.
- Docs: CHANGELOG; `reference/provider-capability-matrix.md`,
  `reference/cli.md` (diagnose); how-to `connect-providers`, `domain-connect`,
  `configure-wildcard`, `troubleshoot`; OpenAPI `/provider-capabilities`.
- Governance: CLAUDE.md rule 29 updated (tech debt -> GitHub Issues, not files);
  removed `docs/ag-domains/BACKLOG.md`, migrated to issue #76.
- `crates/ag-domains/src/api.rs`: `POST /v1/domains/attachments/{id}/verify`
  with an injectable `OwnershipVerifier` (emits `domain.ownership.verified`).

## Phase affected

Phase 5 (extends Phase 4.5 `ag-domains`). RFC-0011 phase 2 hardening.

## Type of change

- Feature implementation (additive; native logic; no new runtime deps)
- Test

## Related documents

- `docs/rfc/RFC-0011-ag-domains-control-plane.md`
- Issue #76 — ag-domains remaining work / tech debt
- `docs/DEBT.md` — DEBT-024 (eTLD+1 heuristic shared by the per-domain counter)

## Test plan

- [x] `cargo test -p ag-domains issuance` — 7 unit tests pass
- [x] `cargo test -p ag-edge --features tls` — incl. `issuance_to_edge` E2E pass
- [x] `cargo clippy -p ag-domains --all-features --all-targets -- -D warnings` — clean
- [x] `cargo clippy -p ag-edge --all-features --all-targets -- -D warnings` — clean
- [x] `cargo fmt` — clean

## Exit criteria advanced

- Blueprint section 13.4: SAN-set deduplication and per-registered-domain rate
  limits implemented and tested. Remaining: ARI-aware renewal scheduling and a
  live `#[ignore]` staging E2E (tracked in issue #76).

## Final checklist

- [x] Belongs to correct phase
- [x] Respects documentation (RFC-0011)
- [x] Does not break architecture (additive; ACME issuer injected)
- [x] No unnecessary complexity added
- [x] No circular dependencies
- [x] Compiles
- [x] Tests pass
- [x] `cargo fmt` passes
- [x] `cargo clippy` passes
- [x] Documentation updated in same PR
- [x] No emojis
- [x] No AI attribution
- [x] Commit messages under 256 characters
- [x] PR descriptor present
