# ADR-0020 - ag-data typed ORM, RLS and tenancy: accepted as design, implementation deferred

- Status: accepted (design only; implementation deferred)
- Date: 2026-11-22
- Decider: BDFL (Angel Nereira, project maintainer); recorded by the architecture committee chair
- Origin RFC: RFC-0025
- Related issues: #154, #185

## Context

RFC-0025 is a design-only proposal for DSL-generated typed repositories, an
explicit `TenantContext`, transaction-local row-level security and
expand/contract migrations for `ag-data`. It targets Phase 5+ and states that
no increment may start while the pre-Phase-5 release gate is open. On
2026-11-22 that gate at `docs/audits/PRE_FASE5_RELEASE_GATE.md` is still open:
Audit and Deny are `fail` (issues #173, #174), and Fuzz 24h, Benchmarks and
Open debt are `pending` (issues #156, #157). The timestamp type mapping also
depends on the issue #185 / RFC-0026 decision. The design incorporates the
reviewed technical input at `docs/modules/ag-data/data-layer-rls-tenancy.md`.

## Decision

1. RFC-0025 is ACCEPTED AS DESIGN. Its increments I1-I3 are the specified
   contract for a future Phase 5+ implementation; this acceptance authorises
   no API, crate, phase or release-gate change.
2. IMPLEMENTATION IS DEFERRED. No PR may start I1-I3 until Phase 5 opens and
   every blocking row of `docs/audits/PRE_FASE5_RELEASE_GATE.md` is `pass`.
   The issue #185 timestamp decision must be recorded before the DSL type map
   is frozen in generated code.
3. Issue #154 stays open. A separate implementation PR must record the gate
   closure and the phase before merging any increment (`CLAUDE.md` rule 8).
4. The RFC status line and section 10 record this decision with date and
   decider (`CLAUDE.md` rule 28).

## Consequences

- Contributors have one authoritative answer: the design is settled, but no
  code is expected yet. This removes the ambiguity between "proposed" and
  "ready to implement".
- No build, test, fuzz, diagnostics or golden-artifact gate is touched today.
- The RLS/tenancy design remains dependent on the #185 decision; that
  dependency stays explicit instead of being hidden in a provisional mapping.
- Deferral is not rejection: once the gate closes, implementation follows the
  five-PR plan in RFC-0025 section 5 without reopening the design.

## Alternatives

- **Accept and implement immediately:** rejected, it advances a Phase 5+
  capability while the pre-Phase-5 gate has failing and pending rows
  (`CLAUDE.md` rules 5, 8, 22).
- **Reject the RFC:** rejected, it would discard a reviewed design that the
  phase contract already anticipates and leave #154 without a contract.
- **Keep it "proposed" indefinitely:** rejected, contributors would keep
  guessing whether the design is stable (`CLAUDE.md` rule 6).

## Status

Accepted as design on 2026-11-22 by the BDFL. Implementation deferred until
Phase 5 opens and the release gate closes. Supersedes the "proposed" status
of RFC-0025; see `docs/rfc/README.md`.
