# ag-domains / ag-edge — backlog vs initial blueprint

This file tracks the gap between the original `ag-domains` implementation
blueprint and the current state of the code, so the remaining work is explicit
(CLAUDE.md rule 29: no hidden debt). Status legend:

- DONE — implemented and tested in this branch.
- PARTIAL — core exists; listed gap remains.
- TODO — not started (deliberately not stubbed).

Governing docs: `docs/rfc/RFC-0011-ag-domains-control-plane.md`,
`docs/adr/0012-ag-domains-control-plane.md`. Cross-refs: `docs/DEBT.md`
(DEBT-024, DEBT-025).

## 1. Blueprint acceptance checklist (§21)

| # | Item | Status | Note |
|---|------|--------|------|
| 1 | Existing routing unchanged without feature flags | DONE | All additions are additive / feature-gated. |
| 2 | Existing tests pass | DONE | Workspace builds; touched crates green. |
| 3 | New domain tables are additive migrations | DONE | `sql-store` feature: `SqlAttachmentStore` + `migrations/0001_*.sql` (additive `CREATE TABLE IF NOT EXISTS`). Native store remains default. |
| 4 | Hostname normalization | DONE | `hostname.rs` (idna). |
| 5 | TXT ownership verification | DONE | `ownership.rs` + propagation. |
| 6 | DNS instruction generation (apex + subdomain) | DONE | `instructions.rs` (+ wildcard, + BIND export). |
| 7 | DNS diagnostics (expected vs observed) | DONE | `diagnostics.rs`. |
| 8 | ACME staging issuance for exact hostname | PARTIAL | `acme::renewal::issue` exists (DNS-01, staging flag); attachment->issue->edge glue + `#[ignore]` staging test pending. |
| 9 | Edge selects certificate by SNI | DONE | `ag-edge::cert` + real TLS test. |
| 10 | Gateway routes by Host/:authority | DONE | `ag-edge::server` + real HTTP test. |
| 11 | Unknown domains cannot route to another tenant | DONE | Fail-closed; tested. |
| 12 | Detached domains enter tombstone/quarantine | DONE | `store.rs` tombstones. |
| 13 | CAA check before issuance | DONE | `caa.rs`. |
| 14 | Certificate renewal queue | PARTIAL | `spawn_renewal_task` + `issuance` dedup-by-SAN and per-domain rate limits DONE; ARI-aware scheduling pending. |
| 15 | CLI docs updated | DONE | `docs/ag-domains/reference/cli.md`. |
| 16 | OpenAPI docs updated | DONE | `openapi/ag-domains.v1.yaml` (implemented contract). |
| 17 | Provider docs for first providers | DONE | `how-to/connect-providers.md` (Namecheap/Hostinger/Squarespace/Cloudflare/Route53/Google/Azure), `reference/provider-capability-matrix.md` backed by `provider::capabilities` + `GET /v1/domains/provider-capabilities`. |
| + | Release/migration notes | DONE | CHANGELOG + DEBT + RFC. |

## 2. Phased roadmap (§20)

| Phase | Scope | Status | Remaining |
|-------|-------|--------|-----------|
| 0 | Audit + scaffolding | DONE | — |
| 1 | Manual attachment (CLI/API, TXT, instructions, diagnostics, shadow) | DONE | — |
| 2 | Managed TLS for exact hostnames | PARTIAL | issuance dedup-by-SAN + per-registered-domain rate limits + issuer seam DONE (`issuance` module; ACME path is the injected issuer); remaining: ARI-aware renewal scheduling and a live `#[ignore]` staging E2E. |
| 3 | Active routing + canonical policies | DONE | http->https upgrade at listener (only canonical redirects today). |
| D | SQL-backed store (`sql-store`, Postgres) | DONE | `SqlAttachmentStore` (JSONB + indexed columns) + migration; `#[ignore]` integration tests requiring `DATABASE_URL`. Native store stays default. |
| 4 | Provider automation | PARTIAL | Capability registry + matrix + provider how-to guides DONE; BIND export DONE; Cloudflare adapter DONE. Remaining: Domain Connect, Route 53 / Google / Azure / Namecheap adapters, richer adapter SDK (discover/read/diff/apply/rollback/verify). |
| 5 | Wildcards + DNS-01 | PARTIAL | classification/routing/cert selection DONE; end-to-end wildcard issuance orchestration + stricter wildcard policy pending. |
| 6 | ag-registrars module | TODO | Design proposal only (out of v1 by ADR-0007). |

## 3. Documentation gaps (Diataxis, §17)

DONE: tutorial (attach first domain); how-to (serve + API, connect-providers,
domain-connect, configure-wildcard, troubleshoot); reference (CLI, state machine,
DNS matrix, events-and-metrics, provider-capability-matrix); explanation (apex vs
subdomain, why TXT ownership).

TODO:
- Tutorials: dedicated subdomain and apex tutorials (currently combined).
- Reference: TLS lifecycle reference, security model, migration/compatibility notes.
- Explanation: why wildcards need stricter validation, how HTTP-01/DNS-01 work,
  how routing by Host/SNI works, why purchase and attachment are separate.

## 4. Cross-cutting items (§11, §15, §16)

| Item | Status | Note |
|------|--------|------|
| Provider capability registry + matrix | DONE | `provider::capabilities` (data) + `reference/provider-capability-matrix.md` + REST endpoint. |
| DNS diagnostics command (expected vs observed) | DONE | `ag domains diagnose` wires `diagnostics::diagnose` to live resolver lookups (`propagation::lookup_observed`). |
| Provider adapter SDK (discover/read/diff/apply/rollback/verify) | TODO | Only `DnsProvider` (list/upsert/delete) exists today. |
| On-demand TLS (restricted) | PARTIAL | `ag-edge::tls::allow_on_demand` logic + unit tests; not wired to a live issuance trigger. |
| Domain event log (`domain.*` events) | DONE | `events` module: `DomainEvent` + `EventSink` (`NullEventSink`/`InMemoryEventSink`/`TracingEventSink`); REST API emits `attachment.created` and `detached`. Native, no broker. |
| Control-plane metrics (`ag_domains_*`) | PARTIAL | Added `attachments_total`, `detached_total`, `verification_failures_total`; full §16.1 set (active gauge, tls/dns, cache hit-ratio, route latency) not all wired. |
| Dangling-DNS detection worker | TODO | Background scan for hosts pointing at edge but unattached. |
| Abuse controls (per-tenant limits, global ACME queue, rate limits) | TODO | — |
| eTLD+1 via Public Suffix List | TODO | DEBT-024; current two-label heuristic. |
| SQL-backed store (Postgres) behind `sql-store` | DONE | `SqlAttachmentStore` + migration; `#[ignore]` integration tests. |

## 5. Suggested next order

1. Phase E provider adapters (each behind a feature, native default preserved),
   starting with a Domain Connect discovery + the adapter SDK seam.
2. ARI-aware renewal scheduling + a live `#[ignore]` ACME staging E2E.
3. Remaining control-plane metrics (active gauge, tls/dns counters, edge cache
   hit-ratio, route-resolution latency) + dangling-DNS detection worker.
4. DEBT-024 PSL (needs a dependency RFC) — also fixes the per-domain counter key.
5. Phase F `ag-registrars` design RFC.
