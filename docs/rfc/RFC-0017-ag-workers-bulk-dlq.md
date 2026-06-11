# RFC-0017 - ag-workers: bulk DLQ re-drive and purge-by-filter

- Status: proposed (pending review and approval)
- Author: Gravital Labs - Nereira Technology and Business Solutions
- Draft date: 2026-06-11
- Target phase: maintenance of Phase 4.6-D ag-workers (CLI surface extension)
- Affected modules/crates: `ag-cli` (`Dlq` subcommands), `ag-workers` (DLQ query
  /re-drive API)
- Predecessor RFC: RFC-0012 (ag-workers)
- Comment period: minimum seven calendar days

## 1. Motivation

The DLQ CLI (RFC-0012 §20/§27) operates one job at a time for re-drive
(`ag workers dlq retry JOB_ID`) and purges only by age
(`ag workers dlq purge --older-than 30d`). When a downstream dependency recovers,
an operator typically needs to re-drive *many* dead-lettered jobs of one
`kind`/`queue` at once, not paste IDs one by one. This is a standard day-2 DLQ
operation. This is issue #114.

This RFC is required because it extends the CLI surface, which CLAUDE.md rule 28
gates behind an approved RFC.

## 2. Problem

- RFC-0012 §27 specifies single-ID re-drive and age-based purge only.
- Bulk re-drive/purge by `queue`/`kind` is missing, so recovery from a
  dependency outage is tedious and error-prone.
- Bulk operations are destructive/large-effect and need a safety rail.

## 3. Alternatives considered

1. **Do nothing.** Operators script around the single-ID commands. Pro: no new
   surface. Con: poor day-2 ergonomics; scripting re-drive loops is exactly what
   the tool should provide. Rejected.
2. **External tooling only.** Rejected: pushes a common operation outside the
   product and duplicates DLQ knowledge.
3. **Add bounded bulk subcommands with a dry-run (this RFC).** Chosen.

## 4. Proposed design

Extend the existing `Dlq` subcommands (behind the existing `workers-runtime`
feature), adding filtered, bounded bulk variants:

```text
ag workers dlq retry  --queue mail [--kind send_receipt] [--limit N] [--dry-run]
ag workers dlq purge  --queue mail [--kind X] [--older-than 30d] [--dry-run]
```

- `--queue` is required for bulk operations; `--kind` narrows further.
- `--limit N` bounds how many entries one invocation affects (default a
  conservative value, e.g. 100) so a single command cannot re-drive an unbounded
  flood.
- `--dry-run` prints exactly which entries would be affected (count + sample) and
  makes no change — required before any destructive bulk action.
- `purge` keeps `--older-than` and now also accepts the filter, so age and filter
  compose.

`ag-workers` gains a small bulk query/re-drive API on the DLQ store
(filter by queue/kind, with a limit), reused by both the in-memory and Postgres
backends. Single-ID `retry`/`purge` stay unchanged (backward compatible).

No DSL, master-doc or CI changes beyond documenting the new flags.

## 5. Implementation plan (only after approval)

1. PR 1: `ag-workers` DLQ filter+limit query and bulk re-drive/purge API, with
   unit tests on the in-memory backend and `#[ignore]` Postgres parity tests.
2. PR 2: `ag-cli` flag surface (`--queue`/`--kind`/`--limit`/`--dry-run`), dry-run
   output, and docs (modules/ag-workers + README CLI table).

## 6. Risks

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| Mass re-drive overwhelms a still-degraded dependency | Medium | Medium | `--limit` bound + `--dry-run` preview; default conservative limit |
| Accidental over-broad purge | Low | High | `--dry-run` required workflow; filter is explicit; single-ID path unchanged |
| Backend divergence (memory vs Postgres) | Low | Medium | Shared API + parity tests |

## 7. Impact

- Product scope: an additive CLI ergonomics improvement; no new concept.
- Public APIs: new CLI flags and a small `ag-workers` DLQ API; existing commands
  unchanged.
- Operational complexity: lower (recovery is one command, safely previewed).
- Documentation: ag-workers module docs + README CLI table.

## 8. Rollback

Remove the bulk flags and the bulk API; single-ID `retry`/`purge` remain. No
state migration is involved.

## 9. Decision

To be completed after the comment period.

- Decider: BDFL or technical committee.
- Decision date: YYYY-MM-DD.
- Outcome: accepted / rejected / deferred.

## 10. References

- Issue #114. RFC-0012 §18/§20/§27. CLAUDE.md rule 28.
- `crates/ag-cli/src/main.rs` (`Dlq`), `crates/ag-workers/src/queue/dlq.rs`.
