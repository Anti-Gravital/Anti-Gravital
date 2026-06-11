# Resolve open issues: RFC approvals, docs honesty, PSL, bulk DLQ, DNS adapters

## Summary

Resolves the open, in-environment-reproducible issues, one commit per issue, in
priority then precedence order. The maintainer (BDFL) approved RFC-0015, RFC-0016
and RFC-0017 with the comment period waived (as for RFC-0011/0012), unblocking
the RFC-gated work. Issues blocked on external infrastructure (live PostgreSQL,
real-domain ACME) are left untouched, and the design-deferred edge wiring is left
as documented.

- **#93 ag-registrars design RFC:** RFC-0015 accepted (design only, Phase F; no
  code until that phase). Provider- and registrar-agnostic core preserved.
- **#71 docs honesty (Stage 10 gate):** phase status across the master roadmap,
  `STATUS.md` and the README is reconciled to explicit evidence-based states;
  Stage 10 reconciliation report added; the gate's "Docs honesty" row flips to
  pass while fuzz-24h, benchmarks and open-debt stay pending (gate remains OPEN).
- **#117 split-masters reconciliation:** derived `docs/architecture/*` and
  `docs/roadmap/*` regenerated in English from the bilingual masters, byte-for-byte
  minus breadcrumbs; no Spanish-only content lost.
- **#78 (p2) eTLD+1 via PSL:** RFC-0016 accepted; a single shared
  `registrable_domain` becomes the only eTLD+1 source, PSL-correct behind the
  optional `psl` feature, two-label heuristic as the offline default; hostname and
  issuance counting share it. DEBT-024 resolved.
- **#114 (p3) bulk DLQ:** RFC-0017 accepted; `ag workers dlq retry|purge` gain
  `--queue/--kind/--limit/--dry-run` filtered bulk operations over the existing
  `workers-runtime` feature, single-ID behaviour unchanged.
- **#80/#81/#82/#83 (p3) DNS adapters:** Route 53, Google Cloud DNS, Azure DNS and
  Namecheap `DnsProvider` adapters, each behind its own Cargo feature, mock/contract
  tested, real-credential tests `#[ignore]`; capability matrix flipped to read/apply.

## Phase affected

Phase 4.5 (ag-domains) and additive Phase 4.6 (ag-workers); plus documentation
honesty (pre-Phase-5 gate Stage 10). No phase transition; work is additive and
feature-gated where applicable.

## Type of change

- [ ] Security fix
- [ ] Bug fix
- [x] Tests
- [x] Documentation
- [x] New feature
- [ ] Breaking public API change

## Related documents

- `docs/rfc/RFC-0015-ag-registrars-design.md`, `RFC-0016-eldp1-public-suffix-list.md`,
  `RFC-0017-ag-workers-bulk-dlq.md` (accepted)
- `docs/audits/PRE_FASE5_RELEASE_GATE.md`, `docs/audits/pre-fase5-docs-reconciliation.md`
- `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`, `docs/roadmap/STATUS.md`, `README.md`
- `docs/architecture/*`, `docs/roadmap/*` (regenerated), `tools/split-masters.sh`
- `docs/modules/ag-domains*`, `docs/modules/ag-workers*`,
  `docs/ag-domains/reference/provider-capability-matrix.md`

## Test plan

- [ ] `cargo fmt --all --check` — no diffs
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean
- [ ] `cargo test --workspace --all-features` — 0 failures
- [ ] `cargo build -p ag-domains --features route53,google-cloud-dns,azure-dns,namecheap`
- [ ] `bash tools/split-masters.sh` + `git diff --stat docs/architecture docs/roadmap`
- Real-credential / live-database paths are `#[ignore]` (ADR-0009 convention);
  their verification is delegated to a credentialed environment.

## Exit criteria advanced

- #76 ag-domains remaining work: #78, #80, #81, #82, #83 resolved; #93 RFC accepted.
- ag-workers: #114 resolved.
- Pre-Phase-5 gate: Stage 10 (docs honesty, #71) closed; gate remains OPEN
  (fuzz-24h, benchmarks, open-debt still pending).
- Still blocked on external infrastructure (untouched, documented): #108, #109,
  #103 (live PostgreSQL), #87 (real-domain ACME staging).
- Design-deferred (untouched): #112 (ag-edge producer wiring, no consumer yet).

## Final checklist

- [x] Belongs to correct phase
- [x] Respects documentation
- [x] Does not break architecture
- [x] No unnecessary complexity added
- [x] No circular dependencies
- [ ] Compiles
- [ ] Tests pass (`cargo test --workspace --all-features`, exit 0)
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes
- [x] Documentation updated in same PR
- [x] No emojis
- [x] No AI attribution
- [x] Commit messages under 256 characters
- [x] PR descriptor present
