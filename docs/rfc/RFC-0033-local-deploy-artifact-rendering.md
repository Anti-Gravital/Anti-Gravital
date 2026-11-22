# RFC-0033 - Local deployment artifact rendering (`ag deploy`)

- Status: proposed for the local renderer; authorized for implementation by the
  wave 7 community coordination. This decision does NOT approve the Phase 5
  `ag-cloud` deploy described in `docs/architecture/10-despliegue-ag-cloud.md`.
- Decision date: 2026-11-21 (wave 7 coordination)
- Author: Gravital Labs community
- Target phase: additive wave 7 (deployment design), before Phase 5
- Affected modules/crates: `ag-cli` only
- Predecessors: RFC-0021 (sections 4.3, plan and apply), RFC-0024 (section 9),
  RFC-0013 (capability confinement), ADR-0008 (English canonical)

## 1. Title

Capability-scoped, plan-first local rendering of Docker and Kubernetes
artifacts through `ag deploy plan|apply|status`.

## 2. Motivation

The wave 7 deployment work produces Dockerfiles, compose files and Kubernetes
manifests under `deploy/docker/` and `deploy/k8s/`. Projects need a predictable
way to turn those authored files into a reviewable deployment artifact set
without installing a container runtime or connecting to a cluster. RFC-0021
requires a plan identifier before any mutating action. The same interface must
work offline and be usable by scripts through the versioned JSON contract.

## 3. Problem

1. Copying deployment files by hand is error prone and unreviewable.
2. RFC-0021 section 4.3.4 requires a plan id and explicit apply for mutating
   commands; no `ag-cli` command currently implements that pair.
3. The existing documentation says `ag deploy` is a Phase 5 command. A local
   renderer must be distinguishable from the cloud deploy so the contract does
   not promise cluster operations it cannot perform.
4. RFC-0024 section 9 prerequisite 5 requires one plan model: a mutating tool
   either returns a plan id for follow-up apply, or uses the plan/apply pair
   consistently, not both in one contract version.

## 4. Alternatives considered

- **A. Call `docker build` and `kubectl apply`.** Rejected: requires a runtime
  and a cluster, contradicts rule 12 (integrate, never replace) and adds a
  large attack surface. Not needed to produce artifacts.
- **B. Generate manifests from the DSL.** Rejected for this scope: the Docker
  and Kubernetes roles own the authored files. Fabricating manifests would
  duplicate their contract and drift. The renderer copies verbatim or fails.
- **C. A global `--plan`/`--apply` flag pair on every mutating command.**
  Rejected here: RFC-0024 section 9 prerequisite 5 forbids two plan models in
  one contract version. The explicit command pair is used consistently.
- **D. Local render with explicit plan id (chosen).** No external dependency,
  reviewable, deterministic, and a clean training example for the plan model.

## 5. Design

### 5.1 Capability and boundaries

The capability is `deploy.render`: read authored files under `deploy/docker/`
or `deploy/k8s/` in the current project and write copies under
`deploy/generated/`. It never opens a socket, never starts a subprocess and
never reads credentials. Files are discovered by walking the source directory
without following symlinks; only regular files with documented names are
considered. All destinations are built from relative components obtained from
that walk, so they cannot escape `deploy/generated/` within the project.

Writes use `cap_std::fs::Dir` (the same capability confinement RFC-0013
established for `ag-storage`), rooted at `deploy/generated/`. `cap-std` is
already a workspace dependency, mature, maintained and used in this workspace;
it is justified here by the need to refuse symlink redirection of destination
paths. No new transitive dependency is introduced.

No cluster, registry, container runtime, DNS or network API is contacted. This
command is NOT the Phase 5 `ag-cloud` deploy: it produces files for review by
the operator's own tooling.

### 5.2 Documented source set

| Target | Source directory | Accepted files | Required marker |
| --- | --- | --- | --- |
| `docker` | `deploy/docker/` | `Dockerfile`, `Dockerfile.*` (for example `Dockerfile.template`), `*.Dockerfile`, `.dockerignore`, `.env.example`, `docker-compose.yml`, `docker-compose.yaml`, `compose.yml`, `compose.yaml`, and any `*.yml`/`*.yaml` under `compose/` | at least one Dockerfile-named file |
| `k8s` | `deploy/k8s/` | any `*.yaml` or `*.yml`, recursively (root, `base/`, overlays, components) | `kustomization.yaml` or `kustomization.yml` |

