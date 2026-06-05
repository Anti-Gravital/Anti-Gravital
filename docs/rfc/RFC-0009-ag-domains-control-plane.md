# RFC-0009: ag-domains — domain attachment and serving control plane

- Status: accepted
- Author: Angel Nereira (BDFL), Gravital Labs
- Draft date: 2026-06-04
- Target phase: 5 (extends `ag-domains` beyond its Phase 4.5 library form)
- Modules or crates affected: `ag-domains` (extended), `ag-edge` (new),
  `ag-cli` (new subcommands), `ag-cloud` (future consumer)
- Predecessor RFC: RFC-0007 (ag-domains scope, library form)
- Governing ADR: ADR-0007 (ag-mail/ag-domains), ADR-0010 (this expansion)
- Comment period: waived by BDFL decision. This RFC records an explicit
  scope expansion authorized by the project owner.

## 1. Motivation

RFC-0007 defined `ag-domains` as a declarative DNS + TLS library consumed by
`ag-cloud` during `ag deploy`. Domains are declared in `schema.ag`; the crate
applies records through a `DnsProvider` adapter and issues certificates via
ACME. It has no notion of domain lifecycle, ownership proof, hostname routing,
or attaching an externally purchased domain to a running project.

The project owner has decided to extend `ag-domains` into a native domain
attachment and serving control plane so that a developer can buy a domain from
any registrar, attach it to an Anti-Gravital project, copy exact DNS records
into the provider, have ownership and routing verified, and have Anti-Gravital
serve HTTPS for that hostname through its own native edge.

This RFC records that decision, fixes the boundaries, and stages the work so it
remains additive and non-regressive.

## 2. Problem

The declarative library cannot answer:

- Which external hostname maps to which project/environment/service.
- Has the operator proven control of the domain before it serves traffic.
- What exact DNS records must the operator paste into their provider.
- Which certificate is served for a given TLS SNI.
- How an incoming request hostname resolves to an upstream target.
- What happens to a hostname after it is detached (takeover prevention).

These require persistent attachment state, a state machine, ownership
verification, a DNS instruction engine, and a hostname routing layer.

## 3. Scope decision and boundaries

### 3.1 In scope

```text
- domain attachment lifecycle and persistent state
- ownership proof (TXT) before activation
- DNS instruction generation (apex/subdomain/wildcard) + BIND zone export
- DNS routing and ownership diagnostics (expected vs observed)
- CAA preflight before ACME issuance
- managed TLS lifecycle (reusing the existing ACME client)
- hostname-to-target binding and native edge routing logic
- detach/tombstone lifecycle to prevent takeover
- audit events and metrics
```

### 3.2 Explicitly out of scope (unchanged from RFC-0007)

```text
- domain purchase, transfer, renewal payments, WHOIS, EPP/ICANN (a future
  ag-registrars module, not this RFC)
- replacing Terraform/Pulumi/Kubernetes
- arbitrary multi-tenant DNS zone management as a hosting panel product
```

### 3.3 Native-first invariant (ADR-0009 rule 2 and 5)

Every external integration (Cloudflare, Route 53, Domain Connect, Postgres)
sits behind a Cargo feature with a native default. The default attachment
store is in-memory / local JSON file. The default edge logic is pure Rust.
Provider automation and a SQL-backed store are optional accelerators, never
requirements.

## 4. Architecture

```text
DNS provider / registrar   (external, owned by the user)
        |
        v
ag-domains control plane   (attach, verify, diagnose, secure, route metadata)
        |
        v
ag-edge data plane         (hostname resolution, SNI selection, redirect policy)
        |
        v
Anti-Gravital runtime      (project/service/environment resolution)
```

`ag-domains` owns control-plane state and decisions. `ag-edge` owns the
request-time resolution logic (a library of pure functions and types; it does
not, in this RFC, bind sockets — wiring a live listener is a later stage so the
edge logic can be unit-tested in isolation first).

## 5. Phased plan (additive, non-regressive)

```text
Phase A (this RFC, first installment):
  - hostname normalization + classification (idna)
  - attachment state machine types
  - attachment store trait + in-memory + JSON-file native impls
  - TXT ownership token generation + verification (via PropagationChecker)
  - DNS instruction engine + BIND zone export
  - CAA preflight
  - diagnostics (expected vs observed)
  - ag-edge crate: resolve_hostname + canonical/redirect policy + SNI selector
  - ag-cli: attach / status / detach / instructions / export-zone / diagnose
  - docs (Diataxis skeleton), OpenAPI skeleton, README/CHANGELOG sync

Phase B (later): live edge listeners (80/443), HTTP-01 responder wired to
  the running edge, SNI certificate serving end to end.

Phase C (later): REST API surface (/v1/domains/...) backed by the store,
  behind an `api` feature; OpenAPI fully populated.

Phase D (later): SQL-backed store (Postgres) behind a `sql-store` feature.

Phase E (later): provider automation (Domain Connect, additional adapters),
  DNS-01 wildcard automation.

Phase F (later): ag-registrars design proposal (separate module).
```

Each phase keeps the previous behavior intact. No phase removes a working path.

## 6. Backward compatibility

The existing `DnsProvider` trait, `AgDomainsError`, `record`, `acme`,
`propagation`, `mail_records`, and `metrics` modules are untouched in their
public contract. The existing `ag domains check` and `ag domains sync`
commands keep working. New modules are additive. If no attachment matches an
incoming hostname, the edge resolver returns an explicit fallback variant so
existing routing is preserved (fail-closed for unknown custom hostnames,
fail-open only for legacy default routes).

## 7. Risks

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| Scope drift into a hosting panel | Medium | High | Boundaries in 3.2; registrar split; reject arbitrary zone management. |
| eTLD+1 without a public-suffix list is wrong for multi-label TLDs | Medium | Medium | Best-effort now, tracked in DEBT.md; a PSL-backed impl is a later dep RFC. |
| New surface area increases maintenance | Medium | Medium | Phased, feature-gated, native-first; library-level and tested before wiring. |
| Edge fail-open leaks across tenants | Low | High | Unknown custom hostnames fail-closed; only legacy defaults fall open. |

## 8. Rollback

Phase A is additive library code plus new CLI subcommands. Removing the new
modules and subcommands restores the Phase 4.5 surface. `ag-edge` is an
independent crate; dropping it from the workspace does not affect `ag-domains`
library consumers. No migration is destructive (in-memory/JSON store).

## 9. Decision

- Decider: Angel Nereira (BDFL)
- Date: 2026-06-04
- Result: accepted (explicit scope expansion over RFC-0007)
- Justification: recorded per CLAUDE.md rules 22 and 28 (RFC required before
  new crates, new CLI commands, and module-boundary changes). ADR-0010 records
  the architectural decision and consequences.

## 10. References

- RFC-0007 — ag-domains library scope (predecessor).
- ADR-0007 — ag-mail / ag-domains decision.
- ADR-0010 — control-plane expansion decision.
- ADR-0009 — native-first / real-state governance.
- ADR-0008 — language policy (English canonical for technical docs).
- RFC 8555 (ACME), RFC 6066 (TLS SNI).
- OWASP Subdomain Takeover Prevention Cheat Sheet.
