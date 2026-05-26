# Pre-Phase-5 Corrective Audit — P1 through P6

## Summary

- **P1 — Documentation reconciliation:** Removed all stale "skeleton/empty" status markers
  from ag-mail, ag-domains, ag-data, ag-dsl, ag-cli, ag-cache. Created `docs/DEBT.md` as
  the single source of truth for 11 tracked tech debts. Added ADR-0009 (real-status and
  self-sufficiency rules) to CLAUDE.md and formalized in `docs/adr/0009-gobernanza-correctiva.md`.
- **P2 — ag-mail: PersistentQueue + SMTP custom headers:** Full PostgreSQL-backed queue via
  `FOR UPDATE SKIP LOCKED`. Custom SMTP headers via lettre `raw_header` API. Closes DEBT-001
  and DEBT-002.
- **P3 — ag-domains: notAfter parsing + date-based renewal:** `parse_not_after()` via
  x509-parser. `seconds_until_renewal()` pure testable helper. `spawn_renewal_task` sleeps
  until the renewal window instead of a fixed interval. `ag_domains_cert_days_until_expiry`
  gauge exported. Closes DEBT-005.
- **P4 — ag-realtime: 50k subscriber proof + EventBuffer:** Load test verifies 100% delivery
  to 50,000 subscribers in < 35 ms. `EventBuffer` append-only NDJSON persistence with
  `replay_into_bus`. Closes DEBT-007 and DEBT-008.
- **P5 — ag-cache RESP2 L2:** BLOCKED — awaiting RFC-0005 approval. No changes.
- **Security — hickory-resolver CVE:** Upgraded hickory-resolver 0.24 → 0.26.1 in
  ag-domains to patch the hickory-proto CPU-exhaustion vulnerability (O(n²) name
  compression). Migrated propagation checker to new 0.26 API: `TokioResolver` builder
  pattern, `NameServerConfig::udp()`, `RData` field access for TXT record iteration.
- **P6 — Tooling and onboarding:** `install.sh` + `install.ps1` auditable installers.
  cargo-tarpaulin coverage gate (>=80%) in `quality.yml`. E2E cross-module test
  ag-domains + ag-mail. Interactive `ag new` template prompt (non-interactive safe via
  dialoguer). CLI subcommand reference and bilingual onboarding chapter
  `docs/manual/04-instalacion-y-onboarding.md`. Closes DEBT-010 and DEBT-011.

## Phase affected

Pre-Phase-5 corrective (all phases 0-4.5).

## Type of change

- Documentation reconciliation (P1)
- Feature implementation (P2, P3, P4, P6)
- CI/tooling (P6)
- Test (P4, P6)

## Related documents

- `docs/DEBT.md` — technical debt register (source of truth for all closed entries)
- `docs/adr/0009-gobernanza-correctiva.md` — ADR formalizing real-status rules
- `docs/benchmarks/ag-realtime-load.md` — 50k subscriber load test methodology
- `docs/manual/04-instalacion-y-onboarding.md` — onboarding chapter (bilingual)
- `docs/superpowers/plans/2026-05-26-corrective-before-fase5-MASTER.md` — master plan

## Test plan

- [ ] `cargo build --workspace` — compiles clean
- [ ] `cargo test -p ag-mail --features queue-persistent` — 0 failures
- [ ] `cargo test -p ag-domains --features acme` — 32 passed
- [ ] `cargo test -p ag-domains --features propagation` — 32 passed
- [ ] `cargo test -p ag-realtime --features event-persistence` — 27 passed
- [ ] `cargo test -p ag-realtime --test load_50k -- --ignored --nocapture` — PASS
- [ ] `cargo test -p ag-integration-tests auth_sends_verification` — PASS
- [ ] `cargo test -p ag-cli` — 6 passed
- [ ] `cargo fmt --all -- --check` — no diffs
- [ ] `cargo clippy -p ag-mail --features queue-persistent -- -D warnings` — clean
- [ ] `cargo clippy -p ag-domains --features acme -- -D warnings` — clean
- [ ] `cargo clippy -p ag-realtime --features event-persistence -- -D warnings` — clean
- [ ] `cargo clippy -p ag-cli -- -D warnings` — clean
- [ ] `bash -n install.sh` — syntax OK
- [ ] `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/quality.yml'))"` — OK

## Exit criteria advanced

- DEBT-001, DEBT-002: ag-mail production-grade queue and SMTP headers
- DEBT-005: date-based ACME certificate renewal
- DEBT-007, DEBT-008: ag-realtime 50k proof + event persistence
- DEBT-010, DEBT-011: CI coverage gate + unified installer
- ADR-0009: real-status governance rule in effect
- Security: hickory-proto CPU-exhaustion CVE closed via hickory-resolver 0.26.1 upgrade

## Final checklist

- [x] Belongs to correct phase
- [x] Respects documentation
- [x] Does not break architecture
- [x] No unnecessary complexity added
- [x] No circular dependencies
- [x] Compiles
- [x] Tests pass
- [x] `cargo fmt` passes
- [x] `cargo clippy` passes (per-crate, relevant features)
- [x] Documentation updated in same PR
- [x] No emojis
- [x] No AI attribution
- [x] Commit messages under 256 characters
- [x] PR descriptor present
