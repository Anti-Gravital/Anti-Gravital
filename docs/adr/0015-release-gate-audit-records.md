# ADR-0015 - `docs/audits/` as authorized release-gate and audit-evidence records

- Status: accepted
- Date: 2026-06-13
- Source RFC: not applicable (governance clarification of CLAUDE.md rule 29)

## Context

CLAUDE.md rule 29 prohibits creating repository files whose purpose is to
document debt, failures, problem audits or pending work; that information
belongs in GitHub Issues. The stated exception covers RFC/ADR documents
that enumerate phases or limitations as part of a design decision.

`docs/audits/` contains the pre-Phase-5 audit set: a formal release gate
(`PRE_FASE5_RELEASE_GATE.md`) plus per-stage evidence records
(`pre-fase5-security.md`, `pre-fase5-fuzzing.md`, `pre-fase5-build-gate.md`,
etc.). README cites the release gate as the decision that authorizes
advancing to Phase 5.

Their status under rule 29 was ambiguous: they read partly as a decision
contract (the gate) and partly as operational findings (the stage
records), so it was unclear whether they were authorized contract
documents or prohibited debt registers that should live in Issues.

## Decision

`docs/audits/` is authorized as **architectural contract**, with a
bounded scope:

1. **Allowed to live here:** the release-gate decision document (its gate
   table of pass/fail/pending rows, the criteria, and the dated,
   reproducible evidence for each `pass`), and the per-stage audit
   evidence records (what was checked, the command, the result, the
   date/commit). These are the verifiable trail behind a release
   decision, equivalent in nature to an ADR.
2. **Must go to GitHub Issues, not here:** any *open finding* or pending
   actionable work. A gate row in `pending`/`fail` state, or any audit
   note describing work that still has to be done, must reference a
   tracking Issue (label `tech-debt` and the relevant area). The audit
   file states the gate state and links the Issue; it does not carry the
   open finding's long-form description or its plan.
3. **No new free-standing debt/audit files.** This authorization is
   specific to the pre-Phase-5 gate set. A future gate (e.g. pre-Phase 6)
   may reuse the same pattern, but routine debt does not get a new file:
   it gets an Issue.

This clarifies rule 29 rather than weakening it: the single honest view
of *open* state remains the Issues board; `docs/audits/` records
*decisions and evidence*, with every still-open item pointing at an
Issue.

## Consequences

Positive:

- The release gate keeps its auditable, reproducible evidence trail in
  one place, as a decision contract.
- Open work is not duplicated in prose: each pending gate row links a
  single tracking Issue, so the board stays the source of truth.

Negative / cost:

- Contributors must remember that a `pending`/`fail` gate row requires an
  Issue link; a bare `pending` with no Issue is now a governance defect.

## Alternatives considered

- **Treat `docs/audits/` as prohibited and migrate everything to Issues.**
  Rejected: the gate's pass-row evidence (dated commands, commits) is a
  decision record, not open debt; scattering it across closed Issues
  would lose the consolidated release-decision view README depends on.
- **Leave the status ambiguous.** Rejected: ambiguity is exactly what
  rule 29's lifecycle rules forbid; the gate must be unambiguously either
  contract or debt.

## Notes

- Applied reconciliation: the release-gate `pending` rows reference their
  tracking Issues — Fuzz 24h -> #156, Benchmarks -> #157, Open debt ->
  the `tech-debt`-labelled Issues board. Stage records that mention those
  pending items link the same Issues.
- Related: CLAUDE.md rule 29 (debt in Issues), rule 22 (governance),
  rule 36 (reproducibility); ADR-0009 (corrective governance).
