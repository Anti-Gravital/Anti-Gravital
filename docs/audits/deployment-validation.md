# Deployment artifact validation - real evidence

Date (UTC): 2026-11-22T04:48:17Z
Commit under test: `e9347ab44b58b0c288a3db433dac327da80ee9c6`
Branch: `community7/deployqa` (no push, no merge)
Host: `angel-nereira-HP-ProBook-440-14-inch-G9-Notebook-PC`
Harness: `scripts/validate-deploy.sh` (mode 100755)

## 1. Tool versions used (all measured, not assumed)

| Tool | Version | Role in this validation |
| --- | --- | --- |
| bash | GNU bash 5.3.9(1)-release | harness interpreter |
| python3 | 3.14.4 | YAML parser host |
| PyYAML | 6.0.3 | real YAML parser (`yaml.safe_load_all`) |
| kustomize | v5.5.0 | offline kustomization rendering |
| kubeconform | not installed | k8s schema validation: honest SKIP |
| docker client | 29.10.0 (build fdd4b8e) | image build: honest SKIP (no daemon) |
| kubectl client | v1.37.1 | server-side dry-run: honest SKIP (no cluster) |
| rustc / cargo | 1.99.1 / 1.99.1 | `ag` build context |

Cargo environment for every run:
`CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target`
`CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0`

The `ag` binary used was
`/home/angel-nereira/Github-local-angelnereira/shared-cargo-target/debug/ag`,
built from the base commit. It does **not** contain a `deploy` subcommand yet.

## 2. Method

1. Run the harness on the untouched branch (no `deploy/` tree) to establish the
   missing-artifact baseline.
2. Assemble a read-only snapshot of the artifacts currently authored by the
   wave-7 roles by copying `deploy/` from the `docker`, `k8s`, `stacks` and
   `deploy` worktrees into `/tmp/ag-deploy-merged/deploy`. No worktree was
   modified, no branch was merged and nothing was pushed. The snapshot is a
   point-in-time view of in-progress work (42 files: 34 YAML, 1 Dockerfile,
   7 compose docs, env examples and verification scripts). An earlier snapshot
   at 04:49:52Z caught a transient bad relative path in the k8s example
   kustomization (`../overlays/api` instead of `../../overlays/api`); the k8s
   role corrected it before the authoritative snapshot below. That detection
   confirms the kustomize step is a real check, not a decorative PASS.
3. Run the harness against that snapshot with `DEPLOY_ROOT` pointing to it.
4. Record every PASS, FAIL and SKIP verbatim. A missing artifact is FAIL; an
   unavailable tool is SKIP and never counted as a PASS.

## 3. Run A - baseline, no deployment artifacts (verbatim)

Command: `./scripts/validate-deploy.sh` with the Cargo environment above.

```text
host: angel-nereira-HP-ProBook-440-14-inch-G9-Notebook-PC
date_utc: 2026-11-22T04:47:25Z
commit: e9347ab44b58b0c288a3db433dac327da80ee9c6
deploy_root: /home/angel-nereira/Github-local-angelnereira/worktrees7/deployqa/deploy
yaml_parser: PyYAML 6.0.3 (/usr/bin/python3)
[FAIL] artifact-tree (rc=1): deploy root missing: /home/angel-nereira/Github-local-angelnereira/worktrees7/deployqa/deploy
[FAIL] yaml-parse (rc=1): no YAML artifacts to parse (missing artifact)
[FAIL] dockerfile-user (rc=1): no Dockerfiles under /home/angel-nereira/Github-local-angelnereira/worktrees7/deployqa/deploy (missing artifact)
[FAIL] dockerfile-healthcheck (rc=1): no Dockerfiles under /home/angel-nereira/Github-local-angelnereira/worktrees7/deployqa/deploy (missing artifact)
ag_binary: /home/angel-nereira/Github-local-angelnereira/shared-cargo-target/debug/ag
[FAIL] ag-deploy-plan (rc=1): ag binary has no 'deploy' subcommand: /home/angel-nereira/Github-local-angelnereira/shared-cargo-target/debug/ag
[FAIL] ag-deploy-apply-noplan (rc=1): ag binary has no 'deploy' subcommand: /home/angel-nereira/Github-local-angelnereira/shared-cargo-target/debug/ag
[FAIL] ag-deploy-status-offline (rc=1): ag binary has no 'deploy' subcommand: /home/angel-nereira/Github-local-angelnereira/shared-cargo-target/debug/ag
[FAIL] kustomize-build (rc=1): kustomize installed but no kustomization manifest under /home/angel-nereira/Github-local-angelnereira/worktrees7/deployqa/deploy
[SKIP] kubeconform (rc=127): kubeconform not installed; no k8s schema validation
[SKIP] docker-build (rc=1): docker client Docker version 29.10.0, build fdd4b8e present but daemon unreachable
[SKIP] kubectl-dry-run (rc=1): kubectl client Client Version: v1.37.1 present but no cluster context
summary: PASS=0 FAIL=8 SKIP=3
```

