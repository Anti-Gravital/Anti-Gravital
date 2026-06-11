# RFC-0015 - ag-registrars: domain purchase, transfer and renewal (phase F)

- Status: accepted
- Author: Angel Nereira (BDFL), Gravital Labs
- Draft date: 2026-06-11
- Decision date: 2026-06-11
- Target phase: Phase F (post Phase 4.5/4.6; after the pre-Phase-5 gate)
- Affected modules/crates: a new `ag-registrars` crate; `ag-domains` and
  `ag-edge` are explicitly NOT modified by this RFC
- Predecessor RFC: RFC-0011 (ag-domains control plane), RFC-0007 (ag-domains scope)
- Comment period: waived by BDFL decision. This RFC records an approved design
  for a future, optional Phase-F crate; no code is written until Phase F starts.

## 1. Motivation

`ag-domains` attaches, verifies, secures and routes domains the operator already
owns; it is explicitly **not** a registrar (RFC-0011 §3.2, RFC-0007). Operators
still have to buy, transfer and renew names somewhere. A separate, optional
module that handles that *commerce* — without contaminating the attachment
lifecycle — closes the loop for users who want a single tool, while keeping the
core provider-agnostic and registrar-agnostic.

This RFC only proposes the design. Per issue #93, **no code is written until this
RFC is approved.**

## 2. Problem

- Buying/transferring/renewing a domain is a commercial transaction with a
  registrar (pricing, billing, EPP/auth codes, ICANN obligations, WHOIS/RDAP,
  transfer locks, grace periods). This is a fundamentally different concern from
  attaching a domain to a project.
- Folding purchase into `ag-domains` would: (a) make the attachment lifecycle
  depend on a billing relationship, (b) pull heavy registrar SDKs/credentials
  into a core module, and (c) risk turning Anti-Gravital into a reseller/registrar
  it is not positioned to be.
- Yet leaving purchase completely outside means users juggle two tools and copy
  names by hand.

## 3. Alternatives considered

1. **Do nothing (status quo).** Operators buy domains externally; `ag-domains`
   only attaches. Pro: zero scope creep, zero new surface. Con: no purchase/renew
   automation; the loop stays manual. This remains the default until this RFC is
   approved and is the fallback if it is rejected.
2. **Add purchase to `ag-domains`.** Rejected: violates RFC-0011 §3.2 and RFC-0007
   (ag-domains is not a registrar), couples attachment to billing, and bloats a
   core module. Explicitly out of bounds.
3. **Separate optional `ag-registrars` module (this RFC).** A new crate behind its
   own feature gates, depending on `ag-domains` (not the reverse), reusing the
   `DnsProvider`/adapter SDK only where a registrar also hosts DNS. Pro: keeps the
   core clean and provider-agnostic; opt-in; reversible. Con: a new module to
   maintain and a new trust boundary (credentials, money). Chosen.

## 4. Proposed design

### 4.1 Scope boundary (hard constraints)

- `ag-domains` core has **no dependency** on `ag-registrars`. The allowed
  direction is `ag-registrars -> ag-domains` only (a registrar flow may, after
  purchase, hand a name to the attachment lifecycle). This is symmetric with the
  `ag-edge -> ag-domains` rule (ADR-0012) and prevents a dependency cycle.
- The attachment lifecycle stays **provider-agnostic and registrar-agnostic**: a
  domain attached through `ag-domains` behaves identically whether it was bought
  via `ag-registrars` or anywhere else.
- `ag-registrars` is **optional** and native-first (ADR-0009): the crate compiles
  and the default build performs no network/commerce; every registrar is an
  adapter behind a Cargo feature.

### 4.2 Crate placement (CLAUDE.md rule 14)

`ag-registrars` is classified **Optional infrastructure** (alongside
`ag-domains`/`ag-edge`), not Core/Standard. It is never installed by default in
official templates.

### 4.3 Core abstraction (sketch, not final API)

A small trait isolates the registrar commerce surface, mirroring how
`DnsProvider` isolates DNS:

```text
trait Registrar {
    fn name(&self) -> &'static str;
    async fn check_availability(&self, domain) -> Availability;     // name + price
    async fn purchase(&self, order: PurchaseOrder) -> Registration; // buy
    async fn renew(&self, domain, period) -> Registration;          // renew
    async fn transfer_in(&self, domain, auth_code) -> TransferState; // EPP transfer
    async fn list_registrations(&self) -> Vec<Registration>;
}
```