These names match the artifacts produced by the wave 7 `docker` and `k8s`
roles. Documentation such as `deploy/docker/README.md` and verification
scripts are intentionally not rendered.

If the directory is absent, empty or lacks the required marker, the command
fails with a message naming the missing set. No placeholder content is
invented.

### 5.3 Commands

- `ag deploy plan --target docker|k8s` (read-only): inspects the project,
  reads the source set and prints a deterministic plan: target, detected
  project name, every destination path with byte count and a per-file
  fingerprint, and the plan identifier. No file is written.
- `ag deploy apply --plan ID` (mutating): parses the target from the id,
  recomputes the plan from current files and refuses to continue when the id
  no longer matches (RFC-0021 section 4.3.4 state-change rule). On success it
  writes each file verbatim under `deploy/generated/`, writes
  `deploy/generated/state.json` and appends a local audit line to
  `deploy/generated/audit.log`.
- `ag deploy status` (read-only): reports the recorded artifact set, whether
  the source set is still present and whether the applied plan is current.

`apply` has no JSON shape and refuses `--output json` before doing work, as
required for mutating commands. Only `plan` and `status` add versioned shapes;
this additive change bumps the CLI JSON contract minor version.

### 5.4 Plan identifier

```
id := "dpl-" + target + "-" + hex16(FNV1a64(canonical))
canonical := "ag-deploy-plan-v1\n"
             + field(target) + field(project)
             + for each sorted file: field(source) + field(bytes) + field(fingerprint)
field(v) := decimal_byte_length(v) + ":" + v + "\n"
fingerprint := hex16(FNV1a64(file_bytes))
```

FNV-1a 64 is a deterministic identity and drift detector, not a cryptographic
signature. The id is stable across machines for identical inputs. Replacing,
adding or removing any source file changes it. Collisions are not a security
boundary: this renderer has no privilege beyond local file writes.

### 5.5 Exit codes and errors

Prose mode keeps the existing convention: 2 for usage errors (invalid target,
malformed or missing plan id, `--output json` on `apply`), 1 for failures
(missing source set, stale plan, I/O). JSON mode uses `read_failed` for a plan
whose source set cannot be read, matching the existing versioned error set.

## 6. Risks

| Risk | Mitigation |
| --- | --- |
| Source filenames differ from the documented set as roles publish | Discovery accepts documented variants and fails loudly; no fabricated content. |
| Confusion with Phase 5 cloud deploy | RFC scope statement, help text and README wording state local-only explicitly. |
| Path traversal or symlink escapes | Walk without following symlinks; destinations built from relative components. |
| Plan id collision or forgery | Documented as identity only; no authority beyond local writes. |
| JSON contract drift | Version bumped with schema, contract doc, constant and snapshots in one change. |

## 7. Impact

- New CLI command group and one new module in `ag-cli` (`commands/deploy.rs`).
- JSON contract `0.2.0` adds `deploy.plan` and `deploy.status` shapes.
- Documentation: RFC index, `docs/modules/ag-cli/README.md`, MCP contract
  files, CHANGELOG and bilingual README command tables.
- `ag-cli` gains the already-vendored `cap-std` workspace dependency (RFC-0013
  confinement). No new transitive dependency, crate, CI change or changes to
  other roles' directories.

## 8. Rollback

Remove the `Deploy` variant from `cli.rs`, delete `commands/deploy.rs`, revert
the JSON contract to `0.1.0` and remove the generated documentation entries.
No state outside a project's `deploy/generated/` is created, so rollback
leaves no infrastructure behind.

## 9. Testing

Unit tests cover plan determinism, id parsing, missing sources, stale-plan
refusal and verbatim writes. Integration tests cover plan/apply parity through
the real binary, refusal to apply without a valid plan id, JSON-only read
commands, and a sentinel `PATH` check proving no container or cluster command
is invoked.