Exit code: 1. The harness correctly refuses to call a missing artifact a SKIP.

## 4. Run B - assembled artifacts before the CLI was compiled (superseded by Run C)

Command:
`DEPLOY_ROOT=/tmp/ag-deploy-merged/deploy ./scripts/validate-deploy.sh`
with the same Cargo environment.

```text
host: angel-nereira-HP-ProBook-440-14-inch-G9-Notebook-PC
date_utc: 2026-11-22T04:50:10Z
commit: e9347ab44b58b0c288a3db433dac327da80ee9c6
deploy_root: /tmp/ag-deploy-merged/deploy
yaml_parser: PyYAML 6.0.3 (/usr/bin/python3)
[PASS] artifact-tree (rc=0): deploy root present with 34 YAML artifact(s)
[PASS] yaml-parse (rc=0): parsed 34/34 YAML artifact(s)
[PASS] dockerfile-user (rc=0): 1 Dockerfile(s), all with a non-root final USER
[PASS] dockerfile-healthcheck (rc=0): 1 service Dockerfile(s), all with HEALTHCHECK
ag_binary: /home/angel-nereira/Github-local-angelnereira/shared-cargo-target/debug/ag
[FAIL] ag-deploy-plan (rc=1): ag binary has no 'deploy' subcommand: /home/angel-nereira/Github-local-angelnereira/shared-cargo-target/debug/ag
[FAIL] ag-deploy-apply-noplan (rc=1): ag binary has no 'deploy' subcommand: /home/angel-nereira/Github-local-angelnereira/shared-cargo-target/debug/ag
[FAIL] ag-deploy-status-offline (rc=1): ag binary has no 'deploy' subcommand: /home/angel-nereira/Github-local-angelnereira/shared-cargo-target/debug/ag
[PASS] kustomize-build (rc=0): 10 kustomization(s) rendered with kustomize build
[SKIP] kubeconform (rc=127): kubeconform not installed; no k8s schema validation
[SKIP] docker-build (rc=1): docker client Docker version 29.10.0, build fdd4b8e present but daemon unreachable
[SKIP] kubectl-dry-run (rc=1): kubectl client Client Version: v1.37.1 present but no cluster context
summary: PASS=5 FAIL=3 SKIP=3
```

Exit code: 1. Every artifact-based check passed with a real tool. The three
CLI failures are expected at this snapshot: the `deploy` role had published its
contract (RFC-0033) but the CLI was still being written. A direct build attempt
from that worktree, with the prescribed Cargo environment, was also run:

```text
cd <deploy worktree>
cargo build --locked --offline -p ag-cli --bin ag
error[E0432]: unresolved import `commands::deploy`
  --> crates/ag-cli/src/main.rs:34:15
error: could not compile `ag-cli` (bin "ag") due to 1 previous error
BUILD_RC=101
```

An earlier attempt in the same loop also reported a non-exhaustive match on
`cli::Commands::Deploy`. Both were in-progress compile errors in another role's
worktree; this validator did not edit them.

## 5. Run C - final authoritative validation (verbatim)

The deploy role finished the CLI during this loop. Rebuilding with the same
read-only, locked command succeeded:

```text
cd <deploy worktree>
cargo build --locked --offline -p ag-cli --bin ag
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.10s
BUILD_RC=0
```

Two harness defects were also found and fixed before this run, both by
inspecting the real CLI output: (a) `ag deploy plan` requires `--target`, with
values `docker` and `k8s`, so the harness now verifies both targets; (b) the
first plan-id extractor could match the temporary project name (`plan-docker`)
or a file fingerprint instead of the id, so it now requires the documented
`dpl-<target>-<16 hex>` form.

Command: same as Run B, after the successful rebuild.