- Pricing, currency and billing identifiers are returned by the adapter; the
  module never invents prices (CLAUDE.md rule 17 spirit: no fabricated numbers).
- `Registration` carries expiry, auto-renew state, transfer-lock and EPP/auth
  status — enough for renewal scheduling and transfers, nothing more.
- After a successful `purchase`/`transfer_in`, the caller MAY hand the name to
  `ag-domains` attachment; the module does not do it implicitly.

### 4.4 Adapters

Each registrar is a feature-gated adapter (e.g. `registrar-<name>`), confined to
its module, named per ADR-0011 (a third-party brand is allowed only as an
explicit adapter label, never as a generic component name). The default,
no-adapter build is a no-op surface.

### 4.5 Credentials and money (security)

- Registrar credentials and any payment/billing tokens are read from the
  environment/secret store, never hardcoded (CLAUDE.md rule 16), and are confined
  to the adapter behind its feature.
- Purchase/renew/transfer are irreversible, money-moving operations and require
  explicit confirmation in any CLI surface (dry-run first), consistent with
  destructive-op safety.

### 4.6 No changes to: DSL, master docs (beyond a roadmap phase-F note), CI
beyond adding the crate to the workspace matrix when implementation lands.

## 5. Implementation plan (only after approval)

1. PR 1: create the empty `ag-registrars` crate (native, no adapters), the
   `Registrar` trait, `Availability`/`Registration`/`PurchaseOrder` types, and
   contract tests against a mock registrar. No real adapter.
2. PR 2: one reference adapter behind a feature, with `wiremock` unit tests and
   `#[ignore]` real-credential tests.
3. PR 3: optional renewal scheduling that reuses `ag-domains` ARI-aware renewal
   patterns where applicable; CLI surface with dry-run.
4. Each PR ships docs and a capability matrix; no PR couples `ag-domains` to
   `ag-registrars`.

## 6. Risks

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| Scope creep into a full reseller/registrar | Medium | High | Hard boundary in §4.1; trait stays minimal; optional crate |
| Dependency cycle with `ag-domains` | Low | High | One-way `ag-registrars -> ag-domains`; CI cycle check |
| Credential/payment mishandling | Low | High | Env/secret-store only, confined adapters, explicit confirmation, dry-run |
| Maintenance burden of many adapters | Medium | Medium | Adapters are opt-in and community-extensible behind features |
| Fabricated pricing/availability | Low | Medium | Prices come only from the adapter's live API; never synthesized |

## 7. Impact

- Product scope: adds an optional commerce surface without changing the core's
  positioning; `ag-domains` stays "attach, not buy".
- Roadmap: a Phase-F item; does not affect the pre-Phase-5 gate.
- Operational complexity: none by default (no-op build); opt-in only.
- Public APIs: a new crate surface; no change to `ag-domains`/`ag-edge`.
- Documentation: a new module card and capability matrix; a roadmap phase-F note.

## 8. Rollback

Because `ag-registrars` is optional and nothing depends on it, rollback is
removing the crate (or leaving it unbuilt). No core module unwinds. Signals to
roll back: adapter maintenance cost outweighs adoption, or a credential/payment
incident.

## 9. Decision

- Decider: Angel Nereira (BDFL).
- Decision date: 2026-06-11.
- Outcome: accepted (design only; comment period waived by BDFL).
- Rationale: the design keeps `ag-domains`/`ag-edge` provider- and
  registrar-agnostic (no core dependency on registrar commerce), confines the
  commercial concern to an optional Phase-F crate, and follows the brand policy
  (ADR-0011) and native-first principle (ADR-0009). Approval authorizes the
  design as the reference for Phase F; per issue #93, no code is written until
  Phase F begins.

## 10. References

- Issue #93 (this RFC's tracking issue), issue #76 (ag-domains remaining work).
- RFC-0011 (ag-domains control plane), RFC-0007 (ag-domains scope), ADR-0012
  (ag-domains control plane), ADR-0009 (native-first), ADR-0011 (third-party
  brand policy), CLAUDE.md rules 12, 14, 15, 16, 28.
