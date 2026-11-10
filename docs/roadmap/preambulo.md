# Hoja de ruta: como leer este documento

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Siguiente: [fase-00-fundaciones-y-gobernanza.md](./fase-00-fundaciones-y-gobernanza.md)

## How to read this document

This document defines the sequence of phases that the Anti-Gravital project must pass through from its inception until it becomes a stable, market-ready version 1.0, with the promise fulfilled.

Each phase contains four blocks:

1. **Entry criteria**: conditions that must be met before the phase can begin. These come from the previous phase.
2. **Deliverables**: concrete artifacts that the phase must produce.
3. **Exit criteria (gate)**: conditions that must be met before moving to the next phase. They function as blocking gates: if they are not met, there is no advancement. This is non-negotiable.
4. **Phase-specific risks and mitigations**.

The deliverables and exit criteria are expressed as checkable boxes. This document is kept in the repository and is updated by crossing off what has been accomplished. It serves as the project's public dashboard.

The main rule is: **a phase is not considered concluded until all of its exit criteria boxes are checked**. The project may be temporarily paused between phases, but it cannot skip steps due to external pressure or urgency.

---

## Phase summary

Status uses explicit evidence-based labels, not a binary "complete". A phase is
not called complete while a blocking technical criterion is still open; external
adoption criteria (stars, contributors, blog posts) are tracked separately and do
not block technical advancement. The per-phase remaining gate work is in the
README "Evidence-based roadmap" table and the formal `PRE_FASE5_RELEASE_GATE.md`.

| Phase | Name                                       | Estimated duration | Status    |
|-------|--------------------------------------------|--------------------|-----------|
| 0     | Foundations and governance                 | 1–2 months         | In progress (external deliverables pending) |
| 1     | The Shield MVP                             | 2–3 months         | Implemented and tested; reference performance/coverage gate open |
| 2     | The Core MVP + roundtrip                   | 2 months           | Implemented; published benchmarks below the 40K req/s and p99 targets |
| 3     | Anti-DSL alpha (v0.1–v0.4)                 | 3 months           | Implemented; 24h fuzz and generated-vs-manual benchmark gates open (issue #70) |
| 4     | Standard modules (auth, data, realtime)    | 3 months           | Implemented and tested; scale evidence and documented debt open |
| 4.5   | `ag-mail` + `ag-domains`: communication and domains | 1–2 months | Implemented; `ag-domains` active, release/doc evidence open |
| 4.6   | Additive pre-Phase-5 hardening (`ag-mail` MTA, `ag-workers`) | — | Implemented and CI-verified; live-DB parity and producer wiring open (issues #108/#109/#103/#112) |
| 4.7   | Interactive terminal experience (`ag` TUI) | 1–2 months         | Proposed (`RFC-0018`); no code yet |
| 4.8   | Project foundations: layers, environment, installation, remote operation | 2–3 months | Proposed (`RFC-0019`, `RFC-0020`, `RFC-0021`); no code yet |
| 5     | `ag-cloud` — simplified deployment         | 2 months           | Next |
| 5.5   | Serverless functions                       | 2 months           | Proposed (`RFC-0023`); no code yet |
| 6     | `ag-ai` and Knowledge Graph                | 2 months           | Pending |
| 7     | `ag-migrate` — importers                   | 2 months           | Pending |
| 8     | `ag-mobile` — Flutter bridge               | 2 months           | Pending |
| 9     | WASI plugin system                         | 2 months           | Pending |
| 10    | Hardening and 1.0 milestone                | 3 months           | Pending |

**Total estimated duration:** 25–30 months from the start.
**Public beta version milestone (0.5):** end of phase 5 (~15 months).
**Stable version 1.0 milestone:** end of phase 10 (~30 months).

Phases 4.5, 4.6, 4.7 and 4.8 are additive: they do not move the two
milestones above, although Phase 4.8 stages A and B change what Phase 5 builds
on and are entry criteria of its stage 5-A. Phase 4.7 runs in parallel with the closing of the pre-Phase-5 gate and
with Phase 5; its stages A and B are exit criteria of Phase 5 (section 5.3) and
its stage C is an entry criterion of Phase 10 (section 10.1).

**How later phases are detailed (amendment of 2026-11-10).** From Phase 5
onward each phase carries, in addition to the four blocks, a **Stages** block
that orders its deliverables into independently mergeable steps and a list of
**RFC prerequisites** (rule 28: new commands, new DSL constructs and new
dependencies need an approved RFC before code). Stages add no scope: every
stage item is a deliverable already listed in the phase.

**Status at the close of Phase 4.5 (2026-05-24), updated for the pre-Phase-5
gate (2026-06-11).** Phases 1 through 4.5 are technically implemented and merged
to `main` (code, tests, fmt, clippy, audit and deny deliverables fulfilled), but
none is declared formally complete while the pre-Phase-5 release gate
(`docs/audits/PRE_FASE5_RELEASE_GATE.md`) is OPEN: the 24-hour fuzz, stabilized
benchmark and open-debt rows remain pending. The granular detail of each box
lives in `docs/roadmap/STATUS.md`, the operational dashboard. Phase 0 remains in
progress due to external deliverables (Discord, landing, domain). Phase 5
(`ag-cloud`) is next and may not start until every blocking gate row passes.

**Note on Phase 4.5.** Phase 4.5 is an **additive** phase introduced by
`ADR-0007` after closing Phase 4. It does not modify the scope nor the
deliverables of the already-completed Phase 4. It does not advance the v0.5 BETA milestone, which
remains at the end of Phase 5. The ecosystem count goes from 15 to 17
crates with the incorporation of `ag-mail` and `ag-domains`, and grows
additively to 20 with `ag-lsp` (Phase 3 DSL tooling), `ag-edge`
(`ADR-0012`) and `ag-workers` (`ADR-0013`, the second deferred standard).

> Phases 0-4.5 are technically implemented; formal completion is gated by the
> pre-Phase-5 release gate. Outstanding technical debt that must be closed before
> Phase 5 is tracked as GitHub Issues (label `tech-debt`, CLAUDE.md rule 29);
> `docs/DEBT.md` is a frozen historical record.

---

