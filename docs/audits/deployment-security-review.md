# Deployment security review (wave 7)

- Reviewer role: security (read-only audit; no push, no merge)
- Date: 2026-11-22
- Base commit reviewed: `e9347ab44b58b0c288a3db433dac327da80ee9c6`
- Artifact tips reviewed: docker `397a381`, k8s `75f5ea4`, and the uncommitted
  working trees of `stacks`, `deploy` and `deployqa` at review time.
- Scope: the wave 7 deployment artifacts (`docker`, `k8s`, `stacks`, `deploy`,
  with `design` and `deployqa` as supporting work).
- Method: read-only inspection of each role's worktree, baseline verification
  of the pre-existing container asset, and source-level reproduction. No files
  outside this role's worktree were edited.

## Scope and limitation

This review covers Dockerfiles and compose files (`deploy/docker/`), Kubernetes
manifests and kustomization (`deploy/k8s/`), compose stacks for backing
services (`deploy/stacks/`), and the `ag deploy plan|apply|status`
implementation. Findings are grounded in the actual file contents observed in
those worktrees; each issue records reproduction steps and verification
criteria so the coordinator can re-check the committed tips. Where a role was
still accumulating uncommitted work, its working tree is cited explicitly.

## Threat model

- Image build: no secrets baked into layers; non-root runtime; minimal attack
  surface (static binary in a scratch/distroless image where practical).
- Orchestration: no cluster credentials required by default; dropped Linux
  capabilities, non-root users, read-only root filesystems and resource limits;
  health checks.
- Deploy command: plan/apply must be a real gate (RFC-0021 section 4.3), must
  not open sockets or start subprocesses, must not require Docker/Kubernetes by
  default, and must not gain privilege because the caller is a machine.
- Supply chain: base images pinned where practical; no third-party brand used
  as a project component name (ADR-0011); integration, never reinvention
  (rule 12).

## Mission checklist

| Check | Result | Notes |
| --- | --- | --- |
| Non-root runtime | PASS | Dockerfile `USER 65532:65532`; k8s `runAsNonRoot: true`, `runAsUser: 10001`. |
| No secrets in images/manifests/compose/stacks | FAIL | Issue #195 (Docker `.env` in builder layer); stacks ship usable default passwords (issue #194). |
| Read-only root filesystem where feasible | PARTIAL | k8s `readOnlyRootFilesystem: true` with `/tmp` emptyDir; compose has no `read_only: true`. |
| Resource limits | PARTIAL | k8s requests/limits on base and overlays; compose has none. |
| Dropped capabilities, no privileged mode | PARTIAL | No `privileged: true` anywhere; k8s drops ALL and blocks escalation; compose has no `cap_drop`/`no-new-privileges`. |
| Health checks | PASS | Dockerfile HEALTHCHECK, compose healthchecks, k8s TCP liveness/readiness; worker stack disables it with rationale. |
| Pinned base image tags/digests | PASS | Version tags (`rust:1.99.1`, `busybox:1.36-musl`, `postgres:16-alpine`, `redis:7-alpine`); digests not used but tags are pinned where practical. |
| No cluster credentials required by default | PASS | k8s `automountServiceAccountToken: false`, optional secret ref; image placeholder `replace-me`; no kubeconfig read. |
| Plan/apply cannot be bypassed | PASS | `Apply { plan: String }` is mandatory; CLI refuses `--output json` on apply before dispatch; apply recomputes plan and rejects stale ids. |
| No network calls in deploy | PASS | `deploy.rs` has no socket, HTTP, subprocess or credential access; status works offline. |
| No third-party brand as component name (ADR-0011) | PASS | Project components are `ag-*`; brand names in stacks are confined labels for integrated open-source tools, consistent with rule 12 and the ADR-0011 adapter exception. |

## Findings by artifact family

### Baseline (pre-existing asset at base commit)

#### B1 (observation) - `examples/todo-api/Dockerfile`

