## ADR-0012: ag-domains as a native domain attachment and serving control plane

**Status:** Accepted
**Date:** 2026-06-04
**Author:** Angel Nereira (BDFL)
**Affected crates:** `ag-domains` (extended), `ag-edge` (new), `ag-cli` (new subcommands)
**Master documents touched:** none required for Phase A; `ANTI-GRAVITAL-Arquitectura-Tecnica.md` is updated as later phases land.

---

## Context

ADR-0007 introduced `ag-domains` as an optional infrastructure library:
declarative DNS + TLS for domains declared in `schema.ag`, consumed by
`ag-cloud` during `ag deploy`. RFC-0007 fixed its technical form (the
`DnsProvider` trait, an ACME client, propagation checks, SPF/DKIM/DMARC).

That form cannot attach an externally purchased domain to a running project,
prove ownership before serving traffic, generate copy-paste DNS instructions,
or route public requests by hostname. The project owner decided to expand
`ag-domains` into a native domain attachment and serving control plane so a
developer can buy a domain anywhere, attach it, paste DNS records, and have
Anti-Gravital serve HTTPS for that hostname through its own edge.

This is a deliberate, owner-authorized expansion of the scope fixed by
ADR-0007 and RFC-0007. It is recorded here so the codebase and the
documentation stay consistent (CLAUDE.md sections 0 and 26) and so future
contributors understand why `ag-domains` grew a control plane and why a new
`ag-edge` crate exists.

## Decision

1. `ag-domains` gains a control-plane layer: hostname normalization and
   classification, an attachment state machine, a persistent attachment store
   (trait + native in-memory and JSON-file implementations), TXT ownership
   verification, a DNS instruction engine with BIND zone export, CAA preflight,
   and diagnostics. The existing library modules keep their public contract.

2. A new crate `ag-edge` holds the request-time resolution logic:
   `resolve_hostname` (exact, then wildcard, then legacy fallback), canonical
   host and redirect policy, and an SNI certificate selection interface. In this
   decision `ag-edge` is a library of pure functions and types; binding live
   listeners is a later phase so the logic is unit-testable first.

3. The boundary from ADR-0007 holds: this is not a registrar, not a Terraform
   replacement, and not an arbitrary multi-tenant DNS hosting panel. Domain
   purchase/transfer/renewal remain out of scope for a future `ag-registrars`.

4. Native-first per ADR-0009: every external integration (provider APIs,
   Domain Connect, Postgres) is feature-gated with a working native default.
   The default store is in-memory / local JSON; the default edge logic is pure
   Rust. No external service is required to attach a domain manually.

5. The work is staged (RFC-0011 phases A-F) so each step is additive and
   non-regressive. Unknown custom hostnames fail closed at the edge; only
   pre-existing legacy default routes fall open.

## Consequences

Positive:

- A developer can attach an external domain, get exact DNS instructions,
  prove ownership, and (in later phases) serve native HTTPS — without a cloud
  vendor lock-in.
- The control plane and edge are decoupled and independently testable.
- The scope expansion is documented, bounded, and reversible.

Negative / costs:

- Larger surface area in `ag-domains` and a new crate to maintain.
- eTLD+1 detection without a public-suffix list is best-effort and tracked as
  tech debt (DEBT.md) until a PSL-backed implementation is justified by RFC.
- Later phases (live edge, REST API, SQL store) add operational concerns that
  must each preserve the native default.

## Alternatives

- **Keep ag-domains as a library only (status quo, RFC-0007).** Rejected by the
  owner: it does not deliver custom-domain attachment and native serving.
- **Build the control plane inside ag-cloud.** Rejected: conflates "simple
  deploy" with "domain control plane"; ADR-0007 already separated these.
- **Adopt a cloud vendor (Vercel/Cloudflare for SaaS) for custom domains.**
  Rejected: violates the native-first invariant (ADR-0009, CLAUDE.md section 12).

## Supersession note

This ADR expands, but does not contradict, the out-of-scope list of ADR-0007
(no registrar, no Terraform replacement). It supersedes ADR-0007 only on the
point that `ag-domains` is now more than a declarative library: it is also a
domain attachment and serving control plane.
