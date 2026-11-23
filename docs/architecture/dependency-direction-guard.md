# Dependency-direction guard

## Purpose

`scripts/check-dependency-direction.sh` turns the documented crate dependency
direction (CLAUDE.md rules 14 and 15) into an automatic, reproducible check.
Until this guard existed, the direction was verified by hand. The last manual
pass is recorded in `docs/audits/pre-fase5-architecture.md:48-62`, when the
roster still had 17 crates. Three crates were added afterwards (`ag-lsp`,
`ag-edge`, `ag-workers`) with new documented edges. The proposal to automate
this control is task A5 in `docs/audits/oportunidades-arquitectura.md:231-265`.

The guard is a policy check over manifest text. It is deliberately opt-in and
local: it does not modify CI, does not hook the default verification gate and
does not add a crate or a dependency.

## Verified sources

All sources are in-repo canonical documents, read directly for this change;
no external or unverified source is used.

| Source | Anchor | What it fixes |
| --- | --- | --- |
| `CLAUDE.md` | rule 14, lines 317-377 | 19-crate roster; deferred and optional classes; sixth and seventh dependency rules |
| `CLAUDE.md` | rule 15, lines 380-396 | `ag-core` isolation; no circular dependencies; independent semver; isolated features; minimal deps |
| `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` | section 5.3, lines 1494-1525 | seven numbered dependency rules, including `ag-auth -> ag-mail` (sixth) and `ag-cloud -> ag-domains` (seventh) |
| `docs/adr/0007-ag-mail-ag-domains.md` | sixth-rule decision | unidirectional `ag-auth` / `ag-mail` relationship |
| `docs/adr/0012-ag-domains-control-plane.md` | lines 33-42 and 89 | `ag-edge` is the request-time data plane under the `ag-domains` control plane |
| `docs/rfc/RFC-0011-ag-domains-control-plane.md` | control-plane scope | same `ag-edge` / `ag-domains` direction |
| `docs/rfc/RFC-0012-ag-workers-execution-model.md` | section 4, lines 131-143 | `ag-workers` allowed and forbidden directions |
| `docs/graph/crates.md` | lines 15-49 | curated mirror of classes, direct dependencies and dependency rules |
| `docs/modules/ag-mail/README.md` | lines 160-171 | `ag-mail` may depend on `ag-domains`, never the reverse; must not depend on `ag-auth` |
| `docs/modules/ag-workers/README.md` | lines 100-107 | `ag-workers` must not depend on `ag-mail`, `ag-auth`, `ag-cloud`, `ag-domains`, `ag-edge` or `ag-cli` |
| `docs/superpowers/specs/2026-05-22-ag-storage-design.md` | lines 348-349 | `ag-storage -> ag-auth` is feature-gated; the reverse would be a cycle |

## What the guard asserts

1. **Roster.** Exactly the 19 crates of rule 14 are present. A new crate
   directory requires a reviewed roster update (rule 28 requires an RFC for a
   new crate).
2. **Core isolation.** `ag-core` declares no dependency on any other `ag-*`
   crate, in any dependency table (first rule of section 5.3).
3. **Allow-list.** Every internal edge belongs to the reviewed list below. An
   edge outside it fails with the crate pair, manifest line and a request to
   update the list with an RFC/ADR citation. This is the intended friction:
   no new direction appears silently.
4. **Forbidden reverse directions.** The explicitly documented reverse edges
   are rejected by name, with their source, before the allow-list check.
5. **Required edge.** `ag-edge -> ag-domains` must exist; that dependency is
   part of the definition of the edge data plane (ADR-0012).
6. **Acyclicity.** A depth-first search over the internal graph reports the
   exact cycle path if one exists (no circular dependencies, rule 15).
7. **Fail-closed parsing.** An `ag-*` declaration that the parser cannot read
   is a failure, never a silent pass.

The self dev-dependency in `crates/ag-mail/Cargo.toml` (`ag-mail = { path =
"." }`, explained in a comment in that manifest) is exempt from cycle
detection and counted separately. It is a test-only Cargo pattern that makes
`test-utils` visible to integration tests, not a crate cycle.