Multi-stage `rust:1.99-slim` builder producing a musl static binary copied into
a `scratch` runtime image. No secrets, no privileged mode, minimal surface.
Gaps that do not rise to a merge blocker for this pre-existing example: no
`HEALTHCHECK` (a scratch image has no shell), a minor-version builder tag
rather than a digest, no explicit numeric `USER`, and Spanish source comments
(ADR-0008 canonical English applies to new code). Not a wave 7 artifact.

### Docker (`deploy/docker/`, docker `397a381`)

Strong baseline: multi-stage build to a static musl binary in `scratch`, numeric
non-root `USER`, CA bundle for outbound TLS, a tiny busybox probe for
`HEALTHCHECK`, no privileged mode, no baked secrets, and a static `verify.sh`
that reports honest SKIPs. Three findings:

#### D1 (defect, p2, security) - issue #195: `COPY . .` can bake `.env` secrets into a builder layer

`Dockerfile.template` uses `COPY . .` with no shipped `.dockerignore` and no
root-level one. The documented workflow creates `deploy/docker/compose/.env`
with real `POSTGRES_PASSWORD` / `AI_API_KEY` values, then builds from the repo
root. The `.env` and `.git` history enter the builder layer, contradicting the
README claim that no credential is placed in an image layer. The final scratch
image is clean, but builder layers are distributable. Reproduction and fix in
issue #195. Recommendation: ship `.dockerignore`, align the build instructions,
and add a canary check to `verify.sh`.

#### D2 (observation, low) - compose hardening knobs absent

None of the five compose files declare `cap_drop: ALL`,
`security_opt: no-new-privileges`, `read_only: true`, `mem_limit`/`cpus` or
`pids_limit`. The shipped runtime image is already non-root and shell-free, so
this is defense-in-depth rather than a live vulnerability. Recommendation: add
the three security keys and modest resource limits to each service as the
files move from reference to production use.

#### D3 (observation, low) - `AI_BASE_URL` empty default

`compose/ai.yml` sets `OPENAI_BASE_URL: ${AI_BASE_URL:-}`, which expands to an
empty string. `examples/ai-backend/src/provider/mod.rs` defaults to the public
endpoint only when the variable is *unset*, so an empty value yields an empty
base URL. Functional, not a security defect. Recommendation: use
`${AI_BASE_URL:-https://api.openai.com}` or drop the key when unset.

### Kubernetes (`deploy/k8s/`, k8s `75f5ea4`)

The best-hardened family in this wave. The base Deployment sets
`automountServiceAccountToken: false`, `runAsNonRoot`, `runAsUser`/`runAsGroup`
`10001`, `fsGroup`, `seccompProfile: RuntimeDefault`,
`allowPrivilegeEscalation: false`, `readOnlyRootFilesystem: true`,
`capabilities.drop: [ALL]`, CPU/memory requests and limits, TCP liveness and
readiness probes, and a writable `/tmp` emptyDir. The optional secret ref is
flagged `optional: true`, the image is a `replace-me` placeholder, and overlays
only adjust labels, replicas and resource sizes.

#### K1 (observation, low) - writable path coverage

`readOnlyRootFilesystem: true` provides only `/tmp` as a writable mount.
Templates that persist state under `/var/lib/ag` need an explicit emptyDir or
PVC overlay; without one they fail at runtime. Functional/availability, not a
security defect. Recommendation: document a state-volume overlay for families
that write data.

#### K2 (observation, low) - mutable image tag with `IfNotPresent`

Overlays rewrite the image to `...:replace-me` with `imagePullPolicy:
IfNotPresent`. Operators must change the tag; a reused tag can pull a stale
image. Recommendation: require an immutable tag or digest in the apply
documentation.

### Stacks (`deploy/stacks/`, uncommitted at review time)

Six local stacks (postgres-api, redis-cache, nats-events, mail-sink,
monitoring, object-storage) are mostly well documented: named volumes,
healthchecks, no real credentials committed, and integration-only framing
that preserves native in-memory defaults. One defect:

#### S1 (defect, p2, security) - issue #194: all stacks bind all interfaces despite localhost-only docs

