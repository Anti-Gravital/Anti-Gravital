# ag-domains certificate issuance: dedup + per-domain rate limits (RFC-0011 phase 2)

## Summary

Adds the certificate-issuance rate-limit protections from the blueprint section
13.4 as native, tested logic in `ag-domains`, plus an end-to-end test proving
the issuance output serves through the `ag-edge` certificate store by SNI.
Additive: no existing behavior changes; the real ACME order is the injected
issuer.

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
- Docs: CHANGELOG, `docs/ag-domains/BACKLOG.md` (phase 2 / checklist 14).

## Phase affected

Phase 5 (extends Phase 4.5 `ag-domains`). RFC-0011 phase 2 hardening.

## Type of change

- Feature implementation (additive; native logic; no new runtime deps)
- Test

## Related documents

- `docs/rfc/RFC-0011-ag-domains-control-plane.md`
- `docs/ag-domains/BACKLOG.md` (phase 2; blueprint section 13.4)
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
  live `#[ignore]` staging E2E (tracked in BACKLOG / DEBT-025).

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
