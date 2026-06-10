# docs: align all READMEs and indexes with verified repository state (Phase 4.6 close-out)

## Summary

Repository-wide documentation honesty pass closing the Phase 4.6 documentation:
every README is reconciled with the verified state of the code (ADR-0009 /
CLAUDE.md rule 26), frozen-`docs/DEBT.md` pointers are redirected to GitHub
Issues (rule 29), the ADR/RFC indexes gain their missing entries, a stale
duplicate RFC file is removed, and the missing example README is added.
After this lands, the remaining Phase 4.6 work lives exclusively in GitHub
Issues (#103, #108, #109, #110, #112, #113, #114).

## Phase affected

Phase 4.6 (documentation close-out). No code behavior changes; documentation
only, plus the removal of one stale duplicate RFC file.

## Type of change

- [x] Documentation (alignment, restructuring, honesty corrections)

## What changed

- `crates/ag-workers/README.md` + `docs/modules/ag-workers/README.md`: status
  updated from "S1-S3" to the real S1-S5 done / S6 partial (#112) / S7 M0-M2
  (#109/#103); exit criteria mirrored from `docs/roadmap/STATUS.md`;
  "Planned" headings corrected to implemented.
- Root `README.md` (EN + ES, English canonical): status line re-verified
  2026-06-10 including Phase 4.6; architecture map and roadmap tables gain the
  4.6 row; `ag workers run` and `ag domains diagnose` added to the CLI table;
  overclaims removed (UI/AI/mobile/WASM/cloud/migrate are placeholder crates,
  not "usable today" — the English section claimed more than the Spanish one);
  debt pointer redirected from frozen `docs/DEBT.md` to GitHub Issues.
- `crates/ag-core/README.md`: rewritten in English; it still described the
  Phase-1 bootstrap and listed CORS/CSRF/rate-limit/JWT/TLS/validation as
  missing when all exist with tests (`src/shield/`, `tests/shield_*.rs`).
- `crates/ag-cli/README.md`: adds the `ag workers` group (`workers-runtime`
  default feature), the full `ag domains` subcommand surface and the
  `DATABASE_URL` row.
- `crates/ag-dsl/README.md`: v0.1..v0.8 (includes the `worker` declaration);
  `docs/dsl/README.md` points to where v0.5+ additive declarations live.
- DEBT.md redirections to Issues in: `crates/ag-data`, `crates/ag-domains`
  (also fixed the self-contradictory "deferred" list naming implemented
  phases), `crates/ag-mail`, `docs/modules/ag-mail` (notes the durable spool
  is now served by `workers-postgres`, parity in #109), `docs/ag-domains`.
- `examples/README.md` migrated to English (ADR-0008) and lists the five
  workers examples; `docs/examples/README.md` catalog updated to ten examples;
  `examples/workers-mail-integration/README.md` created (it was the only
  example without one).
- `docs/modules/README.md`: roster gains `ag-workers` and `ag-lsp`; notes the
  additive growth (`ag-edge`, `ag-workers`) and that `ag-edge` has no module
  sheet yet.
- `docs/roadmap/STATUS.md`: header re-dated; S6 references #112; the
  coverage/fuzz exit criterion marked done (82.28% + `fuzz_workers_payload`
  in the CI fuzz-smoke job). `docs/roadmap/README.md` explains where the
  additive 4.6 sub-phases are tracked.
- `docs/adr/README.md` + `docs/rfc/README.md`: indexes gain ADR-0013 and
  RFC-0012/0013/0014; the stale pre-renumber duplicate
  `RFC-0012-capability-filesystem-confinement.md` is removed (its content
  lives in `RFC-0013-capability-filesystem-confinement.md`; nothing referenced
  the old file).
- `docs/en/README.md` + `docs/es/README.md`: rewritten to state the ADR-0008
  language policy (they still declared the superseded Spanish-default rule).
- `docs/fuzz/README.md`: target table now lists all seven fuzz targets.
- `benchmarks/README.md`: states the real benchmark situation (crate-level
  criterion suites in `crates/*/benches/`; the cross-crate folder is empty).

### Architecture-map / crate-roster honesty (second batch)

A review found `ag-workers` was misclassified or omitted from the canonical
crate roster in several places, and the masters had never been updated when
`ag-workers`, `ag-edge` and `ag-lsp` were added (still said "17 crates"):

- Root README architecture map (EN+ES): restructured to the canonical four-tier
  classification of `CLAUDE.md` §14 so `ag-workers` sits in **deferred standard**
  next to `ag-mail` (it had been lumped under "edge"); also corrected `ag-data`
  (Standard, not Core) and `ag-wasm-host` (Core, not Extended).
- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` (EN+ES rosters, deferred
  paragraphs, intro lists, dir-tree diagrams), `Blueprint-v4.1.md` (EN+ES) and
  `Hoja-de-Ruta.md` (EN+ES count): added `ag-workers` (deferred standard),
  `ag-edge` (optional infra) and `ag-lsp` (core); count 17 -> 20.
- `CLAUDE.md` §14 (the constitution): added `ag-workers` to deferred standard,
  `ag-lsp` to core, `ag-edge` to optional-infra; count note updated. This is a
  sync of the already-approved `ADR-0013`/`ADR-0012`, not a new decision.
- `docs/architecture/03` and `05`, `docs/roadmap/calendar.md`: added the missing
  crates; the calendar also had stale phase statuses (3/4/4.5 shown as
  pending/upcoming) and no 4.6 row -- corrected.
- `tools/split-masters.sh`: its hardcoded line ranges were stale by 10-35 lines
  (the masters were restructured for ADR-0008 without updating the tool) and it
  was missing the `fase-04-5` entry. Re-derived all ranges from the current
  `## ` headings and added a warning that a regen migrates derived docs to
  English (ADR-0008) and must be reviewed. The deeper master<->derived EN/ES
  reconciliation is tracked in Issue #117.

Historical/immutable docs that legitimately predate `ag-workers` (ADR-0007,
RFC-0006, old PR drafts and audits) were intentionally left unchanged.

## Related documents

- CLAUDE.md rules 26 (README sync), 29 (Issues, frozen DEBT.md), ADR-0008
  (language), ADR-0009 (real status).
- `docs/roadmap/STATUS.md` Phase 4.6-D; RFC-0012 / ADR-0013.

## Test plan

- [x] Documentation-only change set; no Rust source touched, so no
  fmt/clippy/test deltas are expected. CI (`ci`, `docs`, `quality`) runs on
  the PR and must stay green, including the `prohibited content scan` and the
  coverage gate.
- [x] Every factual claim cross-checked against the code in this session
  (CLI subcommand enums, shield modules and tests, crate `src/` contents,
  fuzz targets on disk, feature definitions in Cargo.toml).

## Exit criteria advanced (docs/roadmap/STATUS.md)

- Phase 4.6-D coverage/fuzz exit criterion marked complete (delivered by the
  previous PR; reconciled here). Remaining 4.6 work is tracked only in Issues.

## Final checklist

- [x] Belongs to the correct phase (4.6 documentation close-out).
- [x] Respects documentation (docs are reconciled to code, not the reverse).
- [x] No architecture changes; no new dependencies; no circular dependencies.
- [x] Documentation updated in the same change as the state it describes.
- [x] No emojis; English canonical with bilingual showcase sections preserved.