Every host port mapping uses `"${PORT:-N}:N"` (0.0.0.0), while each README
describes the stack as localhost-only. `postgres-api` and `monitoring` also
ship usable default passwords (`ag-local-only`, `admin-local-only`) when `.env`
is absent; the other four expose unauthenticated services. Following the
documented `docker compose up -d` on a LAN-connected host exposes them.
Reproduction and fix in issue #194. Recommendation: `127.0.0.1:` bindings and
fail-closed credentials.

#### S2 (observation) - ADR-0011 review

Stack directories and services use names such as `postgres`, `redis`, `nats`,
`grafana`, `prometheus`, `mailpit` and `s3mock`. These are explicit labels for
integrated open-source tools confined to each stack's compose file and README,
matching the ADR-0011 adapter exception and rule 12 (integrate, never
replace). No Anti-Gravital component or crate is named after a third party.
No defect.

### Deploy command (`ag deploy`, uncommitted at review time)

`crates/ag-cli/src/commands/deploy.rs` plus CLI/JSON wiring implements RFC-0033
cleanly: `plan` is read-only and deterministic; `apply` requires `--plan ID`,
recomputes the current plan and refuses stale ids; `--output json` on `apply`
is rejected before dispatch; `status` works with no Docker, cluster or
network; there is no socket, subprocess or credential access; and writes are
confined lexically to `deploy/generated/` with symlinks skipped during source
discovery.

#### P1 (defect, p2, security) - issue #193: destination symlinks defeat write confinement

Writes use `fs::write`/`fs::create_dir_all` on destination paths, and
`state.json`/`audit.log` are written at fixed paths. All follow symlinks, so a
malicious project can redirect an apply to overwrite files outside
`deploy/generated/`. The plan id does not cover destination state, so drift
detection does not catch it. Reproduction and fix in issue #193.
Recommendation: reject symlink components in every write path and add a
symlink regression test.

#### P2 (observation, low) - audit record completeness

`audit.log` records timestamp, plan id and file count, but not the user or the
result, which RFC-0021 section 4.3.6 lists for mutating actions. Recommendation:
append user and success/failure fields.

#### P3 (observation, low) - source root symlink

`collect_sources` calls `source_root.is_dir()`, which follows a symlink placed
at `deploy/docker` itself and could read authored files from outside the
project. Destinations remain confined, so impact is low. Recommendation:
reject a symlinked source root explicitly.

### Validation harness (`deployqa`, uncommitted at review time)

`scripts/validate-deploy.sh` uses real parsers and reports honest SKIPs. One
observation: RFC-0033 section 9 calls for a sentinel `PATH` check proving no
container or cluster command is invoked; the script checks for network-error
strings on `status` instead. That is a verification gap, not an artifact
defect. Recommendation: add the sentinel `PATH` check.

## Issues opened

| Issue | Severity | Area | Defect |
| --- | --- | --- | --- |
| #193 | p2, security | ag-cli / deploy | `apply` follows destination symlinks; writes escape `deploy/generated/`. |
| #194 | p2, security | stacks | All stacks bind 0.0.0.0 despite localhost-only docs; two ship default passwords. |
| #195 | p2, security | docker | `COPY . .` bakes `.env` secrets into a builder layer; no `.dockerignore`. |

All three include reproduction steps, severity and verification criteria.
Observations D2, D3, K1, K2, S2, P2, P3 and the deployqa gap stay in this note
because they are below the defect threshold.

## Go/no-go recommendation

**NO-GO as the artifacts stand.** The Kubernetes base, the Docker scratch
runtime and the `ag deploy` plan/apply gate are well hardened, and no
privileged mode, cluster credential requirement, hidden network call or
third-party brand component name was found. However, three p2 security defects
must be resolved or explicitly accepted before merge: #193 (arbitrary write
via symlink in `ag deploy apply`), #194 (public exposure of local stacks with
default credentials) and #195 (secret material in a Docker builder layer).

**Conditional GO** once #193, #194 and #195 are fixed and verified with the
criteria in each issue, or consciously waived by the coordinator with the
rationale recorded. The remaining observations are hardening follow-ups and do
not block the merge.
