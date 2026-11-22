# Self-hosted runner for the 24-hour fuzz gate

English canonical (ADR-0008). This document prepares a host for the workflow
`.github/workflows/fuzz-24h.yml`, enabled by RFC-0035. It is an operations
guide, not gate evidence. Preparing the runner does **not** close issue #156 or
move the `Fuzz 24h` row to `pass`.

## 1. Scope and non-goals

The gate runs seven `cargo fuzz` targets for 86400 s each, sequentially, for at
least 168 h of wall time. A public GitHub-hosted runner cannot do this: its job
timeout is far shorter than 86400 s and its environment is not quiet. The
workflow therefore targets only the label set `[self-hosted, linux, x64,
fuzz-dedicated]`.

Non-goals:

- No repository secret other than the built-in `GITHUB_TOKEN` is used. Do not
  add cloud credentials, registry tokens or a PAT to this runner.
- No Docker or Kubernetes daemon is required. Do not mount a Docker socket into
  the runner.
- This machine is not a general build runner. It must not accept unrelated
  jobs: noise and load invalidate `exec/s` and RSS measurements.

## 2. Host requirements

| Item | Requirement | Why |
| --- | --- | --- |
| Architecture | Linux x86-64 (`x64`) | All seven harnesses build for this target. |
| Kernel | 6.0 or newer | Record `uname -a` and `/etc/os-release`. |
| CPU | Dedicated, no other heavy load; fixed governor if possible | `exec/s` is resource-sensitive. |
| Memory | Enough that no target is swapped or OOM-killed; record total RAM | An OOM-truncated log is not a completed run. |
| Storage | Dedicated scratch volume, see section 4 | Corpora and logs must not fill the source tree. |
| Power/network | Stable for an uninterrupted 86400 s per target | Any truncated log forces a full restart of that target. |
| Uptime | Capable of 168 h sequential, or seven isolated hosts in parallel | Parallel runs need per-target CPU/RSS allocation recorded. |

Record the environment before the official run:

```sh
uname -a
. /etc/os-release && echo "$PRETTY_NAME"
lscpu | grep '^Model name:' | head -1
nproc
free -h
cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo unknown
df -h <scratch-mount>
```

These are the same fields `scripts/run-fuzz-window.sh` captures in
`metadata.txt` and in the JSONL `host` object
(`docs/fuzz/windows-jsonl-schema.md`).

## 3. Service account and network isolation

1. Run the runner agent as a dedicated, unprivileged service account (for
   example `agfuzz`). It must not be `root`, must not own the source tree, and
   must not have sudo without an explicit audited rule.
2. Do not expose a Docker socket, cloud metadata credential or SSH agent to the
   runner process.
3. Place the host on a segmented network. Default-deny egress; allow only what
   is needed during provisioning:
   - GitHub Actions service endpoints (runner registration and job control).
   - The package/toolchain origins used at setup time (rustup and crates.io)
     if toolchains are not pre-staged.
   During the gate itself no network is required when dependencies are already
   cached; the harness contacts no external service. Restricting egress while
   fuzzing removes a class of supply-chain and exfiltration risk.
4. Inbound: no inbound listener is needed for the runner agent. Do not open a
   port for the gate.
5. The workflow uses only `GITHUB_TOKEN`. Scope it with the workflow's
   permissions: `contents: read` for the gate job, plus `issues: write` in the
   failure-notification job. No other secret exists on this runner.

## 4. Disk sizing

Sizing must be chosen on the real host; do not invent a number as evidence. Use
the following procedure to derive it:

- Seven libFuzzer corpora grow with coverage. Budget a dedicated scratch volume
  of at least 500 GiB free for a sequential 24 h-per-target run, and monitor it.
  This is a starting allocation, not a measured requirement.
- Stop and investigate if free space drops below 20 percent. A full disk can
  truncate logs or corrupt a corpus, and a truncated log is not a valid run.
- Keep the checkout and `CARGO_TARGET_DIR` on the scratch volume too, so a full
  disk cannot corrupt a source tree.
- Corpora under `fuzz/corpus/` are gitignored; they are local cache, not
  committed evidence. Per-target logs, `.rc` files, `metadata.txt`, `timeline.txt`,
  crash inputs and the JSONL file under `docs/fuzz/results/` are uploaded as the
  workflow artifact and are the audit trail.

Check free space before enabling a run:

```sh
df -h <scratch-mount>
find fuzz/corpus -maxdepth 1 -mindepth 1 -type d 2>/dev/null | wc -l
du -sh fuzz/corpus fuzz/artifacts 2>/dev/null || true
```

## 5. Toolchain and tooling

The dated nightly is the reproducibility boundary. The workflow default is
`nightly-2026-11-22`; a maintainer may override it with the repository variable
`FUZZ_NIGHTLY_TOOLCHAIN` or the `toolchain` input on a manual dispatch. Use the
same dated toolchain for all seven targets.

Provision once as the service account:

```sh
rustup toolchain install nightly-2026-11-22 --profile minimal
cargo +nightly-2026-11-22 install cargo-fuzz --locked
# gh is required only by the failure-notification job.
gh --version
```

Do not rely on the floating `nightly` from `fuzz/rust-toolchain.toml` for the
official run. The workflow preflight verifies the exact versions and fails if
they are absent.

## 6. Register the runner

Register a self-hosted runner in the repository settings and assign exactly
these labels:

```
self-hosted
linux
x64
fuzz-dedicated
```

The extra `fuzz-dedicated` label is what prevents an ordinary job from landing
on this host. Removing it, or changing the workflow to a public label, is a CI
change that needs a new RFC under CLAUDE.md rule 28.

## 7. Verification commands (before enabling the scheduled run)

Run these on the prepared host from a checkout of the commit under test. They
verify the runner without starting the 24-hour gate.

```sh
# 1. Environment and labels (runner.environment must be self-hosted).
. /etc/os-release && echo "$PRETTY_NAME"
nproc
df -h <scratch-mount>

# 2. Toolchain and harness availability.
rustc +nightly-2026-11-22 --version
cargo +nightly-2026-11-22 fuzz --version
gh --version

# 3. Workspace hygiene: clean tree and synchronized fuzz lockfile.
git status --porcelain
git rev-parse HEAD
cargo metadata --locked --manifest-path fuzz/Cargo.toml >/dev/null && echo lock-ok

# 4. Harness syntax and interface (does not run any target).
bash -n scripts/run-fuzz-window.sh
bash scripts/run-fuzz-window.sh --help

# 5. Short smoke on the runner: this is NOT the gate.
FUZZ_TOOLCHAIN=nightly-2026-11-22 bash scripts/run-fuzz-window.sh 60
python3 -c 'import json,sys; [json.loads(l) for l in open(sys.argv[1])]' \
  docs/fuzz/results/windows.jsonl
```

Acceptance of the runner setup requires: empty `git status --porcelain`,
`lock-ok`, all three `--version` commands succeeding, and the smoke appending
parseable JSONL with `gate_label` `SHORT_WINDOW_NOT_GATE`. The smoke does not
satisfy issue #156.

## 8. Official run and evidence handling

Trigger the workflow by `workflow_dispatch` (preferred for the first recorded
run) or let the monthly schedule start it. It runs a matrix of seven jobs with
`max-parallel: 1`, one target per job, and calls
`FUZZ_TARGETS=<target> bash scripts/run-fuzz-window.sh "$DURATION"` with
`DURATION=86400`. Each job therefore produces one full-length window and its
JSONL `gate_label` is `GATE_DURATION_MET_MANUAL_REVIEW_REQUIRED` - never a pass
claim. The split is required because a self-hosted job is capped at 5 days and
the token at 24 hours, while the seven windows need about 168 h in total.

On failure the workflow posts one idempotent comment on issue #156, keyed on
the run id, and keeps the `blocked` label. On success or failure it uploads
`docs/fuzz/results/` and `fuzz/artifacts/` as `fuzz-24h-<run_id>`.

A human must then follow `docs/fuzz/24h-gate-verification.md`: verify every log
has the final `DONE` and `Done N runs in M second(s)` lines, triage crashes,
fill `RESULTS.md`, move the `Fuzz 24h` row to `pass` with date, commit and result
path, and update the README blocker text. Only then can #156 be closed with
`Closes #156` in a change that carries the full-length evidence.

## 9. Decommissioning

Remove the workflow and unregister the runner. Corpora and logs on the scratch
volume can be discarded after the evidence has been archived in a completed
run record or git, following the retention policy in the verification package.
Scrubbing the runner account and any cached credentials is part of
decommissioning; the gate itself stores no secret beyond the short-lived token.
