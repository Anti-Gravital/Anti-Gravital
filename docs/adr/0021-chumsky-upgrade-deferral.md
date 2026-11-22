# ADR-0021 - Defer the chumsky upgrade and keep 0.9.3

- Status: accepted (recommendation of RFC-0027)
- Date: 2026-11-22
- Decider: BDFL (Angel Nereira, project maintainer); recorded by the architecture committee chair
- Origin RFC: RFC-0027 (amendment to RFC-0003)
- Related issues: #180, #156

## Context

RFC-0027 evaluates replacing `chumsky` 0.9.3 with the maintained 0.13+/0.14
line in the Anti-DSL parser. The evaluation records that 0.9.3 has no RUSTSEC
advisory, that MSRV is not a blocker, and that no functional limit or measured
parse-time bottleneck of 0.9 is demonstrated. The migration is a rewrite of
the 1,482-line `parser.rs`: new parser lifetimes and input type, `Stream::map`
for byte spans, `parse().into_output_errors()`, `map_with()`, a two-argument
`skip_then_retry_until`, `select!` clones, and a move from `Simple` to `Rich`
errors to preserve the expected-token hint. It would reset the diagnostics
golden tests (which do not exist yet), the fuzz baselines and the
byte-identical `examples/ecommerce-api` artifacts. The 24-hour fuzz gate
(#156) was never completed, not merely reset. The low-level review at
`docs/audits/low-level-review-178-180.md` was incorporated.

## Decision

1. RFC-0027 is ACCEPTED with its recommendation: DEFER the upgrade. Keep
   `chumsky = "0.9"` in the workspace manifest and `chumsky 0.9.3` in
   `Cargo.lock`.
2. Re-evaluation is triggered only by (a) the DSL v1.0 grammar freeze, (b) a
   RUSTSEC advisory on chumsky 0.9.x, (c) a concrete DX defect that 0.9 cannot
   express, or (d) a tracked benchmark showing a parser-attributable
   bottleneck. On those triggers a successor RFC supersedes RFC-0027.
3. If a migration is later approved, target the newest stable 0.14.x at
   implementation time, not the 0.13.0 named in issue #180, and follow the
   conditional plan in RFC-0027 sections 4.2 and 5: golden diagnostics tests
   first, a benchmark with rule 17 metadata, fuzz baselines before and after,
   and byte-identical generated artifacts.
4. RFC-0003 keeps chumsky 0.9 as the Fase 3 choice; RFC-0027 resolves its
   "future RFC" placeholder with a deferral. No public API, DSL grammar,
   master document, dependency or CI change results from this decision.
5. Issue #180 stays open until the board reconciles it; accepting a deferral
   closes no gate.

## Consequences

- No parser rewrite, no dependency churn and no reset of fuzz, diagnostics or
  golden-artifact evidence while the release gate is open.
- The baseline recorded in RFC-0027 is informational only; it is not a
  0.9-vs-0.13 comparison and must not be cited as one.
- A later migration starts from a larger grammar, so the deferral is reviewed
  at DSL v1.0 freeze rather than treated as permanent.
- Contributors have an explicit answer to issue #180 and stop treating the
  upgrade as an implied pending task.

## Alternatives

- **Migrate now to 0.14.x:** rejected, concrete rewrite cost with no measured
  benefit and three quality gates at risk while #156 is pending.
- **Pin a 0.10/0.11/0.12 stepping stone:** rejected in RFC-0027; 0.11 is
  yanked and the total work is no smaller.
- **Replace the parser (hand-written, lalrpop, pest):** rejected in RFC-0003
  and unchanged by new evidence.
- **Leave RFC-0027 "proposed":** rejected, the evaluation is complete and the
  open status invites repeated re-evaluation (`CLAUDE.md` rule 28).

## Status

Accepted on 2026-11-22 by the BDFL. Recommendation to defer adopted; keep
0.9.3. See `docs/rfc/README.md` and the decision line in RFC-0027 section 9.