## Reviewed allow-list

Derived from the canonical documents above and from the current manifests.
Each row is a dependency declared by the first crate on the second.

| From | To | Source |
| --- | --- | --- |
| `ag-auth` | `ag-data` | section 5.3 rule 2 |
| `ag-auth` | `ag-mail` | rule 14, sixth rule |
| `ag-cache` | `ag-core` | section 5.3 rule 2 |
| `ag-cli` | `ag-core` | section 5.3 rules 2 and 4 |
| `ag-cli` | `ag-dsl` | section 5.3 rule 4 |
| `ag-cli` | `ag-domains` | section 5.3 rule 7 |
| `ag-cli` | `ag-mail` | section 5.3 rule 4 |
| `ag-cli` | `ag-workers` | RFC-0012 section 4 (feature-gated) |
| `ag-data` | `ag-core` | section 5.3 rule 2 |
| `ag-domains` | `ag-core` | section 5.3 rule 2 |
| `ag-edge` | `ag-domains` | ADR-0012 / RFC-0011 |
| `ag-lsp` | `ag-dsl` | rule 14 (core tooling) |
| `ag-mail` | `ag-core` | section 5.3 rule 2 |
| `ag-mail` | `ag-workers` | RFC-0012 section 4 (optional durable delivery) |
| `ag-storage` | `ag-auth` | ag-storage design spec line 348 (feature-gated) |
| `ag-workers` | `ag-data` | RFC-0012 section 4 (durable opt-in) |

Directions that some documents describe as allowed but that are not currently
declared (`ag-cloud -> ag-domains`, `ag-mail -> ag-domains` and
`ag-edge -> ag-workers`) are intentionally not pre-approved. Adding one of
them requires updating the allow-list in the same change, citing the section
that authorizes it. This keeps the list aligned with the code that actually
ships.

## Forbidden directions checked by name

| From | To | Source |
| --- | --- | --- |
| `ag-mail` | `ag-auth` | rule 14, sixth rule; section 5.3 |
| `ag-data` | `ag-auth` | section 5.3 rule 2 |
| `ag-auth` | `ag-storage` | ag-storage design spec line 349 |
| `ag-workers` | `ag-mail`, `ag-auth`, `ag-cloud`, `ag-domains`, `ag-edge`, `ag-cli` | RFC-0012 section 4 |
| `ag-domains` | `ag-edge`, `ag-cloud`, `ag-cli`, `ag-mail` | ADR-0012; rule 7; `ag-mail` module doc line 169 |

## Usage

```bash
# from anywhere in a checkout; the script resolves its own repository root
scripts/check-dependency-direction.sh
scripts/check-dependency-direction.sh --help
```

Exit codes: `0` PASS, `1` FAIL (at least one rule violated, all violations are
printed), `2` ERROR (bad usage or unreadable repository). A `SKIP` status does
not exist: there is no tool to be missing.

The script is deterministic and offline. It uses only `bash`, `awk`, `sort`,
`basename` and standard text utilities; it never reads credentials, never
opens the network and never invokes the Rust toolchain.

## Scope and non-goals

- Opt-in only. No workflow under `.github/` and no default check in
  `scripts/ag-check` is changed by this work.
- The guard does not scan external dependencies for an inline `RFC-XXXX`
  citation. Current manifests do not carry such a comment on every external
  dependency line, so automating that rule would fail on a healthy tree.
  External dependency justification stays a manual review item (CLAUDE.md
  section 15 and policy S-9).
- It complements, and does not replace, Cargo's own resolution and the
  project's tests. It checks declared direction, not compiled usage.

## Maintenance

When an internal edge is added or removed, update the allow-list (and the
tables in this document) in the same change, with the RFC/ADR or section that
authorizes it. If the change alters a documented direction rather than merely
implementing one, an RFC is required (CLAUDE.md rules 22 and 28). Exact
verification output for this introduction is recorded in
`docs/kaizen/pr-drafts/community18-dep-guard.md`.
