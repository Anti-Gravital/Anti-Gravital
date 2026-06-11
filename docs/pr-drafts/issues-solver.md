# Resolve ag-domains / ag-workers / ag-data issues (one commit per issue)

## Summary

Resolves the open, in-environment-reproducible issues, one commit per issue, in
priority then precedence order. Issues blocked on external infrastructure (live
PostgreSQL, real-domain ACME) or on an unapproved RFC are not closed here; for
the RFC-gated ones a design RFC is drafted instead so the maintainer can approve
and implement.

- **#79 (p2) provider adapter SDK seam:** `provider::sdk` adds a declarative
  plan/diff/apply/verify/rollback layer over `DnsProvider`; pure `diff` plus a
  `ProviderAdapter<P>` bridge so Cloudflare participates with no extra code.
- **#85 (p2) DNS-01 wildcard automation:** `acme::wildcard::issue_dns01_with_adapter`
  publishes `_acme-challenge` records through the SDK (scoped to the challenge
  subtree) and tears them down; mock-adapter tested.
- **#88 (p2) control-plane metrics (blueprint 16.1):** active/expiring-soon
  gauges, TLS-orders and DNS-misconfigured counters wired at the right
  operations; edge route-resolution latency + cache hit/miss in `ag-edge`.
- **#89 (p2) dangling-DNS detection:** `dangling` module + `domain.dangling_dns_detected`
  event (subdomain-takeover hygiene).
- **#90 (p2) abuse controls (blueprint 15.6):** per-tenant attachment/issuance
  limits + global ACME queue; REST API returns 429 at the limit (opt-in).
- **#110 (p3) canonical AgTx:** `ag-data` exposes `AgTx`; `enqueue_in_tx` takes it
  instead of a raw `sqlx::Transaction`; TECH-DEBT marker removed.
- **#113 (p3) reserved admission variant:** `RejectedRateLimited` documented as
  reserved RFC-0012 vocabulary, with a test asserting it is never produced.
- **#86 (p3) ARI renewal (RFC 9773):** `acme::ari` parsing + scheduling decision;
  `spawn_renewal_task_with_ari` prefers the CA window, falls back to notAfter.
- **#84 (p3) Domain Connect:** discovery + settings parsing, MX-safe template
  variables, sync apply-URL builder; independent verification reused.
- **#92 (p3) docs:** dedicated apex/subdomain tutorials, TLS-lifecycle /
  security-model / migration references, HTTP-01-vs-DNS-01 / routing-by-Host-SNI
  / purchase-vs-attachment explanations.
- **#93 / #78 / #114 (RFC-gated):** RFC-0015 (ag-registrars), RFC-0016 (PSL
  eTLD+1), RFC-0017 (bulk DLQ) drafted as proposed; no code until approved.

## Phase affected

Phase 4.5 (ag-domains/ag-edge) and additive Phase 4.6 (ag-workers/ag-data).
No phase transition; all work is additive and feature-gated where applicable.

## Type of change

- [ ] Security fix (security-relevant: #89, #90)
- [ ] Bug fix
- [x] Tests
- [x] Documentation
- [x] New feature
- [ ] Breaking public API change

## Related documents

- `docs/ag-domains/reference/provider-adapter-sdk.md`, `.../abuse-controls.md`,
  `.../tls-lifecycle.md`, `.../security-model.md`, `.../migration-compatibility.md`
- `docs/ag-domains/reference/events-and-metrics.md` (updated)
- `docs/ag-domains/tutorials/attach-apex.md`, `.../attach-subdomain.md`
- `docs/ag-domains/explanation/http01-vs-dns01.md`, `.../routing-host-sni.md`,
  `.../purchase-vs-attachment.md`
- `docs/adr/0013-ag-workers-execution-model.md` (AgTx resolution note)
- `docs/rfc/RFC-0015-ag-registrars-design.md`, `RFC-0016-eldp1-public-suffix-list.md`,
  `RFC-0017-ag-workers-bulk-dlq.md`

## Test plan

- [x] `cargo fmt --all --check` — no diffs
- [x] `cargo clippy --workspace --all-targets` — clean
- [x] `cargo test --workspace --all-features` — 0 failures
- [x] `cargo test -p ag-domains --all-features` — SDK/wildcard/metrics/dangling/abuse/ARI/DomainConnect suites pass
- [x] `cargo test -p ag-edge --all-features` — router metrics + TLS edge pass
- [x] `cargo test -p ag-workers --features postgres --no-run` — AgTx enqueue_in_tx compiles
- [x] `cargo build -p workers-postgres` — example builds against AgTx

## Exit criteria advanced

- #76 ag-domains remaining work: #79, #85, #88, #89, #90, #84, #92 resolved;
  #78 unblocked via RFC-0016; #93 RFC drafted.
- ag-workers/ag-data: #110 resolved; #113 reserved; #114 unblocked via RFC-0017.
- Still blocked on external infrastructure (untouched, documented): #108, #109,
  #103 (live PostgreSQL), #87 (real-domain ACME staging).

## Final checklist

- [x] Belongs to correct phase
- [x] Respects documentation
- [x] Does not break architecture
- [x] No unnecessary complexity added
- [x] No circular dependencies
- [x] Compiles
- [x] Tests pass (`cargo test --workspace --all-features`, exit 0)
- [x] `cargo fmt` passes
- [x] `cargo clippy` passes
- [x] Documentation updated in same PR
- [x] No emojis
- [x] No AI attribution
- [x] Commit messages under 256 characters
- [x] PR descriptor present