```text
host: angel-nereira-HP-ProBook-440-14-inch-G9-Notebook-PC
date_utc: 2026-11-22T04:54:45Z
commit: e9347ab44b58b0c288a3db433dac327da80ee9c6
deploy_root: /tmp/ag-deploy-merged/deploy
yaml_parser: PyYAML 6.0.3 (/usr/bin/python3)
[PASS] artifact-tree (rc=0): deploy root present with 34 YAML artifact(s)
[PASS] yaml-parse (rc=0): parsed 34/34 YAML artifact(s)
[PASS] dockerfile-user (rc=0): 1 Dockerfile(s), all with a non-root final USER
[PASS] dockerfile-healthcheck (rc=0): 1 service Dockerfile(s), all with HEALTHCHECK
ag_binary: /home/angel-nereira/Github-local-angelnereira/shared-cargo-target/debug/ag
[PASS] ag-deploy-plan-docker (rc=0): plan id=dpl-docker-5639bd90daf2272b; project file set unchanged
[PASS] ag-deploy-plan-k8s (rc=0): plan id=dpl-k8s-b81aef839f319f06; project file set unchanged
[PASS] ag-deploy-apply-noplan (rc=2): apply refused without a plan (required)
[PASS] ag-deploy-status-offline (rc=0): status answered offline: No deployment artifacts generated under deploy/generated/.
[PASS] kustomize-build (rc=0): 10 kustomization(s) rendered with kustomize build
[SKIP] kubeconform (rc=127): kubeconform not installed; no k8s schema validation
[SKIP] docker-build (rc=1): docker client Docker version 29.10.0, build fdd4b8e present but daemon unreachable
[SKIP] kubectl-dry-run (rc=1): kubectl client Client Version: v1.37.1 present but no cluster context
summary: PASS=9 FAIL=0 SKIP=3
```

Exit code: 0. This is the authoritative result. Both plan targets produced a
correctly shaped deterministic id and the project file set was unchanged, so
`plan` is read-only as documented. `apply` refused to run with no plan, and
`status` answered without contacting any external service.

## 6. Harness self-test (synthetic, not product evidence)

Before Run B the harness was exercised against a throwaway fixture under
`/tmp/ag-deploy-harness-selftest` containing one valid YAML manifest,
one kustomization and one Dockerfile with `USER 65532:65532` plus
`HEALTHCHECK`. All artifact steps reported PASS and the script exited 0 when
`AG_BIN` was pointed at a nonexistent path, proving that PASS is reachable and
that a missing `ag` is an honest SKIP. This synthetic run is a test of the
harness only; the product evidence is Run C against real wave-7 artifacts.

## 7. Honest limitations

- Docker: the `docker` client binary is installed (29.10.0) but the daemon is
  not reachable (`Cannot connect to the Docker daemon at
  unix:///home/angel-nereira/.docker/desktop/docker.sock`). `docker build
  --check` is therefore SKIP, never PASS.
- Kubernetes: `kubectl` v1.37.1 and `kustomize` v5.5.0 are installed, but no API
  server answers (`localhost:8080: connect: connection refused`). Server-side
  `kubectl apply --dry-run` is SKIP. Offline `kustomize build` is real and
  passed for all 10 kustomizations.
- `kubeconform` is not installed, so no JSON-schema validation of Kubernetes
  objects was performed. This is a SKIP, not a claim of validity.
- No image was built, no manifest was applied and no network call was made.
- Run A and Run B predate the finished `ag deploy` binary and are kept only as
  honest history. No stub binary was used to fake a CLI PASS; Run C used a real
  binary built from the deploy role's source with a locked, offline build.

## 8. Re-run after the wave-7 branches merge

```bash
cd <repo>
bash -n scripts/validate-deploy.sh
CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target \
CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 \
AG_BIN=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target/debug/ag \
./scripts/validate-deploy.sh
```

Rebuild `ag` first so the binary contains `ag deploy`. The expected outcome
after a correct merge is PASS for every artifact and CLI step: plan ids shaped
like `dpl-<target>-<16 lowercase hex>` for both `docker` and `k8s`, a nonzero
apply exit before any write, and a zero-exit status with no connection error.
The harness verifies no artifact is written by `plan` by fingerprinting the
project file set before and after. `AG_DEPLOY_TARGETS` can narrow the targets;
`DEPLOY_ROOT` can point at the merged artifact tree.

## 9. Verdict

At the reviewed snapshot, the authored deployment artifacts are syntactically
valid: 34/34 YAML documents parse with PyYAML, the single Dockerfile has a
non-root final `USER` and a `HEALTHCHECK`, and 10/10 kustomizations render with
kustomize. The documented `ag deploy` behaviour is confirmed with a real
binary: both plan targets emit deterministic ids without writing files, apply
refuses to run without a plan, and status works offline. Three checks that
need a Docker daemon, a Kubernetes API server or kubeconform are honest SKIPs,
not PASS. No result was fabricated and no blocked check was marked PASS.
